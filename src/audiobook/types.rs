use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudiobookAuthor {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub dob: Option<String>,
    pub dod: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Audiobook {
    pub id: i64,
    pub title: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub copyright_year: Option<String>,
    pub num_sections: Option<i32>,
    pub total_time: Option<String>,
    pub total_time_secs: Option<i64>,
    pub authors: Vec<AudiobookAuthor>,
    pub url_text_source: Option<String>,
    pub url_zip_file: Option<String>,
    pub url_librivox: Option<String>,
    pub url_iarchive: Option<String>,
    pub coverart_url: Option<String>,
    pub coverart_thumbnail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: i64,
    pub audiobook_id: i64,
    pub title: String,
    pub section_number: i32,
    pub duration: Option<String>,
    pub duration_seconds: Option<i32>,
    pub listen_url: String,
    pub language: Option<String>,
    pub readers: Vec<String>,
}
