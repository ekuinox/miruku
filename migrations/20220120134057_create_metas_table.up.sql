-- Add up migration script here
-- Add migration script here
CREATE TABLE metas (
    media_id TEXT PRIMARY KEY NOT NULL,
    origin TEXT NOT NULL,
    visibility INTEGER NOT NULL,
    date DATETIME NOT NULL,
    attributes JSON
)
