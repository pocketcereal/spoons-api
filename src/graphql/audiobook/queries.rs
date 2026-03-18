use async_graphql::{Context, ErrorExtensions, Object, Result};

use crate::audiobook::AudiobookSource as DomainAudiobookSource;
use crate::domain::AudiobookProvider;
use crate::error::AppError;
use crate::graphql::schema::AppContext;
use crate::graphql::{clamp_limit, get_app_context, validate_id, validate_query};

use super::{Audiobook, Chapter};

fn require_audiobook_provider(
    app_ctx: &std::sync::Arc<AppContext>,
) -> Result<&std::sync::Arc<dyn AudiobookProvider>> {
    app_ctx.audiobook_providers.first().ok_or_else(|| {
        AppError::FeatureDisabled(
            "LibriVox is not configured. Set librivox in config.yaml.".into(),
        )
        .extend()
    })
}

#[derive(Default)]
pub struct AudiobookQuery;

#[Object]
impl AudiobookQuery {
    async fn search_audiobooks(
        &self,
        ctx: &Context<'_>,
        query: String,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<Vec<Audiobook>> {
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let provider = require_audiobook_provider(app_ctx)?;
        provider
            .search_audiobooks(&query, limit, 0)
            .await
            .map_err(|e| e.extend())
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
                let provider = require_audiobook_provider(app_ctx)?;
                provider
                    .get_audiobook(audiobook_id)
                    .await
                    .map_err(|e| e.extend())
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
                let provider = require_audiobook_provider(app_ctx)?;
                provider
                    .get_chapters(raw_id, limit)
                    .await
                    .map_err(|e| e.extend())
            }
        }
    }

    async fn random_audiobooks(
        &self,
        ctx: &Context<'_>,
        #[graphql(default = 10)] limit: i32,
    ) -> Result<Vec<Audiobook>> {
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let provider = require_audiobook_provider(app_ctx)?;
        provider
            .random_audiobooks(limit)
            .await
            .map_err(|e| e.extend())
    }
}
