-- Create podcast cache tables for PodcastIndex data

-- Podcasts table
CREATE TABLE podcasts (
    id BIGINT PRIMARY KEY,              -- PodcastIndex feed_id
    title TEXT NOT NULL,
    author TEXT,
    description TEXT,
    artwork_url TEXT,
    feed_url TEXT NOT NULL,
    language TEXT,
    categories JSONB NOT NULL DEFAULT '[]',
    itunes_id BIGINT,
    episode_count INT,
    latest_publish_time TIMESTAMPTZ,
    trend_score INT,
    podcast_guid TEXT,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Episodes table
CREATE TABLE episodes (
    id BIGINT PRIMARY KEY,              -- PodcastIndex episode_id
    podcast_id BIGINT NOT NULL REFERENCES podcasts(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    audio_url TEXT NOT NULL,
    audio_type TEXT,
    audio_length BIGINT,
    duration_seconds INT,
    published_at TIMESTAMPTZ,
    episode_number INT,
    season_number INT,
    episode_type TEXT,
    image_url TEXT,
    explicit BOOLEAN,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Podcast search cache table
CREATE TABLE podcast_search_cache (
    query_hash TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    podcast_ids BIGINT[] NOT NULL,
    total_count INT NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Podcast trending cache table
CREATE TABLE podcast_trending_cache (
    cache_key TEXT PRIMARY KEY,
    podcast_ids BIGINT[] NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for faster lookups
CREATE INDEX idx_podcasts_title ON podcasts(title);
CREATE INDEX idx_podcasts_itunes_id ON podcasts(itunes_id);
CREATE INDEX idx_episodes_podcast_id ON episodes(podcast_id);
CREATE INDEX idx_episodes_published_at ON episodes(published_at);

-- Indexes for cache expiry queries
CREATE INDEX idx_podcasts_cached_at ON podcasts(cached_at);
CREATE INDEX idx_episodes_cached_at ON episodes(cached_at);
CREATE INDEX idx_podcast_search_cache_cached_at ON podcast_search_cache(cached_at);
