
const configuration = {
    iceServers: [
        { urls: 'stun:stun.l.google.com:19302' }
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

leaveBtn.addEventListener('click', leave);
toggleMicBtn.addEventListener('click', toggleMic);
toggleVideoBtn.addEventListener('click', toggleVideo);
flipCameraBtn.addEventListener('click', flipCamera);
endCallBtn.addEventListener('click', leave);

function adjustGrid() {
    const count = videosGrid.children.length || 1;
    const cols = Math.ceil(Math.sqrt(count));
    videosGrid.style.gridTemplateColumns = `repeat(${cols}, minmax(0, 1fr))`;
}

function addVideoElement(id, stream, label) {
    const vidId = `video-${id}`;
    let videoEl = document.getElementById(vidId);

    if (!videoEl) {
        const wrapper = document.createElement('div');
        wrapper.className = 'video-wrapper';
        wrapper.id = `wrapper-${id}`;

        videoEl = document.createElement('video');
        videoEl.id = vidId;
        videoEl.autoplay = true;
        videoEl.playsInline = true;
        // videoEl.muted = (id === userId);
        wrapper.appendChild(videoEl);

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

function removeVideoElement(id) {
    const wrapper = document.getElementById(`wrapper-${id}`);
    if (wrapper) wrapper.remove();
    adjustGrid();
}


function updateUsersList(users) {
    if (!users || users.length === 0) {
        usersContent.textContent = 'None';
    } else {
        usersContent.innerHTML = users.map(u => {
            const [id, name] = u;
            return `<div class="user-item">👤 ${name} ${id === userId ? '(You)' : ''}</div>`;
        }).join('');
    }
}

function sendMessage(msg) {
    if (ws && ws.readyState === WebSocket.OPEN) {
        ws.send(JSON.stringify(msg));
    }
}

function updateStatus(text, connected) {
    statusEl.textContent = text;
    statusEl.className = 'status ' + (connected ? 'connected' : 'disconnected');
}

async function connect(uid) {
    userId = parseInt(uid);
    roomId = document.getElementById('roomId').value;

    if (!userId || !roomId) { alert('Please enter User ID and Room ID'); return; }

    try {
        const turnResponse = await fetch('http://localhost:8085/turn-credentials');
        const turnConfig = await turnResponse.json();

        const iceServers = [{ urls: 'stun:stun.l.google.com:19302' }];
        if (turnConfig && turnConfig.data && turnConfig.data.urls) {
            iceServers.push({
                urls: turnConfig.data.urls,
                username: turnConfig.data.username,
                credential: turnConfig.data.credential
            });
        }

        localStream = await navigator.mediaDevices.getUserMedia({
            video: { facingMode: currentFacingMode },
            audio: true
        });

        joinContainer.style.display = 'none';
        videoContainer.classList.add('active');
        controlsRow.style.display = 'flex';

        toggleMicBtn.disabled = false;
        toggleVideoBtn.disabled = false;
        flipCameraBtn.disabled = false;
        leaveBtn.disabled = false;

        addVideoElement(userId, localStream, 'You');

        const serverUrl = `/call/ws/rooms`;
        ws = new WebSocket(`${serverUrl}/${roomId}`);

        ws.onopen = () => {
            updateStatus('Connected', true);
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
            cleanup();
        };

    } catch (err) {
        console.error('Failed to get media / TURN', err);
        alert('Could not access camera/microphone or TURN failed: ' + err);
    }
}

async function handleSignalingMessage(msg, iceServers) {
    switch (msg.type) {
        case 'user-joined':
            updateUsersList(msg.users);

            (msg.users || []).forEach(([remoteId, username]) => {
                if (remoteId === userId) return;
                if (!peerConnections[remoteId]) {
                    startConnection(remoteId, username, true, iceServers);
                }
            });
            break;

        case 'offer':
            await handleOffer(msg, iceServers);
            break;

        case 'answer':
            {
                const remoteId = msg.from ?? msg.user_id ?? msg.sender;
                const pc = peerConnections[remoteId];
                if (pc) {
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

function startConnection(remoteId, label = '', isInitiator = false, iceServers) {
    const pc = new RTCPeerConnection({ iceServers: iceServers });
    peerConnections[remoteId] = pc;

    // Initialize transceivers to maintain m-line order
    pc.addTransceiver('audio', { direction: 'sendrecv' });
    pc.addTransceiver('video', { direction: 'sendrecv' });

    if (localStream) {
        const audioTrack = localStream.getAudioTracks()[0];
        const videoTrack = localStream.getVideoTracks()[0];

        const transceivers = pc.getTransceivers();
        if (audioTrack && transceivers[0]) {
            transceivers[0].sender.replaceTrack(audioTrack);
        }
        if (videoTrack && transceivers[1]) {
            transceivers[1].sender.replaceTrack(videoTrack);
        }
    }

    pc.ontrack = (ev) => {
        const stream = ev.streams && ev.streams[0] ? ev.streams[0] : null;
        if (stream) addVideoElement(remoteId, stream, label || `User ${remoteId}`);
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

    let isNegotiating = false;
    pc.onnegotiationneeded = async () => {
        if (!isInitiator || pc.signalingState !== 'stable' || isNegotiating) {
            return;
        }

        try {
            isNegotiating = true;
            await pc.setLocalDescription();
            sendMessage({
                type: 'offer',
                target_user_id: remoteId,
                sdp: pc.localDescription.sdp
            });
        } catch (err) {
            console.error('onnegotiationneeded failed to create offer', err);
        } finally {
            isNegotiating = false;
        }
    };

    // Monitor connection state to detect disconnections
    pc.oniceconnectionstatechange = () => {
        console.log(`ICE connection state for ${remoteId}: ${pc.iceConnectionState}`);
        if (pc.iceConnectionState === 'failed' || pc.iceConnectionState === 'disconnected') {
            console.warn(`Connection to ${remoteId} ${pc.iceConnectionState}. Attempting ICE restart...`);
            // Optionally trigger ICE restart
            setTimeout(() => {
                if (pc.iceConnectionState === 'disconnected' && isInitiator) {
                    restartIce(remoteId);
                }
            }, 3000);
        }
    };

    pc.onconnectionstatechange = () => {
        console.log(`Connection state for ${remoteId}: ${pc.connectionState}`);
        if (pc.connectionState === 'failed') {
            console.error(`Connection to ${remoteId} failed completely.`);
        }
    };

    if (isInitiator) {
        setTimeout(() => {
            if (pc.signalingState === 'stable') {
                pc.dispatchEvent(new Event('negotiationneeded'));
            }
        }, 0);
    }

    return pc;
}

async function restartIce(remoteId) {
    const pc = peerConnections[remoteId];
    if (!pc || pc.signalingState !== 'stable') return;

    try {
        const offer = await pc.createOffer({ iceRestart: true });
        await pc.setLocalDescription(offer);
        sendMessage({ type: 'offer', target_user_id: remoteId, sdp: offer.sdp });
        console.log(`ICE restart initiated for ${remoteId}`);
    } catch (err) {
        console.error('ICE restart failed:', err);
    }
}

async function handleOffer(msg, iceServers) {
    const remoteId = msg.from ?? msg.user_id ?? msg.sender;
    if (!remoteId) { console.warn('Offer missing remote id'); return; }

    if (peerConnections[remoteId]) {
        try { peerConnections[remoteId].close(); } catch (e) { }
        delete peerConnections[remoteId];
        removeVideoElement(remoteId);
    }

    const pc = new RTCPeerConnection({ iceServers: iceServers });
    peerConnections[remoteId] = pc;

    // Initialize transceivers
    pc.addTransceiver('audio', { direction: 'sendrecv' });
    pc.addTransceiver('video', { direction: 'sendrecv' });

    if (localStream) {
        const audioTrack = localStream.getAudioTracks()[0];
        const videoTrack = localStream.getVideoTracks()[0];

        const transceivers = pc.getTransceivers();
        if (audioTrack && transceivers[0]) {
            transceivers[0].sender.replaceTrack(audioTrack);
        }
        if (videoTrack && transceivers[1]) {
            transceivers[1].sender.replaceTrack(videoTrack);
        }
    }

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

    // Monitor connection state
    pc.oniceconnectionstatechange = () => {
        console.log(`ICE connection state for ${remoteId}: ${pc.iceConnectionState}`);
    };

    pc.onconnectionstatechange = () => {
        console.log(`Connection state for ${remoteId}: ${pc.connectionState}`);
    };

    try {
        await pc.setRemoteDescription({ type: 'offer', sdp: msg.sdp });
        const ans = await pc.createAnswer();
        await pc.setLocalDescription(ans);
        sendMessage({ type: 'answer', target_user_id: remoteId, sdp: ans.sdp });
    } catch (err) {
        console.error('handleOffer error', err);
    }
}

function toggleMic() {
    if (!localStream) return;
    const track = localStream.getAudioTracks()[0];
    if (!track) return;
    track.enabled = !track.enabled;
    const on = track.enabled;
    micIcon.className = on ? "fa-solid fa-microphone" : "fa-solid fa-microphone-slash";
    toggleMicBtn.classList.toggle('off', !on);
}

function toggleVideo() {
    if (!localStream) return;
    const track = localStream.getVideoTracks()[0];
    if (!track) return;
    track.enabled = !track.enabled;
    const on = track.enabled;
    videoIcon.className = on ? "fa-solid fa-video" : "fa-solid fa-video-slash";
    toggleVideoBtn.classList.toggle('off', !on);
}

async function flipCamera() {
    if (!localStream) {
        console.error("Local stream not available.");
        return;
    }

    const wasVideoEnabled = localStream.getVideoTracks()[0]?.enabled ?? false;
    const oldVideoTrack = localStream.getVideoTracks()[0];

    try {
        currentFacingMode = currentFacingMode === 'user' ? 'environment' : 'user';

        const newStream = await navigator.mediaDevices.getUserMedia({
            video: { facingMode: currentFacingMode },
            audio: false // Don't request audio, we already have it
        });

        const newVideoTrack = newStream.getVideoTracks()[0];
        if (newVideoTrack) {
            newVideoTrack.enabled = wasVideoEnabled;

            // Replace video track in all peer connections
            Object.values(peerConnections).forEach(pc => {
                const transceivers = pc.getTransceivers();
                const videoTransceiver = transceivers.find(t => t.sender.track?.kind === 'video' || t.receiver.track?.kind === 'video');

                if (videoTransceiver && videoTransceiver.sender) {
                    videoTransceiver.sender.replaceTrack(newVideoTrack)
                        .catch(err => console.error("replaceTrack video failed:", err));
                }
            });

            // Update local stream
            if (oldVideoTrack) {
                localStream.removeTrack(oldVideoTrack);
                oldVideoTrack.stop();
            }
            localStream.addTrack(newVideoTrack);

            // Update local video element
            const videoElement = document.getElementById(`video-${userId}`);
            if (videoElement) {
                videoElement.srcObject = localStream;
                videoElement.style.transform = currentFacingMode === 'user' ? 'scaleX(-1)' : '';
            }
        }

    } catch (err) {
        console.error("Error flipping camera:", err);
        alert("Unable to flip camera: " + err.message);
        // Revert facing mode on error
        currentFacingMode = currentFacingMode === 'user' ? 'environment' : 'user';
    }
}

function leave() {
    if (ws && ws.readyState === WebSocket.OPEN) {
        sendMessage({ type: 'leave', room_id: roomId });
        ws.close();
    }
    cleanup();
}

function cleanup() {
    Object.values(peerConnections).forEach(pc => {
        try { pc.close(); } catch (e) { }
    });
    peerConnections = {};

    if (localStream) localStream.getTracks().forEach(t => { try { t.stop(); } catch (e) { } });
    localStream = null;

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
    currentFacingMode = 'user';

    updateStatus('Disconnected', false);
}

window.addEventListener('beforeunload', () => {
    try { leave(); } catch (e) { }
});
