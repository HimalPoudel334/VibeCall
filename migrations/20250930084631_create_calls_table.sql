-- Add migration script here
CREATE TABLE calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id TEXT NOT NULL,
    caller_id INTEGER NOT NULL,
    status TEXT NOT NULL CHECK (status IN ('initiated', 'ringing', 'active', 'ended', 'missed', 'rejected', 'failed')),
    started_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    ended_at TEXT,
    duration INTEGER,
    FOREIGN KEY (room_id) REFERENCES rooms(id),
    FOREIGN KEY (caller_id) REFERENCES users(id)
);

CREATE INDEX idx_calls_room_id ON calls(room_id);

CREATE INDEX idx_calls_caller_id ON calls(caller_id);

CREATE INDEX idx_calls_status ON calls(status);

CREATE INDEX idx_calls_caller_started ON calls(caller_id, started_at DESC);
