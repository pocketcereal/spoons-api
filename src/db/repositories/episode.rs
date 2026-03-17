use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::db::models::{EpisodeRow, NewEpisodeRow};
use crate::db::schema::episodes;
use crate::db::{DbPool, db_error, get_conn, min_cached_at, validate_batch_size};
use crate::error::Result;
use crate::podcast::Episode;

pub struct EpisodeRepository;

impl EpisodeRepository {
    pub async fn get_by_id(pool: &DbPool, id: i64) -> Result<Option<Episode>> {
        let mut conn = get_conn(pool).await?;

        let result = episodes::table
            .filter(episodes::id.eq(id))
            .select(EpisodeRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get episode"))?;

        Ok(result.map(Into::into))
    }

    pub async fn get_by_podcast_id(
        pool: &DbPool,
        podcast_id: i64,
        limit: i32,
    ) -> Result<Vec<Episode>> {
        let mut conn = get_conn(pool).await?;

        let results: Vec<EpisodeRow> = episodes::table
            .filter(episodes::podcast_id.eq(podcast_id))
            .order(episodes::published_at.desc().nulls_last())
            .limit(limit.into())
            .select(EpisodeRow::as_select())
            .load(&mut conn)
            .await
            .map_err(db_error("Failed to get episodes by podcast ID"))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn get_cached_by_podcast_id(
        pool: &DbPool,
        podcast_id: i64,
        limit: i32,
        ttl_seconds: i64,
    ) -> Result<Option<Vec<Episode>>> {
        let min_cached_at = min_cached_at(ttl_seconds)?;
        let mut conn = get_conn(pool).await?;

        let results: Vec<EpisodeRow> = episodes::table
            .filter(episodes::podcast_id.eq(podcast_id))
            .filter(episodes::cached_at.gt(min_cached_at))
            .order(episodes::published_at.desc().nulls_last())
            .limit(limit.into())
            .select(EpisodeRow::as_select())
            .load(&mut conn)
            .await
            .map_err(db_error("Failed to get cached episodes by podcast ID"))?;

        if results.is_empty() {
            return Ok(None);
        }

        Ok(Some(results.into_iter().map(Into::into).collect()))
    }

    pub async fn get_cached(pool: &DbPool, id: i64, ttl_seconds: i64) -> Result<Option<Episode>> {
        let min_cached_at = min_cached_at(ttl_seconds)?;
        let mut conn = get_conn(pool).await?;

        let result = episodes::table
            .filter(episodes::id.eq(id))
            .filter(episodes::cached_at.gt(min_cached_at))
            .select(EpisodeRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get cached episode"))?;

        Ok(result.map(Into::into))
    }

    pub async fn upsert(pool: &DbPool, episode: &Episode) -> Result<()> {
        let mut conn = get_conn(pool).await?;
        let new_episode = NewEpisodeRow::from(episode);

        diesel::insert_into(episodes::table)
            .values(&new_episode)
            .on_conflict(episodes::id)
            .do_update()
            .set((
                episodes::podcast_id.eq(&new_episode.podcast_id),
                episodes::title.eq(&new_episode.title),
                episodes::description.eq(&new_episode.description),
                episodes::audio_url.eq(&new_episode.audio_url),
                episodes::audio_type.eq(&new_episode.audio_type),
                episodes::audio_length.eq(&new_episode.audio_length),
                episodes::duration_seconds.eq(&new_episode.duration_seconds),
                episodes::published_at.eq(&new_episode.published_at),
                episodes::episode_number.eq(&new_episode.episode_number),
                episodes::season_number.eq(&new_episode.season_number),
                episodes::episode_type.eq(&new_episode.episode_type),
                episodes::image_url.eq(&new_episode.image_url),
                episodes::explicit.eq(&new_episode.explicit),
                episodes::updated_at.eq(Utc::now()),
                episodes::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to upsert episode"))?;

        Ok(())
    }

    pub async fn upsert_many(pool: &DbPool, episode_list: &[Episode]) -> Result<()> {
        if episode_list.is_empty() {
            return Ok(());
        }
        validate_batch_size(episode_list.len())?;

        let mut conn = get_conn(pool).await?;

        let new_episodes: Vec<NewEpisodeRow> =
            episode_list.iter().map(NewEpisodeRow::from).collect();

        diesel::insert_into(episodes::table)
            .values(&new_episodes)
            .on_conflict(episodes::id)
            .do_update()
            .set((
                episodes::podcast_id.eq(diesel::dsl::sql::<diesel::sql_types::BigInt>(
                    "excluded.podcast_id",
                )),
                episodes::title.eq(diesel::dsl::sql::<diesel::sql_types::Text>(
                    "excluded.title",
                )),
                episodes::description.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.description")),
                episodes::audio_url.eq(diesel::dsl::sql::<diesel::sql_types::Text>(
                    "excluded.audio_url",
                )),
                episodes::audio_type.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.audio_type")),
                episodes::audio_length.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::BigInt>,
                >("excluded.audio_length")),
                episodes::duration_seconds.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
                >("excluded.duration_seconds")),
                episodes::published_at.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>,
                >("excluded.published_at")),
                episodes::episode_number.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
                >("excluded.episode_number")),
                episodes::season_number.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
                >("excluded.season_number")),
                episodes::episode_type.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.episode_type")),
                episodes::image_url.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.image_url")),
                episodes::explicit.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Bool>,
                >("excluded.explicit")),
                episodes::updated_at.eq(Utc::now()),
                episodes::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to batch upsert episodes"))?;

        Ok(())
    }
}
