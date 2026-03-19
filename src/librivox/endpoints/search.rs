use std::collections::HashSet;

use crate::audiobook::Audiobook;
use crate::error::Result;
use crate::librivox::client::LibriVoxClient;
use crate::librivox::conversions::audiobook_from_book;
use crate::librivox::types::LibriVoxBooksResponse;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct LibriVoxParams {
    format: &'static str,
    extended: i32,
    coverart: i32,
    limit: i32,
    offset: i32,
}

impl LibriVoxParams {
    fn new(limit: i32, offset: i32) -> Self {
        Self {
            format: "json",
            extended: 1,
            coverart: 1,
            limit: limit.min(1000),
            offset,
        }
    }
}

#[derive(Debug, Serialize)]
struct TitleSearchParams {
    title: String,
    #[serde(flatten)]
    base: LibriVoxParams,
}

#[derive(Debug, Serialize)]
struct AuthorSearchParams {
    author: String,
    #[serde(flatten)]
    base: LibriVoxParams,
}

// Page listing with no search filter — reuses LibriVoxParams directly.

pub async fn search_audiobooks(
    client: &LibriVoxClient,
    query: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Audiobook>> {
    let prefixed = format!("^{query}");
    let base = LibriVoxParams::new(limit, offset);

    let title_params = TitleSearchParams {
        title: prefixed.clone(),
        base: base.clone(),
    };

    let author_params = AuthorSearchParams {
        author: prefixed,
        base: base.clone(),
    };

    let (title_res, author_res) = tokio::join!(
        client.get_with_query::<LibriVoxBooksResponse, _>("/audiobooks", &title_params),
        client.get_with_query::<LibriVoxBooksResponse, _>("/audiobooks", &author_params),
    );

    let title_books = title_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "LibriVox title search failed");
        LibriVoxBooksResponse { books: None }
    });
    let author_books = author_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "LibriVox author search failed");
        LibriVoxBooksResponse { books: None }
    });

    let mut seen = HashSet::new();
    let mut results: Vec<Audiobook> = Vec::new();

    let all_books = title_books
        .books
        .unwrap_or_default()
        .into_iter()
        .chain(author_books.books.unwrap_or_default());

    for book in all_books {
        if let Some(audiobook) = audiobook_from_book(book)
            && seen.insert(audiobook.id) {
                results.push(audiobook);
            }
    }

    results.truncate(base.limit as usize);
    Ok(results)
}

pub async fn get_audiobooks_page(
    client: &LibriVoxClient,
    limit: i32,
    offset: i32,
) -> Result<Vec<Audiobook>> {
    let params = LibriVoxParams::new(limit, offset);

    let response: LibriVoxBooksResponse = client
        .get_with_query("/audiobooks", &params)
        .await?;

    let books = response.books.unwrap_or_default();
    Ok(books.into_iter().filter_map(audiobook_from_book).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_params_defaults() {
        let params = LibriVoxParams::new(10, 0);
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["format"], "json");
        assert_eq!(json["extended"], 1);
        assert_eq!(json["coverart"], 1);
        assert_eq!(json["limit"], 10);
        assert_eq!(json["offset"], 0);
    }

    #[test]
    fn test_base_params_caps_limit() {
        let params = LibriVoxParams::new(5000, 0);
        assert_eq!(params.limit, 1000);
    }

    #[test]
    fn test_title_search_params_serialization() {
        let params = TitleSearchParams {
            title: "^pride".to_string(),
            base: LibriVoxParams::new(10, 0),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["title"], "^pride");
        assert_eq!(json["format"], "json");
        assert_eq!(json["limit"], 10);
    }

    #[test]
    fn test_author_search_params_serialization() {
        let params = AuthorSearchParams {
            author: "^hemingway".to_string(),
            base: LibriVoxParams::new(10, 0),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["author"], "^hemingway");
        assert!(json.get("title").is_none());
    }

    #[test]
    fn test_page_params_flattens_to_base() {
        let params = LibriVoxParams::new(20, 100);
        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["limit"], 20);
        assert_eq!(json["offset"], 100);
        assert!(json.get("title").is_none());
    }
}
