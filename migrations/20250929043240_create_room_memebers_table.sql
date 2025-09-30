-- Add migration script here
CREATE TABLE room_members (
    room_id TEXT NOT NULL REFERENCES rooms(id) ON DELETE CASCADE,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    role TEXT NOT NULL DEFAULT 'participant',
    left_at DATETIME,
    is_muted BOOLEAN NOT NULL DEFAULT FALSE,
    is_video_enabled BOOLEAN NOT NULL DEFAULT TRUE,
    PRIMARY KEY (room_id, user_id)
);

CREATE INDEX idx_room_members_room_id_left_at ON room_members (room_id, left_at);

