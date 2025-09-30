-- Add migration script here

CREATE TABLE call_participants (
    call_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    joined_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    left_at TEXT,
    duration INTEGER,
    PRIMARY KEY (call_id, user_id),
    FOREIGN KEY (call_id) REFERENCES calls(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
);

CREATE INDEX idx_call_participants_user_id ON call_participants(user_id);

CREATE INDEX idx_call_participants_left_at ON call_participants(left_at);

CREATE INDEX idx_call_participants_user_joined ON call_participants(user_id, joined_at DESC);
