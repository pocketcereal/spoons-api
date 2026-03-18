use chrono::Utc;
use diesel::prelude::*;
use diesel_async::RunQueryDsl;

use crate::audiobook::Chapter;
use crate::db::models::{ChapterRow, NewChapterRow};
use crate::db::schema::chapters;
use crate::db::{DbPool, db_error, get_conn, min_cached_at, validate_batch_size};
use crate::error::Result;

pub struct ChapterRepository;

impl ChapterRepository {
    pub async fn get_by_audiobook_id(
        pool: &DbPool,
        audiobook_id: i64,
        limit: i32,
    ) -> Result<Vec<Chapter>> {
        let mut conn = get_conn(pool).await?;

        let results: Vec<ChapterRow> = chapters::table
            .filter(chapters::audiobook_id.eq(audiobook_id))
            .order(chapters::section_number.asc())
            .limit(limit.into())
            .select(ChapterRow::as_select())
            .load(&mut conn)
            .await
            .map_err(db_error("Failed to get chapters by audiobook ID"))?;

        Ok(results.into_iter().map(Into::into).collect())
    }

    pub async fn get_cached_by_audiobook_id(
        pool: &DbPool,
        audiobook_id: i64,
        limit: i32,
        ttl_seconds: i64,
    ) -> Result<Option<Vec<Chapter>>> {
        let min_cached_at = min_cached_at(ttl_seconds)?;
        let mut conn = get_conn(pool).await?;

        let results: Vec<ChapterRow> = chapters::table
            .filter(chapters::audiobook_id.eq(audiobook_id))
            .filter(chapters::cached_at.gt(min_cached_at))
            .order(chapters::section_number.asc())
            .limit(limit.into())
            .select(ChapterRow::as_select())
            .load(&mut conn)
            .await
            .map_err(db_error("Failed to get cached chapters by audiobook ID"))?;

        if results.is_empty() {
            return Ok(None);
        }

        Ok(Some(results.into_iter().map(Into::into).collect()))
    }

    pub async fn upsert(pool: &DbPool, chapter: &Chapter) -> Result<()> {
        let mut conn = get_conn(pool).await?;
        let new_chapter = NewChapterRow::from(chapter);

        diesel::insert_into(chapters::table)
            .values(&new_chapter)
            .on_conflict(chapters::id)
            .do_update()
            .set((
                chapters::audiobook_id.eq(&new_chapter.audiobook_id),
                chapters::title.eq(&new_chapter.title),
                chapters::section_number.eq(&new_chapter.section_number),
                chapters::duration.eq(&new_chapter.duration),
                chapters::duration_seconds.eq(&new_chapter.duration_seconds),
                chapters::listen_url.eq(&new_chapter.listen_url),
                chapters::language.eq(&new_chapter.language),
                chapters::readers.eq(&new_chapter.readers),
                chapters::updated_at.eq(Utc::now()),
                chapters::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to upsert chapter"))?;

        Ok(())
    }

    pub async fn upsert_many(pool: &DbPool, chapter_list: &[Chapter]) -> Result<()> {
        if chapter_list.is_empty() {
            return Ok(());
        }
        validate_batch_size(chapter_list.len())?;

        let mut conn = get_conn(pool).await?;

        let new_chapters: Vec<NewChapterRow> =
            chapter_list.iter().map(NewChapterRow::from).collect();

        diesel::insert_into(chapters::table)
            .values(&new_chapters)
            .on_conflict(chapters::id)
            .do_update()
            .set((
                chapters::audiobook_id.eq(diesel::dsl::sql::<diesel::sql_types::BigInt>(
                    "excluded.audiobook_id",
                )),
                chapters::title.eq(diesel::dsl::sql::<diesel::sql_types::Text>(
                    "excluded.title",
                )),
                chapters::section_number.eq(diesel::dsl::sql::<diesel::sql_types::Integer>(
                    "excluded.section_number",
                )),
                chapters::duration.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.duration")),
                chapters::duration_seconds.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Integer>,
                >("excluded.duration_seconds")),
                chapters::listen_url.eq(diesel::dsl::sql::<diesel::sql_types::Text>(
                    "excluded.listen_url",
                )),
                chapters::language.eq(diesel::dsl::sql::<
                    diesel::sql_types::Nullable<diesel::sql_types::Text>,
                >("excluded.language")),
                chapters::readers.eq(diesel::dsl::sql::<diesel::sql_types::Jsonb>(
                    "excluded.readers",
                )),
                chapters::updated_at.eq(Utc::now()),
                chapters::cached_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .await
            .map_err(db_error("Failed to batch upsert chapters"))?;

        Ok(())
    }
}
