use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct LibriVoxBooksResponse {
    pub books: Option<Vec<LibriVoxBook>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibriVoxBook {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub copyright_year: Option<String>,
    pub num_sections: Option<String>,
    pub totaltime: Option<String>,
    pub totaltimesecs: Option<i64>,
    pub url_text_source: Option<String>,
    pub url_zip_file: Option<String>,
    pub url_librivox: Option<String>,
    pub url_iarchive: Option<String>,
    pub authors: Option<Vec<LibriVoxAuthor>>,
    #[serde(rename = "coverart_jpg")]
    pub coverart_jpg: Option<String>,
    #[serde(rename = "coverart_thumbnail")]
    pub coverart_thumbnail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibriVoxAuthor {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub dob: Option<String>,
    pub dod: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibriVoxSectionsResponse {
    pub sections: Option<Vec<LibriVoxSection>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibriVoxSection {
    pub id: String,
    pub title: String,
    pub section_number: String,
    #[serde(rename = "playtime")]
    pub duration: Option<String>,
    pub listen_url: String,
    pub language: Option<String>,
    pub readers: Option<Vec<LibriVoxReader>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LibriVoxReader {
    pub display_name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_books_response() {
        let json = r#"{
            "books": [{
                "id": "128",
                "title": "Pride and Prejudice",
                "description": "A novel by Jane Austen",
                "language": "English",
                "copyright_year": "1813",
                "num_sections": "61",
                "totaltime": "11:54:29",
                "totaltimesecs": 42869,
                "url_text_source": "https://www.gutenberg.org/etext/1342",
                "url_zip_file": "https://www.archive.org/download/pride_and_prejudice_0711_librivox/pride_and_prejudice_0711_librivox_64kb_mp3.zip",
                "url_librivox": "https://librivox.org/pride-and-prejudice-by-jane-austen/",
                "url_iarchive": "https://www.archive.org/details/pride_and_prejudice_0711_librivox",
                "authors": [{"id": "1", "first_name": "Jane", "last_name": "Austen", "dob": "1775", "dod": "1817"}],
                "coverart_jpg": "https://archive.org/download/LibrivoxCdCoverArt8/Pride_and_Prejudice_1002.jpg",
                "coverart_thumbnail": "https://archive.org/download/LibrivoxCdCoverArt8/Pride_and_Prejudice_1002_thumb.jpg"
            }]
        }"#;

        let response: LibriVoxBooksResponse = serde_json::from_str(json).unwrap();
        let books = response.books.unwrap();
        assert_eq!(books.len(), 1);
        assert_eq!(books[0].id, "128");
        assert_eq!(books[0].title, "Pride and Prejudice");
        assert_eq!(books[0].num_sections, Some("61".to_string()));
        assert_eq!(books[0].totaltimesecs, Some(42869));

        let authors = books[0].authors.as_ref().unwrap();
        assert_eq!(authors.len(), 1);
        assert_eq!(authors[0].first_name, "Jane");
        assert_eq!(authors[0].last_name, "Austen");
    }

    #[test]
    fn test_deserialize_books_response_null_books() {
        let json = r#"{"books": null}"#;
        let response: LibriVoxBooksResponse = serde_json::from_str(json).unwrap();
        assert!(response.books.is_none());
    }

    #[test]
    fn test_deserialize_books_response_empty_books() {
        let json = r#"{"books": []}"#;
        let response: LibriVoxBooksResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.books.unwrap().len(), 0);
    }

    #[test]
    fn test_deserialize_sections_response() {
        let json = r#"{
            "sections": [{
                "id": "1001",
                "title": "Chapter 1",
                "section_number": "1",
                "playtime": "00:15:32",
                "listen_url": "https://www.archive.org/download/pride_and_prejudice/chapter01.mp3",
                "language": "English",
                "readers": [{"display_name": "Jane Reader"}]
            }]
        }"#;

        let response: LibriVoxSectionsResponse = serde_json::from_str(json).unwrap();
        let sections = response.sections.unwrap();
        assert_eq!(sections.len(), 1);
        assert_eq!(sections[0].id, "1001");
        assert_eq!(sections[0].title, "Chapter 1");
        assert_eq!(sections[0].section_number, "1");
        assert_eq!(sections[0].duration, Some("00:15:32".to_string()));

        let readers = sections[0].readers.as_ref().unwrap();
        assert_eq!(readers[0].display_name, "Jane Reader");
    }

    #[test]
    fn test_deserialize_minimal_book() {
        let json = r#"{
            "books": [{
                "id": "1",
                "title": "Test Book",
                "description": null,
                "language": null,
                "copyright_year": null,
                "num_sections": null,
                "totaltime": null,
                "totaltimesecs": null,
                "url_text_source": null,
                "url_zip_file": null,
                "url_librivox": null,
                "url_iarchive": null,
                "authors": null,
                "coverart_jpg": null,
                "coverart_thumbnail": null
            }]
        }"#;

        let response: LibriVoxBooksResponse = serde_json::from_str(json).unwrap();
        let books = response.books.unwrap();
        assert_eq!(books[0].title, "Test Book");
        assert!(books[0].authors.is_none());
    }
}
