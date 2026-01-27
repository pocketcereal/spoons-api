// @generated automatically by Diesel CLI.

diesel::table! {
    areas (id) {
        id -> Uuid,
        name -> Text,
        sort_name -> Nullable<Text>,
        cached_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    artist_search_cache (query_hash) {
        query_hash -> Text,
        query_text -> Text,
        artist_ids -> Array<Nullable<Uuid>>,
        total_count -> Int8,
        cached_at -> Timestamptz,
    }
}

diesel::table! {
    artists (id) {
        id -> Uuid,
        name -> Text,
        sort_name -> Nullable<Text>,
        artist_type -> Nullable<Text>,
        country -> Nullable<Text>,
        area_id -> Nullable<Uuid>,
        disambiguation -> Nullable<Text>,
        life_span -> Nullable<Jsonb>,
        cached_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    episodes (id) {
        id -> Int8,
        podcast_id -> Int8,
        title -> Text,
        description -> Nullable<Text>,
        audio_url -> Text,
        audio_type -> Nullable<Text>,
        audio_length -> Nullable<Int8>,
        duration_seconds -> Nullable<Int4>,
        published_at -> Nullable<Timestamptz>,
        episode_number -> Nullable<Int4>,
        season_number -> Nullable<Int4>,
        episode_type -> Nullable<Text>,
        image_url -> Nullable<Text>,
        explicit -> Nullable<Bool>,
        cached_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    podcast_search_cache (query_hash) {
        query_hash -> Text,
        query_text -> Text,
        podcast_ids -> Array<Nullable<Int8>>,
        total_count -> Int4,
        cached_at -> Timestamptz,
    }
}

diesel::table! {
    podcast_trending_cache (cache_key) {
        cache_key -> Text,
        podcast_ids -> Array<Nullable<Int8>>,
        cached_at -> Timestamptz,
    }
}

diesel::table! {
    podcasts (id) {
        id -> Int8,
        title -> Text,
        author -> Nullable<Text>,
        description -> Nullable<Text>,
        artwork_url -> Nullable<Text>,
        feed_url -> Text,
        language -> Nullable<Text>,
        categories -> Jsonb,
        itunes_id -> Nullable<Int8>,
        episode_count -> Nullable<Int4>,
        latest_publish_time -> Nullable<Timestamptz>,
        trend_score -> Nullable<Int4>,
        podcast_guid -> Nullable<Text>,
        cached_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    recording_search_cache (query_hash) {
        query_hash -> Text,
        query_text -> Text,
        recording_ids -> Array<Nullable<Uuid>>,
        total_count -> Int8,
        cached_at -> Timestamptz,
    }
}

diesel::table! {
    recordings (id) {
        id -> Uuid,
        title -> Text,
        length_ms -> Nullable<Int8>,
        disambiguation -> Nullable<Text>,
        video -> Nullable<Bool>,
        cached_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    release_group_search_cache (query_hash) {
        query_hash -> Text,
        query_text -> Text,
        release_group_ids -> Array<Nullable<Uuid>>,
        total_count -> Int8,
        cached_at -> Timestamptz,
    }
}

diesel::table! {
    release_groups (id) {
        id -> Uuid,
        title -> Text,
        primary_type -> Nullable<Text>,
        secondary_types -> Nullable<Jsonb>,
        first_release_date -> Nullable<Text>,
        disambiguation -> Nullable<Text>,
        cached_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    release_search_cache (query_hash) {
        query_hash -> Text,
        query_text -> Text,
        release_ids -> Array<Nullable<Uuid>>,
        total_count -> Int8,
        cached_at -> Timestamptz,
    }
}

diesel::table! {
    releases (id) {
        id -> Uuid,
        title -> Text,
        status -> Nullable<Text>,
        release_date -> Nullable<Text>,
        country -> Nullable<Text>,
        barcode -> Nullable<Text>,
        disambiguation -> Nullable<Text>,
        release_group_id -> Nullable<Uuid>,
        cached_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::joinable!(artists -> areas (area_id));
diesel::joinable!(episodes -> podcasts (podcast_id));
diesel::joinable!(releases -> release_groups (release_group_id));

diesel::allow_tables_to_appear_in_same_query!(
    areas,
    artist_search_cache,
    artists,
    episodes,
    podcast_search_cache,
    podcast_trending_cache,
    podcasts,
    recording_search_cache,
    recordings,
    release_group_search_cache,
    release_groups,
    release_search_cache,
    releases,
);
