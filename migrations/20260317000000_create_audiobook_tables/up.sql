CREATE TABLE audiobooks (
    id BIGINT PRIMARY KEY,
    title TEXT NOT NULL,
    description TEXT,
    language TEXT,
    copyright_year TEXT,
    num_sections INT,
    total_time TEXT,
    total_time_secs BIGINT,
    authors JSONB NOT NULL DEFAULT '[]',
    url_text_source TEXT,
    url_zip_file TEXT,
    url_librivox TEXT,
    url_iarchive TEXT,
    coverart_url TEXT,
    coverart_thumbnail TEXT,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE chapters (
    id BIGINT PRIMARY KEY,
    audiobook_id BIGINT NOT NULL REFERENCES audiobooks(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    section_number INT NOT NULL,
    duration TEXT,
    duration_seconds INT,
    listen_url TEXT NOT NULL,
    language TEXT,
    readers JSONB NOT NULL DEFAULT '[]',
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE audiobook_search_cache (
    query_hash TEXT PRIMARY KEY,
    query_text TEXT NOT NULL,
    audiobook_ids BIGINT[] NOT NULL,
    total_count INT NOT NULL,
    cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audiobooks_cached_at ON audiobooks(cached_at);
CREATE INDEX idx_chapters_audiobook_id ON chapters(audiobook_id);
CREATE INDEX idx_chapters_cached_at ON chapters(cached_at);
CREATE INDEX idx_audiobook_search_cache_cached_at ON audiobook_search_cache(cached_at);
