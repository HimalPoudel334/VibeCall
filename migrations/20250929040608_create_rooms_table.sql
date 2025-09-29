-- Add migration script here
CREATE TABLE rooms (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    room_type TEXT NOT NULL CHECK (room_type IN ('public', 'private', 'one_on_one', 'group', 'instant', 'meeting')),
    created_by INTEGER NOT NULL,
    description TEXT,
    max_participants INTEGER DEFAULT 10,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TEXT NOT NULL DEFAULT (CURRENT_TIMESTAMP),
    FOREIGN KEY (created_by) REFERENCES users(id)
);
