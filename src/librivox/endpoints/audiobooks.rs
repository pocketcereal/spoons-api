use crate::audiobook::Audiobook;
use crate::error::Result;
use crate::librivox::client::LibriVoxClient;
use crate::librivox::conversions::audiobook_from_book;
use crate::librivox::types::LibriVoxBooksResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct AudiobookByIdParams {
    id: i64,
    format: &'static str,
    extended: i32,
    coverart: i32,
}

pub async fn get_audiobook_by_id(
    client: &LibriVoxClient,
    id: i64,
) -> Result<Option<Audiobook>> {
    let params = AudiobookByIdParams {
        id,
        format: "json",
        extended: 1,
        coverart: 1,
    };

    let response: LibriVoxBooksResponse = client
        .get_with_query("/audiobooks", &params)
        .await?;

    let books = response.books.unwrap_or_default();
    Ok(books.into_iter().next().and_then(audiobook_from_book))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audiobook_by_id_params_serialization() {
        let params = AudiobookByIdParams {
            id: 128,
            format: "json",
            extended: 1,
            coverart: 1,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["id"], 128);
        assert_eq!(json["format"], "json");
    }
}
