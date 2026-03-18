use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::audiobook::Audiobook;
use crate::db::models::{AudiobookRow, NewAudiobookRow};
use crate::db::schema::audiobooks;
use crate::db::{DbPool, db_error, get_conn, min_cached_at, validate_batch_size};
use crate::error::Result;

pub struct AudiobookRepository;

impl AudiobookRepository {
    pub async fn get_by_id(pool: &DbPool, id: i64) -> Result<Option<Audiobook>> {
        let mut conn = get_conn(pool).await?;

        let result = audiobooks::table
            .filter(audiobooks::id.eq(id))
            .select(AudiobookRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get audiobook"))?;

        Ok(result.map(Into::into))
    }

    pub async fn get_by_ids(pool: &DbPool, ids: &[i64]) -> Result<Vec<Audiobook>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }
        validate_batch_size(ids.len())?;

        let mut conn = get_conn(pool).await?;

        let results: Vec<AudiobookRow> = audiobooks::table
            .filter(audiobooks::id.eq_any(ids))
            .select(AudiobookRow::as_select())
            .load(&mut conn)
            .await
            .map_err(db_error("Failed to get audiobooks by IDs"))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn get_cached(
        pool: &DbPool,
        id: i64,
        ttl_seconds: i64,
    ) -> Result<Option<Audiobook>> {
        let min_cached_at = min_cached_at(ttl_seconds)?;
        let mut conn = get_conn(pool).await?;

        let result = audiobooks::table
            .filter(audiobooks::id.eq(id))
            .filter(audiobooks::cached_at.gt(min_cached_at))
            .select(AudiobookRow::as_select())
            .first(&mut conn)
            .await
            .optional()
            .map_err(db_error("Failed to get cached audiobook"))?;

        Ok(result.map(Into::into))
    }

    pub async fn upsert(pool: &DbPool, audiobook: &Audiobook) -> Result<()> {
        let mut conn = get_conn(pool).await?;
        let new_audiobook = NewAudiobookRow::from(audiobook);

        diesel::insert_into(audiobooks::table)
            .values(&new_audiobook)
            .on_conflict(audiobooks::id)
            .do_update()
            .set((
                audiobooks::title.eq(&new_audiobook.title),
                audiobooks::description.eq(&new_audiobook.description),
                audiobooks::language.eq(&new_audiobook.language),
                audiobooks::copyright_year.eq(&new_audiobook.copyright_year),
                audiobooks::num_sections.eq(&new_audiobook.num_sections),
                audiobooks::total_time.eq(&new_audiobook.total_time),
                audiobooks::total_time_secs.eq(&new_audiobook.total_time_secs),
                audiobooks::authors.eq(&new_audiobook.authors),
                audiobooks::url_text_source.eq(&new_audiobook.url_text_source),
                audiobooks::url_zip_file.eq(&new_audiobook.url_zip_file),
                audiobooks::url_librivox.eq(&new_audiobook.url_librivox),
                audiobooks::url_iarchive.eq(&new_audiobook.url_iarchive),
                audiobooks::coverart_url.eq(&new_audiobook.coverart_url),
                audiobooks::coverart_thumbnail.eq(&new_audiobook.coverart_thumbnail),
                audiobooks::updated_at.eq(Utc::now()),
                audiobooks::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to upsert audiobook"))?;

        Ok(())
    }

    pub async fn upsert_many(pool: &DbPool, audiobook_list: &[Audiobook]) -> Result<()> {
        if audiobook_list.is_empty() {
            return Ok(());
        }
        validate_batch_size(audiobook_list.len())?;

        let mut conn = get_conn(pool).await?;

        let new_audiobooks: Vec<NewAudiobookRow> =
            audiobook_list.iter().map(NewAudiobookRow::from).collect();

        diesel::insert_into(audiobooks::table)
            .values(&new_audiobooks)
            .on_conflict(audiobooks::id)
            .do_update()
            .set((
                audiobooks::title.eq(diesel::dsl::sql::<diesel::sql_types::Text>(
                    "excluded.title",
                )),
                audiobooks::description.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.description")),
                audiobooks::language.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.language")),
                audiobooks::copyright_year.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.copyright_year")),
                audiobooks::num_sections.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
                >("excluded.num_sections")),
                audiobooks::total_time.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.total_time")),
                audiobooks::total_time_secs.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::BigInt>,
                >("excluded.total_time_secs")),
                audiobooks::authors.eq(diesel::dsl::sql::<diesel::sql_types::Jsonb>(
                    "excluded.authors",
                )),
                audiobooks::url_text_source.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.url_text_source")),
                audiobooks::url_zip_file.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.url_zip_file")),
                audiobooks::url_librivox.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.url_librivox")),
                audiobooks::url_iarchive.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.url_iarchive")),
                audiobooks::coverart_url.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.coverart_url")),
                audiobooks::coverart_thumbnail.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.coverart_thumbnail")),
                audiobooks::updated_at.eq(Utc::now()),
                audiobooks::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to batch upsert audiobooks"))?;

        Ok(())
    }
}
