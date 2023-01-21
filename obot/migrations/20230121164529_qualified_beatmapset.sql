-- if there are multiple difficulties and keys, devide by a space
-- !!! Diffs and keys must be in the same order !!!
CREATE TABLE IF NOT EXISTS "ranked_beatmapsets" (
    id INTEGER NOT NULL,
    title TEXT NOT NULL,
    artist TEXT NOT NULL,
    creator TEXT NOT NULL,
    stars TEXT NOT NULL,
    keys TEXT NOT NULL,
    card_url TEXT NOT NULL
)