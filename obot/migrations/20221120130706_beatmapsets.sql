-- Add migration script here
CREATE TABLE IF NOT EXISTS "beatmapsets" (
    id INTEGER NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    stat TEXT NOT NULL
)