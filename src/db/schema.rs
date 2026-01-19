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
diesel::joinable!(releases -> release_groups (release_group_id));

diesel::allow_tables_to_appear_in_same_query!(
    areas,
    artist_search_cache,
    artists,
    recording_search_cache,
    recordings,
    release_group_search_cache,
    release_groups,
    release_search_cache,
    releases,
);
