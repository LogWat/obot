-- Add migration script here
CREATE TABLE IF NOT EXISTS "beatmaps" (
    id SERIAL PRIMARY KEY,
    beatmapset_id INTEGER NOT NULL,
    difficulty_rating FLOAT NOT NULL,
    statu VARCHAR(255) NOT NULL
)