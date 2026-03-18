use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::audiobook::AudiobookSource as DomainAudiobookSource;
use crate::error::AppError;
use crate::graphql::helpers::random_sample;
use crate::graphql::{
    clamp_limit, get_app_context, require_audiobook_service, validate_id, validate_query,
};

use super::audiobook_types::AudiobookSource;
use super::{Audiobook, Chapter};

#[derive(Default)]
pub struct AudiobookQuery;

#[Object]
impl AudiobookQuery {
    async fn search_audiobooks(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 20)] limit: i32,
        source: Option<AudiobookSource>,
    ) -> Result<Vec<Audiobook>> {
        let _ = source;
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let service = require_audiobook_service(app_ctx)?;

        let results = service
            .search_audiobooks(&query, limit, 0)
            .await
            .map_err(|e| e.extend())?;

        Ok(results.into_iter().map(Audiobook::from).collect())
    }

    async fn audiobook(&self, ctx: &Context<'_>, id: String) -> Result<Option<Audiobook>> {
        let id = validate_id(&id)?;
        let app_ctx = get_app_context(ctx)?;

        let (source, audiobook_id) =
            DomainAudiobookSource::parse_id(&id).ok_or_else(|| {
                AppError::InvalidInput(format!(
                    "Invalid audiobook ID format: {}",
                    id.chars().take(50).collect::<String>()
                ))
                .extend()
            })?;

        match source {
            DomainAudiobookSource::LibriVox => {
                let service = require_audiobook_service(app_ctx)?;
                let audiobook = service
                    .get_audiobook(audiobook_id)
                    .await
                    .map_err(|e| e.extend())?;
                Ok(audiobook.map(Audiobook::from))
            }
        }
    }

    async fn chapters(
        &self,
        ctx: &Context<'_>,
        audiobook_id: String,
        #[graphql(default = 100)] limit: i32,
    ) -> Result<Vec<Chapter>> {
        let audiobook_id_str = validate_id(&audiobook_id)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;

        let (source, raw_id) =
            DomainAudiobookSource::parse_id(&audiobook_id_str).ok_or_else(|| {
                AppError::InvalidInput(format!(
                    "Invalid audiobook ID format: {}",
                    audiobook_id_str.chars().take(50).collect::<String>()
                ))
                .extend()
            })?;

        match source {
            DomainAudiobookSource::LibriVox => {
                let service = require_audiobook_service(app_ctx)?;
                let chapters = service
                    .get_chapters(raw_id, limit)
                    .await
                    .map_err(|e| e.extend())?;

                Ok(chapters.into_iter().map(Chapter::from).collect())
            }
        }
    }

    async fn random_audiobooks(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i32,
        source: Option<AudiobookSource>,
    ) -> Result<Vec<Audiobook>> {
        let _ = source;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let service = require_audiobook_service(app_ctx)?;

        let audiobooks = random_librivox_audiobooks(service, limit)
            .await
            .map_err(|e| e.extend())?;

        Ok(audiobooks.into_iter().map(Audiobook::from).collect())
    }
}

const LIBRIVOX_MAX_OFFSET: i64 = 20_000;
const RANDOM_RETRY_ATTEMPTS: u32 = 3;

async fn random_librivox_audiobooks(
    service: &crate::services::AudiobookService,
    limit: i32,
) -> std::result::Result<Vec<crate::audiobook::Audiobook>, AppError> {
    let fetch_limit = limit * 2;
    let mut offset = rand::Rng::gen_range(&mut rand::thread_rng(), 0..LIBRIVOX_MAX_OFFSET as i32);

    for _ in 0..RANDOM_RETRY_ATTEMPTS {
        let results = service
            .get_audiobooks_page(fetch_limit, offset)
            .await?;

        if !results.is_empty() {
            return Ok(random_sample(results, limit as usize));
        }

        offset /= 2;
    }

    Ok(Vec::new())
}
