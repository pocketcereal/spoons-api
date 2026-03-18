use crate::audiobook::Chapter;
use crate::error::Result;
use crate::librivox::client::LibriVoxClient;
use crate::librivox::conversions::chapter_from_section;
use crate::librivox::types::LibriVoxSectionsResponse;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct ChapterParams {
    id: i64,
    format: &'static str,
    extended: i32,
}

pub async fn get_chapters(
    client: &LibriVoxClient,
    audiobook_id: i64,
) -> Result<Vec<Chapter>> {
    let params = ChapterParams {
        id: audiobook_id,
        format: "json",
        extended: 1,
    };

    let response: LibriVoxSectionsResponse = client
        .get_with_query("/audiotracks", &params)
        .await?;

    let sections = response.sections.unwrap_or_default();
    Ok(sections
        .into_iter()
        .filter_map(|s| chapter_from_section(s, audiobook_id))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chapter_params_serialization() {
        let params = ChapterParams {
            id: 128,
            format: "json",
            extended: 1,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["id"], 128);
        assert_eq!(json["format"], "json");
        assert_eq!(json["extended"], 1);
    }
}
