//! Podcast repository for database operations.

use chrono::{Duration, Utc};
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::db::models::{NewPodcastRow, PodcastRow};
use crate::db::schema::podcasts;
use crate::db::{DbPool, db_error, get_conn, validate_batch_size};
use crate::error::Result;
use crate::podcast::Podcast;

/// Repository for podcast database operations.
pub struct PodcastRepository;

impl PodcastRepository {
    /// Get a podcast by ID (regardless of cache expiry).
    pub async fn get_by_id(pool: &DbPool, id: i64) -> Result<Option<Podcast>> {
        let mut conn = get_conn(pool).await?;

        let result = podcasts::table
            .filter(podcasts::id.eq(id))
            .select(PodcastRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get podcast"))?;

        Ok(result.map(Into::into))
    }

    /// Get multiple podcasts by their IDs.
    ///
    /// # Errors
    /// Returns an error if the batch size exceeds the maximum allowed (100).
    pub async fn get_by_ids(pool: &DbPool, ids: &[i64]) -> Result<Vec<Podcast>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size(ids.len())?;

        let mut conn = get_conn(pool).await?;

        let results: Vec<PodcastRow> = podcasts::table
            .filter(podcasts::id.eq_any(ids))
            .select(PodcastRow::as_select())
            .load(&mut conn)
            .await
            .map_err(db_error("Failed to get podcasts by IDs"))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    /// Get a cached podcast by ID if not expired.
    pub async fn get_cached(pool: &DbPool, id: i64, ttl_seconds: i64) -> Result<Option<Podcast>> {
        let min_cached_at = Utc::now() - Duration::seconds(ttl_seconds);
        let mut conn = get_conn(pool).await?;

        let result = podcasts::table
            .filter(podcasts::id.eq(id))
            .filter(podcasts::cached_at.gt(min_cached_at))
            .select(PodcastRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get cached podcast"))?;

        Ok(result.map(Into::into))
    }

    /// Upsert a podcast (insert or update).
    pub async fn upsert(pool: &DbPool, podcast: &Podcast) -> Result<()> {
        let mut conn = get_conn(pool).await?;
        let new_podcast = NewPodcastRow::from(podcast);

        diesel::insert_into(podcasts::table)
            .values(&new_podcast)
            .on_conflict(podcasts::id)
            .do_update()
            .set((
                podcasts::title.eq(&new_podcast.title),
                podcasts::author.eq(&new_podcast.author),
                podcasts::description.eq(&new_podcast.description),
                podcasts::artwork_url.eq(&new_podcast.artwork_url),
                podcasts::feed_url.eq(&new_podcast.feed_url),
                podcasts::language.eq(&new_podcast.language),
                podcasts::categories.eq(&new_podcast.categories),
                podcasts::itunes_id.eq(&new_podcast.itunes_id),
                podcasts::episode_count.eq(&new_podcast.episode_count),
                podcasts::latest_publish_time.eq(&new_podcast.latest_publish_time),
                podcasts::trend_score.eq(&new_podcast.trend_score),
                podcasts::podcast_guid.eq(&new_podcast.podcast_guid),
                podcasts::updated_at.eq(Utc::now()),
                podcasts::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to upsert podcast"))?;

        Ok(())
    }

    /// Upsert multiple podcasts using batch operations.
    pub async fn upsert_many(pool: &DbPool, podcast_list: &[Podcast]) -> Result<()> {
        if podcast_list.is_empty() {
            return Ok(());
        }

        let mut conn = get_conn(pool).await?;

        let new_podcasts: Vec<NewPodcastRow> =
            podcast_list.iter().map(NewPodcastRow::from).collect();

        diesel::insert_into(podcasts::table)
            .values(&new_podcasts)
            .on_conflict(podcasts::id)
            .do_update()
            .set((
                podcasts::title.eq(diesel::dsl::sql::<diesel::sql_types::Text>(
                    "excluded.title",
                )),
                podcasts::author.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.author")),
                podcasts::description.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.description")),
                podcasts::artwork_url.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.artwork_url")),
                podcasts::feed_url.eq(diesel::dsl::sql::<diesel::sql_types::Text>(
                    "excluded.feed_url",
                )),
                podcasts::language.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.language")),
                podcasts::categories.eq(diesel::dsl::sql::<diesel::sql_types::Jsonb>(
                    "excluded.categories",
                )),
                podcasts::itunes_id.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::BigInt>,
                >("excluded.itunes_id")),
                podcasts::episode_count.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
                >("excluded.episode_count")),
                podcasts::latest_publish_time.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>,
                >("excluded.latest_publish_time")),
                podcasts::trend_score.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
                >("excluded.trend_score")),
                podcasts::podcast_guid.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.podcast_guid")),
                podcasts::updated_at.eq(Utc::now()),
                podcasts::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to batch upsert podcasts"))?;

        Ok(())
    }
}
