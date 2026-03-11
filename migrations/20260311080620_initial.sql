-- Add migration script here
CREATE TABLE maps (
    id          INTEGER PRIMARY KEY,
    name        TEXT NOT NULL,
    tier_soldier SMALLINT NOT NULL,
    tier_demoman SMALLINT NOT NULL,
    rating_soldier SMALLINT NOT NULL,
    rating_demoman SMALLINT NOT NULL,
    zone_counts JSONB NOT NULL DEFAULT '{}',
    authors     JSONB NOT NULL DEFAULT '[]',
    video_soldier TEXT,
    video_demoman TEXT,
    fetched_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE users (
    id       INTEGER PRIMARY KEY,
    steamid  TEXT NOT NULL,
    name     TEXT NOT NULL
);

CREATE TABLE records (
    id       INTEGER PRIMARY KEY,
    map_id   INTEGER NOT NULL REFERENCES maps(id),
    user_id  INTEGER NOT NULL REFERENCES users(id),
    class    SMALLINT NOT NULL,
    duration DOUBLE PRECISION NOT NULL,
    date     TIMESTAMPTZ NOT NULL,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (user_id, map_id, class)
);

CREATE INDEX idx_results_user_class ON records(user_id, class);
CREATE INDEX idx_maps_tier_soldier ON maps(tier_soldier);
CREATE INDEX idx_maps_tier_demoman ON maps(tier_demoman);