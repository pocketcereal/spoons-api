-- Create music cache tables for MusicBrainz data

-- Areas table (shared reference for artists)
CREATE TABLE areas (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    sort_name TEXT,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Artists table
CREATE TABLE artists (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    sort_name TEXT,
    artist_type TEXT,
    country TEXT,
    area_id UUID REFERENCES areas(id) ON DELETE SET NULL,
    disambiguation TEXT,
    life_span JSONB,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Release groups table
CREATE TABLE release_groups (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    primary_type TEXT,
    secondary_types JSONB,
    first_release_date TEXT,
    disambiguation TEXT,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Releases table
CREATE TABLE releases (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    status TEXT,
    release_date TEXT,
    country TEXT,
    barcode TEXT,
    disambiguation TEXT,
    release_group_id UUID REFERENCES release_groups(id) ON DELETE SET NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Recordings table
CREATE TABLE recordings (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    length_ms BIGINT,
    disambiguation TEXT,
    video BOOLEAN,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Search result cache tables
CREATE TABLE artist_search_cache (
    query_hash TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    artist_ids UUID[] NOT NULL,
    total_count BIGINT NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE release_search_cache (
    query_hash TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    release_ids UUID[] NOT NULL,
    total_count BIGINT NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE recording_search_cache (
    query_hash TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    recording_ids UUID[] NOT NULL,
    total_count BIGINT NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE release_group_search_cache (
    query_hash TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    release_group_ids UUID[] NOT NULL,
    total_count BIGINT NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for faster lookups
CREATE INDEX idx_artists_name ON artists(name);
CREATE INDEX idx_artists_area_id ON artists(area_id);
CREATE INDEX idx_releases_title ON releases(title);
CREATE INDEX idx_releases_release_group_id ON releases(release_group_id);
CREATE INDEX idx_recordings_title ON recordings(title);
CREATE INDEX idx_release_groups_title ON release_groups(title);

-- Indexes for cache expiry queries
CREATE INDEX idx_artists_cached_at ON artists(cached_at);
CREATE INDEX idx_releases_cached_at ON releases(cached_at);
CREATE INDEX idx_recordings_cached_at ON recordings(cached_at);
CREATE INDEX idx_release_groups_cached_at ON release_groups(cached_at);
CREATE INDEX idx_artist_search_cache_cached_at ON artist_search_cache(cached_at);
CREATE INDEX idx_release_search_cache_cached_at ON release_search_cache(cached_at);
CREATE INDEX idx_recording_search_cache_cached_at ON recording_search_cache(cached_at);
CREATE INDEX idx_release_group_search_cache_cached_at ON release_group_search_cache(cached_at);
