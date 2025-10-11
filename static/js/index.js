
/*
    Single-file VibeCall:
    - Multi-peer: one RTCPeerConnection per remote user (peerConnections map)
    - Responsive square-ish grid (2x2, 3x3, ...) using columns = ceil(sqrt(n))
    - Flip camera: replace video track in all RTCPeerConnections so remote users see new camera (FIXED)
    - Mic/Video toggle; End Call to cleanup & return to join UI
    - kept TURN / signaling calls pattern (adjust URL to your server)
*/

const configuration = {
    iceServers: [
        { urls: 'stun:stun.l.google.com:19302' }
        // TURN will be set after fetching credentials
    ]
};

// Signaling / state
let ws = null;
let peerConnections = {}; // remoteId -> RTCPeerConnection
let localStream = null;
let userId = null;
let roomId = null;
let currentFacingMode = 'user';

// UI elements
const joinContainer = document.getElementById('joinContainer');
const connectBtn = document.getElementById('connectBtn');
const leaveBtn = document.getElementById('leaveBtn');
const statusEl = document.getElementById('status');

const videoContainer = document.getElementById('videoContainer');
const videosGrid = document.getElementById('videosGrid');
const controlsRow = document.getElementById('controlsRow');

const toggleMicBtn = document.getElementById('toggleMicBtn');
const toggleVideoBtn = document.getElementById('toggleVideoBtn');
const flipCameraBtn = document.getElementById('flipCameraBtn');
const endCallBtn = document.getElementById('endCallBtn');

const micIcon = document.getElementById('micIcon');
const videoIcon = document.getElementById('videoIcon');

const usersContent = document.getElementById('usersContent');

connectBtn.addEventListener('click', connect);
leaveBtn.addEventListener('click', leave);
toggleMicBtn.addEventListener('click', toggleMic);
toggleVideoBtn.addEventListener('click', toggleVideo);
flipCameraBtn.addEventListener('click', flipCamera);
endCallBtn.addEventListener('click', leave);

// Utility: adjust grid columns based on participant count
function adjustGrid() {
    // count of participants (videos present)
    const count = videosGrid.children.length || 1;
    const cols = Math.ceil(Math.sqrt(count));
    videosGrid.style.gridTemplateColumns = `repeat(${cols}, minmax(0, 1fr))`;
}

// Add / update a video element for a given user id
function addVideoElement(id, stream, label) {
    const vidId = `video-${id}`;
    let videoEl = document.getElementById(vidId);

    if (!videoEl) {
        // create wrapper + video
        const wrapper = document.createElement('div');
        wrapper.className = 'video-wrapper';
        wrapper.id = `wrapper-${id}`;

        videoEl = document.createElement('video');
        videoEl.id = vidId;
        videoEl.autoplay = true;
        videoEl.playsInline = true;
        videoEl.muted = (id === userId); // mute local preview
        wrapper.appendChild(videoEl);

        // label
        const lbl = document.createElement('div');
        lbl.className = 'video-label';
        lbl.textContent = label || `User ${id}`;
        wrapper.appendChild(lbl);

        videosGrid.appendChild(wrapper);
    }

    videoEl.srcObject = stream;

    // mirror local preview for user comfort
    if (id === userId) {
        videoEl.style.transform = currentFacingMode === 'user' ? 'scaleX(-1)' : '';
    } else {
        videoEl.style.transform = '';
    }

    adjustGrid();
}

// Remove a video element
function removeVideoElement(id) {
    const wrapper = document.getElementById(`wrapper-${id}`);
    if (wrapper) wrapper.remove();
    adjustGrid();
}

// update users list UI
function updateUsersList(users) {
    if (!users || users.length === 0) {
        usersContent.textContent = 'None';
    } else {
        usersContent.innerHTML = users.map(u =>
            `<div class="user-item">👤 User ${u}${u === userId ? ' (You)' : ''}</div>`
        ).join('');
    }
}

// Send message helper (only if ws open)
function sendMessage(msg) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(msg));
    }
}

function updateStatus(text, connected) {
    statusEl.textContent = text;
    statusEl.className = 'status ' + (connected ? 'connected' : 'disconnected');
}

// connect and join
async function connect() {
    userId = parseInt(document.getElementById('userId').value);
    roomId = document.getElementById('roomId').value;

    if (!userId || !roomId) { alert('Please enter User ID and Room ID'); return; }

    try {
        // keep your TURN credentials call exactly as you used
        const turnResponse = await fetch(`https://himalpoudel.name.np/vibecall/turn-credentials?user_id=${userId}`);
        const turnConfig = await turnResponse.json();

        // append TURN server to ICE servers
        const iceServers = [{ urls: 'stun:stun.l.google.com:19302' }];
        if (turnConfig && turnConfig.urls) {
            iceServers.push({
                urls: turnConfig.urls,
                username: turnConfig.username,
                credential: turnConfig.credential
            });
        }
        // NOTE: Configuration for new PCs must be passed in startConnection/handleOffer

        // get media with the requested facing mode
        localStream = await navigator.mediaDevices.getUserMedia({
            video: { facingMode: currentFacingMode },
            audio: true
        });

        // show UI: hide joinContainer, show videoContainer & controls
        joinContainer.style.display = 'none';
        videoContainer.classList.add('active');
        controlsRow.style.display = 'flex';

        // enable control buttons
        toggleMicBtn.disabled = false;
        toggleVideoBtn.disabled = false;
        flipCameraBtn.disabled = false;
        leaveBtn.disabled = false;

        // add local video element (userId)
        addVideoElement(userId, localStream, 'You (Local)');

        // connect signaling websocket (your server path)
        const serverUrl = `wss://himalpoudel.name.np/vibecall/call/ws/rooms`;
        ws = new WebSocket(`${serverUrl}/${roomId}?user_id=${userId}`);

        ws.onopen = () => {
            updateStatus('Connected', true);
            // tell server we joined
            sendMessage({ type: 'join', room_id: roomId, user_id: userId });
            connectBtn.disabled = true;
        };

        ws.onmessage = async (evt) => {
            try {
                const msg = JSON.parse(evt.data);
                await handleSignalingMessage(msg, iceServers);
            } catch (err) {
                console.error('Invalid message', err);
            }
        };

        ws.onerror = (e) => console.error('WebSocket error', e);
        ws.onclose = () => {
            // server closed
            cleanup();
        };

    } catch (err) {
        console.error('Failed to get media / TURN', err);
        alert('Could not access camera/microphone or TURN failed: ' + err);
    }
}

// handle various signaling messages
async function handleSignalingMessage(msg, iceServers) {
    switch (msg.type) {
        case 'user-joined':
            updateUsersList(msg.users);

            (msg.users || []).forEach(remoteId => {
                if (remoteId === userId) return;
                // Only initiate connection if PC does not exist
                if (!peerConnections[remoteId]) {
                    // Check if *this* user is the initiator (lower ID is the standard initiator)
                    const isInitiator = userId < remoteId;
                    if (isInitiator) {
                        startConnection(remoteId, /*isInitiator=*/ true, iceServers);
                    }
                }
            });
            break;

        case 'offer':
            // incoming offer: msg.from contains remote user id
            await handleOffer(msg, iceServers);
            break;

        case 'answer':
            // incoming answer to our offer
            {
                const remoteId = msg.from ?? msg.user_id ?? msg.sender;
                const pc = peerConnections[remoteId];
                if (pc) {
                    // FIX: Check signaling state to avoid InvalidStateError (though offerer should be waiting for answer)
                    if (pc.signalingState !== 'stable') {
                        await pc.setRemoteDescription({ type: 'answer', sdp: msg.sdp });
                    } else {
                        console.warn(`Answer received for peer ${remoteId} but signaling state is stable. Ignoring.`);
                    }
                } else {
                    console.warn('Answer for unknown peer', remoteId);
                }
            }
            break;

        case 'ice-candidate':
            {
                const remoteId = msg.from ?? msg.user_id ?? msg.sender;
                const pc = peerConnections[remoteId];
                if (pc && msg.candidate) {
                    try {
                        await pc.addIceCandidate({
                            candidate: msg.candidate,
                            sdpMid: msg.sdp_mid,
                            sdpMLineIndex: msg.sdp_m_line_index
                        });
                    } catch (err) {
                        console.warn('addIceCandidate failed', err);
                    }
                }
            }
            break;

        case 'user-left':
            const uid = msg.user_id;
            if (peerConnections[uid]) {
                try { peerConnections[uid].close(); } catch (e) { }
                delete peerConnections[uid];
            }
            removeVideoElement(uid);
            updateUsersList(msg.users);
            break;

        case 'error':
            console.error('Server error:', msg.message);
            alert('Server error: ' + msg.message);
            break;

        default:
            console.log('Unknown msg type', msg.type);
    }
}

// create a peer connection for a remote user
function startConnection(remoteId, isInitiator = false, iceServers) {
    const pc = new RTCPeerConnection({ iceServers: iceServers }); // Pass full config here
    peerConnections[remoteId] = pc;

    // add local tracks
    if (localStream) {
        localStream.getTracks().forEach(track => pc.addTrack(track, localStream));
    }

    pc.ontrack = (ev) => {
        // show remote stream in grid
        const stream = ev.streams && ev.streams[0] ? ev.streams[0] : null;
        if (stream) addVideoElement(remoteId, stream, `User ${remoteId}`);
    };

    pc.onicecandidate = (ev) => {
        if (ev.candidate) {
            sendMessage({
                type: 'ice_candidate',
                target_user_id: remoteId,
                candidate: ev.candidate.candidate,
                sdp_mid: ev.candidate.sdpMid,
                sdp_m_line_index: ev.candidate.sdpMLineIndex
            });
        }
    };

    pc.onnegotiationneeded = async () => {
        if (isInitiator && pc.signalingState === 'stable') {
            try {
                const offer = await pc.createOffer();
                await pc.setLocalDescription(offer);
                sendMessage({ type: 'offer', target_user_id: remoteId, sdp: offer.sdp });
            } catch (err) {
                console.error('onnegotiationneeded failed to create offer', err);
            }
        }
    };

    pc.onconnectionstatechange = () => {
        // console.log('pc state for', remoteId, pc.connectionState);
    };

    if (isInitiator) {
        // Initial offer will be triggered by onnegotiationneeded after tracks are added
        // Forcing a negotiation needed now if necessary
        if (pc.signalingState === 'stable') {
            pc.dispatchEvent(new Event('negotiationneeded'));
        }
    }

    return pc;
}

// handle incoming offer
async function handleOffer(msg, iceServers) {
    const remoteId = msg.from ?? msg.user_id ?? msg.sender;
    if (!remoteId) { console.warn('Offer missing remote id'); return; }

    // if pc exists, close & re-create (clean state for a new offer)
    if (peerConnections[remoteId]) {
        try { peerConnections[remoteId].close(); } catch (e) { }
        delete peerConnections[remoteId];
        removeVideoElement(remoteId); // Clean up the remote video too, in case of a crash/reconnect
    }

    const pc = new RTCPeerConnection({ iceServers: iceServers }); // Pass full config here
    peerConnections[remoteId] = pc;

    // add local tracks
    if (localStream) localStream.getTracks().forEach(t => pc.addTrack(t, localStream));

    pc.ontrack = (ev) => {
        const stream = ev.streams && ev.streams[0] ? ev.streams[0] : null;
        if (stream) addVideoElement(remoteId, stream, `User ${remoteId}`);
    };

    pc.onicecandidate = (ev) => {
        if (ev.candidate) {
            sendMessage({
                type: 'ice_candidate',
                target_user_id: remoteId,
                candidate: ev.candidate.candidate,
                sdp_mid: ev.candidate.sdpMid,
                sdp_m_line_index: ev.candidate.sdpMLineIndex
            });
        }
    };

    // set remote offer, create & send answer
    try {
        await pc.setRemoteDescription({ type: 'offer', sdp: msg.sdp });
        const ans = await pc.createAnswer();
        await pc.setLocalDescription(ans);
        sendMessage({ type: 'answer', target_user_id: remoteId, sdp: ans.sdp });
    } catch (err) {
        console.error('handleOffer error', err);
    }
}

// Mic toggle
function toggleMic() {
    if (!localStream) return;
    const track = localStream.getAudioTracks()[0];
    if (!track) return;
    track.enabled = !track.enabled;
    const on = track.enabled;
    micIcon.className = on ? "fa-solid fa-microphone" : "fa-solid fa-microphone-slash";
    toggleMicBtn.classList.toggle('off', !on);
}

// Video toggle (enable/disable local video track)
function toggleVideo() {
    if (!localStream) return;
    const track = localStream.getVideoTracks()[0];
    if (!track) return;
    track.enabled = !track.enabled;
    const on = track.enabled;
    videoIcon.className = on ? "fa-solid fa-video" : "fa-solid fa-video-slash";
    toggleVideoBtn.classList.toggle('off', !on);
}

// Flip camera: get new stream with facingMode and replace video track in all peer connections
async function flipCamera() {
    if (!localStream) {
        console.error("Local stream not available.");
        return;
    }

    // Capture the state of tracks we want to maintain
    const wasVideoEnabled = localStream.getVideoTracks()[0]?.enabled ?? false;
    const audioTrack = localStream.getAudioTracks()[0];
    const wasAudioEnabled = audioTrack?.enabled ?? false;

    // Stop all current tracks before switching
    localStream.getTracks().forEach(track => track.stop());

    try {
        // Determine new facing mode
        currentFacingMode = currentFacingMode === 'user' ? 'environment' : 'user';

        // Request new stream with opposite facing mode
        const newStream = await navigator.mediaDevices.getUserMedia({
            video: { facingMode: currentFacingMode },
            audio: true // Request audio again, we'll re-enable/disable based on previous state
        });

        const newVideoTrack = newStream.getVideoTracks()[0];
        const newAudioTrack = newStream.getAudioTracks()[0];

        // Re-apply previous enabled state
        if (newVideoTrack) newVideoTrack.enabled = wasVideoEnabled;
        if (newAudioTrack) newAudioTrack.enabled = wasAudioEnabled;

        // 1. Update the local video element
        const videoElement = document.getElementById(`video-${userId}`);
        if (videoElement) {
            videoElement.srcObject = newStream;
            // Mirror local preview for user comfort
            videoElement.style.transform = currentFacingMode === 'user' ? 'scaleX(-1)' : '';
        } else {
            console.error(`Local video element with ID video-${userId} not found.`);
        }

        // 2. Replace track in every RTCPeerConnection
        if (newVideoTrack) {
            Object.values(peerConnections).forEach(pc => {
                const videoSender = pc.getSenders().find(s => s.track?.kind === "video");
                const audioSender = pc.getSenders().find(s => s.track?.kind === "audio");

                if (videoSender) {
                    videoSender.replaceTrack(newVideoTrack).catch(err => console.error("replaceTrack video failed:", err));
                }

                if (newAudioTrack && audioSender) {
                    audioSender.replaceTrack(newAudioTrack).catch(err => console.error("replaceTrack audio failed:", err));
                }
            });
        }

        // Update the global localStream reference
        localStream = newStream;

    } catch (err) {
        console.error("Error flipping camera:", err);
        alert("Unable to flip camera: " + err.message);
    }
}

// leave / end call
function leave() {
    if (ws && ws.readyState === WebSocket.OPEN) {
        sendMessage({ type: 'leave', room_id: roomId });
        ws.close();
    }
    cleanup();
}

function cleanup() {
    // close all peer connections
    Object.values(peerConnections).forEach(pc => {
        try { pc.close(); } catch (e) { }
    });
    peerConnections = {};

    // stop local tracks
    if (localStream) localStream.getTracks().forEach(t => { try { t.stop(); } catch (e) { } });
    localStream = null;

    // reset UI
    videosGrid.innerHTML = '';
    updateUsersList([]);
    videoContainer.classList.remove('active');
    controlsRow.style.display = 'none';
    joinContainer.style.display = 'block';
    connectBtn.disabled = false;
    leaveBtn.disabled = true;
    toggleMicBtn.disabled = true;
    toggleVideoBtn.disabled = true;
    flipCameraBtn.disabled = true;
    currentFacingMode = 'user'; // Reset camera to default (front)

    updateStatus('Disconnected', false);
}

/* Optional: graceful cleanup when user navigates away */
window.addEventListener('beforeunload', () => {
    try { leave(); } catch (e) { }
});

/* END of script */
