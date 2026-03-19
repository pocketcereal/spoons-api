use std::sync::Arc;

use async_graphql::{Context, Object, Result};

use crate::domain::{AudiobookProvider, DataSource, MusicProvider, PodcastProvider};
use crate::error::AppError;
use crate::graphql::{clamp_limit, filter_music_providers, get_app_context, validate_query};
use crate::sources::{fan_out_search, SOURCE_TIMEOUT};

use super::types::*;

fn resolve_domains(domains: Option<Vec<ContentDomain>>) -> Vec<ContentDomain> {
    domains.unwrap_or_else(|| {
        vec![
            ContentDomain::Music,
            ContentDomain::Podcasts,
            ContentDomain::Audiobooks,
        ]
    })
}

fn domain_providers<'a, T: ?Sized>(
    providers: &'a [Arc<T>],
    domains: &[ContentDomain],
    domain: ContentDomain,
) -> &'a [Arc<T>] {
    if domains.contains(&domain) {
        providers
    } else {
        &[]
    }
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
        music_sources: Option<Vec<DataSource>>,
        #[graphql(default = 20)] limit: i32,
    ) -> Result<SearchResults> {
        let query = validate_query(&query)?;
        let limit = clamp_limit(limit);
        let app_ctx = get_app_context(ctx)?;
        let domains = resolve_domains(domains);

        let music_providers = filter_music_providers(
            domain_providers(&app_ctx.music_providers, &domains, ContentDomain::Music),
            music_sources.as_deref(),
        );

        let (music, podcasts, audiobooks) = tokio::join!(
            search_music(&music_providers, &query, limit),
            search_podcasts(domain_providers(&app_ctx.podcast_providers, &domains, ContentDomain::Podcasts), &query, limit),
            search_audiobooks(domain_providers(&app_ctx.audiobook_providers, &domains, ContentDomain::Audiobooks), &query, limit),
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
            random_music(domain_providers(&app_ctx.music_providers, &domains, ContentDomain::Music), limit),
            random_podcasts(domain_providers(&app_ctx.podcast_providers, &domains, ContentDomain::Podcasts), limit),
            random_audiobooks(domain_providers(&app_ctx.audiobook_providers, &domains, ContentDomain::Audiobooks), limit),
        );

        let mut results = RandomResults::default();
        set_or_warn(&mut results.music, music, "MUSIC");
        set_or_warn(&mut results.podcasts, podcasts, "PODCASTS");
        set_or_warn(&mut results.audiobooks, audiobooks, "AUDIOBOOKS");
        Ok(results)
    }
}

async fn search_music(
    providers: &[Arc<dyn MusicProvider>],
    query: &str,
    limit: i32,
) -> std::result::Result<Option<MusicSearchResults>, AppError> {
    if providers.is_empty() {
        return Ok(None);
    }
    let (artists, tracks) = tokio::join!(
        fan_out_search(providers, SOURCE_TIMEOUT, |p| {
            let q = query.to_string();
            async move { p.search_artists(&q, limit, 0).await }
        }),
        fan_out_search(providers, SOURCE_TIMEOUT, |p| {
            let q = query.to_string();
            async move { p.search_tracks(&q, limit, 0).await }
        }),
    );
    Ok(Some(MusicSearchResults { artists, tracks }))
}

async fn search_podcasts(
    providers: &[Arc<dyn PodcastProvider>],
    query: &str,
    limit: i32,
) -> std::result::Result<Option<PodcastSearchResults>, AppError> {
    if providers.is_empty() {
        return Ok(None);
    }
    let podcasts = fan_out_search(providers, SOURCE_TIMEOUT, |p| {
        let q = query.to_string();
        async move { p.search_podcasts(&q, limit).await }
    })
    .await;
    Ok(Some(PodcastSearchResults { podcasts }))
}

async fn search_audiobooks(
    providers: &[Arc<dyn AudiobookProvider>],
    query: &str,
    limit: i32,
) -> std::result::Result<Option<AudiobookSearchResults>, AppError> {
    if providers.is_empty() {
        return Ok(None);
    }
    let audiobooks = fan_out_search(providers, SOURCE_TIMEOUT, |p| {
        let q = query.to_string();
        async move { p.search_audiobooks(&q, limit, 0).await }
    })
    .await;
    Ok(Some(AudiobookSearchResults { audiobooks }))
}

async fn random_music(
    providers: &[Arc<dyn MusicProvider>],
    limit: i32,
) -> std::result::Result<Option<MusicRandomResults>, AppError> {
    if providers.is_empty() {
        return Ok(None);
    }
    let (artists, tracks) = tokio::join!(
        fan_out_search(providers, SOURCE_TIMEOUT, |p| {
            async move { p.random_artists(limit).await }
        }),
        fan_out_search(providers, SOURCE_TIMEOUT, |p| {
            async move { p.random_tracks(limit).await }
        }),
    );
    Ok(Some(MusicRandomResults { artists, tracks }))
}

async fn random_podcasts(
    providers: &[Arc<dyn PodcastProvider>],
    limit: i32,
) -> std::result::Result<Option<PodcastRandomResults>, AppError> {
    if providers.is_empty() {
        return Ok(None);
    }
    let episodes = fan_out_search(providers, SOURCE_TIMEOUT, |p| {
        async move { p.random_episodes(limit, None, None).await }
    })
    .await;
    Ok(Some(PodcastRandomResults { episodes }))
}

async fn random_audiobooks(
    providers: &[Arc<dyn AudiobookProvider>],
    limit: i32,
) -> std::result::Result<Option<AudiobookRandomResults>, AppError> {
    if providers.is_empty() {
        return Ok(None);
    }
    let audiobooks = fan_out_search(providers, SOURCE_TIMEOUT, |p| {
        async move { p.random_audiobooks(limit).await }
    })
    .await;
    Ok(Some(AudiobookRandomResults { audiobooks }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_providers_returns_providers_when_domain_present() {
        let providers: Vec<Arc<String>> = vec![Arc::new("a".into())];
        let domains = vec![ContentDomain::Music];
        let result = domain_providers(&providers, &domains, ContentDomain::Music);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_domain_providers_returns_empty_when_domain_absent() {
        let providers: Vec<Arc<String>> = vec![Arc::new("a".into())];
        let domains = vec![ContentDomain::Podcasts];
        let result = domain_providers(&providers, &domains, ContentDomain::Music);
        assert!(result.is_empty());
    }

    #[test]
    fn test_resolve_domains_none_returns_all() {
        let domains = resolve_domains(None);
        assert_eq!(
            domains,
            vec![
                ContentDomain::Music,
                ContentDomain::Podcasts,
                ContentDomain::Audiobooks
            ]
        );
    }

    #[test]
    fn test_resolve_domains_some_returns_specified() {
        let domains = resolve_domains(Some(vec![ContentDomain::Music]));
        assert_eq!(domains, vec![ContentDomain::Music]);
    }

    #[test]
    fn test_resolve_domains_empty_vec_returns_empty() {
        let domains = resolve_domains(Some(vec![]));
        assert!(domains.is_empty());
    }

    #[test]
    fn test_set_or_warn_ok_some_sets_value() {
        let mut field: Option<i32> = None;
        set_or_warn(&mut field, Ok(Some(42)), "TEST");
        assert_eq!(field, Some(42));
    }

    #[test]
    fn test_set_or_warn_ok_none_clears_value() {
        let mut field: Option<i32> = Some(42);
        set_or_warn(&mut field, Ok(None), "TEST");
        assert_eq!(field, None);
    }

    #[test]
    fn test_set_or_warn_err_leaves_field_none() {
        let mut field: Option<i32> = None;
        let err = AppError::Internal(anyhow::anyhow!("boom"));
        set_or_warn(&mut field, Err(err), "TEST");
        assert_eq!(field, None);
    }
}
