use crate::audiobook::Audiobook;
use crate::error::Result;
use crate::librivox::client::LibriVoxClient;
use crate::librivox::conversions::audiobook_from_book;
use crate::librivox::types::LibriVoxBooksResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct SearchParams {
    title: String,
    format: &'static str,
    extended: i32,
    coverart: i32,
    limit: i32,
    offset: i32,
}

#[derive(Debug, Serialize)]
struct PageParams {
    format: &'static str,
    extended: i32,
    coverart: i32,
    limit: i32,
    offset: i32,
}

pub async fn search_audiobooks(
    client: &LibriVoxClient,
    title: &str,
    limit: i32,
    offset: i32,
) -> Result<Vec<Audiobook>> {
    let params = SearchParams {
        title: title.to_string(),
        format: "json",
        extended: 1,
        coverart: 1,
        limit: limit.min(1000),
        offset,
    };

    let response: LibriVoxBooksResponse = client
        .get_with_query("/audiobooks", &params)
        .await?;

    let books = response.books.unwrap_or_default();
    Ok(books.into_iter().filter_map(audiobook_from_book).collect())
}

pub async fn get_audiobooks_page(
    client: &LibriVoxClient,
    limit: i32,
    offset: i32,
) -> Result<Vec<Audiobook>> {
    let params = PageParams {
        format: "json",
        extended: 1,
        coverart: 1,
        limit: limit.min(1000),
        offset,
    };

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
    fn test_search_params_serialization() {
        let params = SearchParams {
            title: "pride".to_string(),
            format: "json",
            extended: 1,
            coverart: 1,
            limit: 10,
            offset: 0,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["title"], "pride");
        assert_eq!(json["format"], "json");
        assert_eq!(json["extended"], 1);
        assert_eq!(json["coverart"], 1);
        assert_eq!(json["limit"], 10);
        assert_eq!(json["offset"], 0);
    }

    #[test]
    fn test_page_params_serialization() {
        let params = PageParams {
            format: "json",
            extended: 1,
            coverart: 1,
            limit: 20,
            offset: 100,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["limit"], 20);
        assert_eq!(json["offset"], 100);
        assert!(json.get("title").is_none());
    }
}
