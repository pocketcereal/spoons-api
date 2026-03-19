use crate::error::Result;
use futures::future::join_all;
use std::sync::Arc;
use std::time::Duration;

pub const SOURCE_TIMEOUT: Duration = Duration::from_secs(10);

/// Dispatches a search to multiple sources in parallel, combining results.
/// Failures and timeouts are logged but don't fail the overall query.
pub async fn fan_out_search<S, T, F, Fut>(
    sources: &[Arc<S>],
    timeout_duration: Duration,
    search_fn: F,
) -> Vec<T>
where
    S: ?Sized + Send + Sync,
    T: Send + 'static,
    F: Fn(Arc<S>) -> Fut + Send,
    Fut: std::future::Future<Output = Result<Vec<T>>> + Send,
{
    let futures = sources.iter().map(|s| {
        let source = Arc::clone(s);
        let fut = search_fn(source);
        async move {
            match tokio::time::timeout(timeout_duration, fut).await {
                Ok(Ok(items)) => items,
                Ok(Err(e)) => {
                    tracing::warn!(error = %e, "Source query failed");
                    vec![]
                }
                Err(_) => {
                    tracing::warn!("Source query timed out");
                    vec![]
                }
            }
        }
    });

    join_all(futures).await.into_iter().flatten().collect()
}

/// Dispatches a single-entity lookup to multiple sources, returning the first Some result.
pub async fn fan_out_single<S, T, F, Fut>(
    sources: &[Arc<S>],
    timeout_duration: Duration,
    search_fn: F,
) -> Result<Option<T>>
where
    S: ?Sized + Send + Sync,
    T: Send + 'static,
    F: Fn(Arc<S>) -> Fut + Send,
    Fut: std::future::Future<Output = Result<Option<T>>> + Send,
{
    let futures = sources.iter().map(|s| {
        let source = Arc::clone(s);
        let fut = search_fn(source);
        tokio::time::timeout(timeout_duration, fut)
    });

    for result in join_all(futures).await {
        match result {
            Ok(Ok(Some(item))) => return Ok(Some(item)),
            Ok(Ok(None)) => continue,
            Ok(Err(e)) => tracing::warn!(error = %e, "Source lookup failed"),
            Err(_) => tracing::warn!("Source lookup timed out"),
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::AppError;
    use std::time::Duration;

    struct Source {
        items: Vec<String>,
        fail: bool,
        slow: bool,
    }

    impl Source {
        fn ok(items: &[&str]) -> Arc<Self> {
            Arc::new(Self {
                items: items.iter().map(|s| s.to_string()).collect(),
                fail: false,
                slow: false,
            })
        }
        fn err() -> Arc<Self> {
            Arc::new(Self {
                items: vec![],
                fail: true,
                slow: false,
            })
        }
        fn slow() -> Arc<Self> {
            Arc::new(Self {
                items: vec![],
                fail: false,
                slow: true,
            })
        }
        fn none() -> Arc<Self> {
            Arc::new(Self {
                items: vec![],
                fail: false,
                slow: false,
            })
        }
    }

    async fn do_search(s: Arc<Source>) -> Result<Vec<String>> {
        if s.slow {
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
        if s.fail {
            return Err(AppError::Internal(anyhow::anyhow!("source error")));
        }
        Ok(s.items.clone())
    }

    async fn do_lookup(s: Arc<Source>) -> Result<Option<String>> {
        if s.slow {
            tokio::time::sleep(Duration::from_secs(30)).await;
        }
        if s.fail {
            return Err(AppError::Internal(anyhow::anyhow!("source error")));
        }
        Ok(s.items.first().cloned())
    }

    #[tokio::test]
    async fn fan_out_search_combines_results_from_multiple_sources() {
        let sources = vec![Source::ok(&["a", "b"]), Source::ok(&["c"])];
        let timeout = Duration::from_secs(5);

        let results = fan_out_search(&sources, timeout, do_search).await;

        assert_eq!(results.len(), 3);
        assert!(results.contains(&"a".to_string()));
        assert!(results.contains(&"b".to_string()));
        assert!(results.contains(&"c".to_string()));
    }

    #[tokio::test]
    async fn fan_out_search_returns_results_from_successful_sources_when_one_fails() {
        let sources = vec![Source::ok(&["ok"]), Source::err()];
        let timeout = Duration::from_secs(5);

        let results = fan_out_search(&sources, timeout, do_search).await;

        assert_eq!(results, vec!["ok".to_string()]);
    }

    #[tokio::test]
    async fn fan_out_search_handles_timeouts_gracefully() {
        let sources = vec![Source::ok(&["fast"]), Source::slow()];
        let timeout = Duration::from_millis(50);

        let results = fan_out_search(&sources, timeout, do_search).await;

        assert_eq!(results, vec!["fast".to_string()]);
    }

    #[tokio::test]
    async fn fan_out_single_returns_first_some_result() {
        let sources = vec![
            Source::none(),
            Source::ok(&["found"]),
            Source::ok(&["second"]),
        ];
        let timeout = Duration::from_secs(5);

        let result = fan_out_single(&sources, timeout, do_lookup).await;

        assert!(matches!(result, Ok(Some(ref v)) if v == "found"));
    }

    #[tokio::test]
    async fn fan_out_single_returns_none_when_all_sources_return_none() {
        let sources = vec![Source::none(), Source::none()];
        let timeout = Duration::from_secs(5);

        let result = fan_out_single(&sources, timeout, do_lookup).await;

        assert!(matches!(result, Ok(None)));
    }
}
