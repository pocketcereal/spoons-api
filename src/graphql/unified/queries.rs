use std::sync::Arc;
use std::time::Duration;

use async_graphql::{Context, Object, Result};

use crate::error::AppError;
use crate::graphql::audiobook::queries::random_librivox_audiobooks;
use crate::graphql::schema::{
    random_audius_artists, random_audius_tracks, random_musicbrainz_artists,
    random_musicbrainz_tracks, search_audius_artists, search_audius_tracks, search_sources,
    search_musicbrainz_artists, search_musicbrainz_tracks, AppContext,
};
use crate::graphql::{clamp_limit, get_app_context, validate_query};

use super::types::*;

const DOMAIN_TIMEOUT: Duration = Duration::from_secs(10);

fn gql_to_app_error(e: async_graphql::Error) -> AppError {
    let detail = e
        .source
        .as_ref()
        .and_then(|s| s.downcast_ref::<AppError>())
        .map(|s| format!(": {}", s));
    AppError::Internal(anyhow::anyhow!("{}{}", e.message, detail.unwrap_or_default()))
}

fn resolve_domains(domains: Option<Vec<ContentDomain>>) -> Vec<ContentDomain> {
    domains.unwrap_or_else(|| {
        vec![
            ContentDomain::Music,
            ContentDomain::Podcasts,
            ContentDomain::Audiobooks,
        ]
    })
}

fn set_or_warn<T>(
    field: &mut Option<T>,
    result: std::result::Result<Option<T>, AppError>,
    domain: &str,
) {
    match result {
        Ok(val) => *field = val,
        Err(e) => tracing::warn!(domain = domain, error = %e, "Domain query failed"),
    }
}

#[derive(Default)]
pub struct UnifiedQuery;

#[Object]
impl UnifiedQuery {
    async fn search(
        &self,
        ctx: &Context<'_>,
        query: String,
        domains: Option<Vec<ContentDomain>>,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<SearchResults> {
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let domains = resolve_domains(domains);

        let (music, podcasts, audiobooks) = tokio::join!(
            search_music(app_ctx, &domains, &query, limit),
            search_podcasts(app_ctx, &domains, &query, limit),
            search_audiobooks(app_ctx, &domains, &query, limit),
        );

        let mut results = SearchResults::default();
        set_or_warn(&mut results.music, music, "MUSIC");
        set_or_warn(&mut results.podcasts, podcasts, "PODCASTS");
        set_or_warn(&mut results.audiobooks, audiobooks, "AUDIOBOOKS");

        Ok(results)
    }

    async fn random(
        &self,
        ctx: &Context<'_>,
        domains: Option<Vec<ContentDomain>>,
        #[graphql(default = 10)] limit: i32,
    ) -> Result<RandomResults> {
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let domains = resolve_domains(domains);

        let (music, podcasts, audiobooks) = tokio::join!(
            random_music(app_ctx, &domains, limit),
            random_podcasts(app_ctx, &domains, limit),
            random_audiobooks(app_ctx, &domains, limit),
        );

        let mut results = RandomResults::default();
        set_or_warn(&mut results.music, music, "MUSIC");
        set_or_warn(&mut results.podcasts, podcasts, "PODCASTS");
        set_or_warn(&mut results.audiobooks, audiobooks, "AUDIOBOOKS");

        Ok(results)
    }
}

// ==================== Search Helpers ====================

async fn search_music(
    app_ctx: &Arc<AppContext>,
    domains: &[ContentDomain],
    query: &str,
    limit: i32,
) -> std::result::Result<Option<MusicSearchResults>, AppError> {
    if !domains.contains(&ContentDomain::Music) {
        return Ok(None);
    }

    // No outer timeout — search_sources already wraps each sub-source
    // (MusicBrainz, Audius) in SOURCE_QUERY_TIMEOUT individually.
    let (artists, tracks) = tokio::join!(
        search_sources(
            None,
            search_musicbrainz_artists(app_ctx, query, limit, 0),
            search_audius_artists(app_ctx, query, limit, 0),
            "artist",
        ),
        search_sources(
            None,
            search_musicbrainz_tracks(app_ctx, query, limit, 0),
            search_audius_tracks(app_ctx, query, limit, 0),
            "track",
        ),
    );

    Ok(Some(MusicSearchResults {
        artists: artists.map_err(gql_to_app_error)?,
        tracks: tracks.map_err(gql_to_app_error)?,
    }))
}

async fn search_podcasts(
    app_ctx: &Arc<AppContext>,
    domains: &[ContentDomain],
    query: &str,
    limit: i32,
) -> std::result::Result<Option<PodcastSearchResults>, AppError> {
    if !domains.contains(&ContentDomain::Podcasts) {
        return Ok(None);
    }

    let service = match app_ctx.podcast.as_ref() {
        Some(s) => s,
        None => return Ok(None),
    };

    let result = tokio::time::timeout(DOMAIN_TIMEOUT, async {
        let podcasts = service
            .search_podcasts(query, limit)
            .await?
            .into_iter()
            .map(crate::graphql::podcast::Podcast::from)
            .collect();
        Ok::<_, AppError>(PodcastSearchResults { podcasts })
    })
    .await;

    match result {
        Ok(Ok(r)) => Ok(Some(r)),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AppError::Internal(anyhow::anyhow!("Podcast search timed out"))),
    }
}

async fn search_audiobooks(
    app_ctx: &Arc<AppContext>,
    domains: &[ContentDomain],
    query: &str,
    limit: i32,
) -> std::result::Result<Option<AudiobookSearchResults>, AppError> {
    if !domains.contains(&ContentDomain::Audiobooks) {
        return Ok(None);
    }

    let service = match app_ctx.audiobook.as_ref() {
        Some(s) => s,
        None => return Ok(None),
    };

    let result = tokio::time::timeout(DOMAIN_TIMEOUT, async {
        let audiobooks = service
            .search_audiobooks(query, limit, 0)
            .await?
            .into_iter()
            .map(crate::graphql::audiobook::Audiobook::from)
            .collect();
        Ok::<_, AppError>(AudiobookSearchResults { audiobooks })
    })
    .await;

    match result {
        Ok(Ok(r)) => Ok(Some(r)),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AppError::Internal(anyhow::anyhow!("Audiobook search timed out"))),
    }
}

// ==================== Random Helpers ====================

async fn random_music(
    app_ctx: &Arc<AppContext>,
    domains: &[ContentDomain],
    limit: i32,
) -> std::result::Result<Option<MusicRandomResults>, AppError> {
    if !domains.contains(&ContentDomain::Music) {
        return Ok(None);
    }

    // No outer timeout — search_sources already wraps each sub-source individually.
    let (artists, tracks) = tokio::join!(
        search_sources(
            None,
            random_musicbrainz_artists(app_ctx, limit),
            random_audius_artists(app_ctx, limit),
            "random artist",
        ),
        search_sources(
            None,
            random_musicbrainz_tracks(app_ctx, limit),
            random_audius_tracks(app_ctx, limit),
            "random track",
        ),
    );

    Ok(Some(MusicRandomResults {
        artists: artists.map_err(gql_to_app_error)?,
        tracks: tracks.map_err(gql_to_app_error)?,
    }))
}

async fn random_podcasts(
    app_ctx: &Arc<AppContext>,
    domains: &[ContentDomain],
    limit: i32,
) -> std::result::Result<Option<PodcastRandomResults>, AppError> {
    if !domains.contains(&ContentDomain::Podcasts) {
        return Ok(None);
    }

    let service = match app_ctx.podcast.as_ref() {
        Some(s) => s,
        None => return Ok(None),
    };

    let result = tokio::time::timeout(DOMAIN_TIMEOUT, async {
        let episodes = service
            .client()
            .random_episodes(limit, None, None)
            .await?
            .into_iter()
            .map(crate::graphql::podcast::Episode::from)
            .collect();
        Ok::<_, AppError>(PodcastRandomResults { episodes })
    })
    .await;

    match result {
        Ok(Ok(r)) => Ok(Some(r)),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AppError::Internal(anyhow::anyhow!("Podcast random timed out"))),
    }
}

async fn random_audiobooks(
    app_ctx: &Arc<AppContext>,
    domains: &[ContentDomain],
    limit: i32,
) -> std::result::Result<Option<AudiobookRandomResults>, AppError> {
    if !domains.contains(&ContentDomain::Audiobooks) {
        return Ok(None);
    }

    let service = match app_ctx.audiobook.as_ref() {
        Some(s) => s,
        None => return Ok(None),
    };

    let result = tokio::time::timeout(DOMAIN_TIMEOUT, async {
        let audiobooks = random_librivox_audiobooks(service, limit)
            .await?
            .into_iter()
            .map(crate::graphql::audiobook::Audiobook::from)
            .collect();
        Ok::<_, AppError>(AudiobookRandomResults { audiobooks })
    })
    .await;

    match result {
        Ok(Ok(r)) => Ok(Some(r)),
        Ok(Err(e)) => Err(e),
        Err(_) => Err(AppError::Internal(anyhow::anyhow!("Audiobook random timed out"))),
    }
}
