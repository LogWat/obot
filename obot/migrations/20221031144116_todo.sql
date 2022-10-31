-- Add migration script here
CREATE TABLE IF NOT EXISTS "todo" (
    user_id INTEGER NOT NULL,
    todo TEXT NOT NULL
)