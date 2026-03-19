use async_graphql::{Enum, Interface, Object, SimpleObject};

use crate::audiobook::{
    Audiobook as DomainAudiobook, AudiobookAuthor as DomainAudiobookAuthor,
    AudiobookSource as DomainAudiobookSource,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Enum)]
pub enum AudiobookSource {
    LibriVox,
}

impl From<DomainAudiobookSource> for AudiobookSource {
    fn from(source: DomainAudiobookSource) -> Self {
        match source {
            DomainAudiobookSource::LibriVox => AudiobookSource::LibriVox,
        }
    }
}

#[derive(Debug, Clone, SimpleObject)]
pub struct AudiobookAuthor {
    pub first_name: String,
    pub last_name: String,
    pub dob: Option<String>,
    pub dod: Option<String>,
}

impl From<DomainAudiobookAuthor> for AudiobookAuthor {
    fn from(author: DomainAudiobookAuthor) -> Self {
        Self {
            first_name: author.first_name,
            last_name: author.last_name,
            dob: author.dob,
            dod: author.dod,
        }
    }
}

#[allow(clippy::duplicated_attributes)]
#[derive(Interface)]
#[graphql(
    field(
        name = "id",
        ty = "String",
        desc = "Unique identifier (prefixed with source)"
    ),
    field(name = "title", ty = "String", desc = "Audiobook title"),
    field(
        name = "source",
        ty = "AudiobookSource",
        desc = "Data source this record came from"
    ),
    field(name = "source_id", ty = "String", desc = "ID in the source system"),
    field(
        name = "description",
        ty = "Option<String>",
        desc = "Audiobook description"
    ),
    field(name = "language", ty = "Option<String>", desc = "Language"),
    field(
        name = "authors",
        ty = "Vec<AudiobookAuthor>",
        desc = "Audiobook authors"
    ),
    field(
        name = "num_sections",
        ty = "Option<i32>",
        desc = "Number of sections/chapters"
    ),
    field(
        name = "total_time",
        ty = "Option<String>",
        desc = "Total duration as HH:MM:SS"
    ),
    field(
        name = "total_time_secs",
        ty = "Option<i64>",
        desc = "Total duration in seconds"
    ),
    field(name = "coverart_url", ty = "Option<String>", desc = "Cover art URL")
)]
pub enum Audiobook {
    LibriVox(LibriVoxAudiobook),
}

#[derive(Debug, Clone)]
pub struct LibriVoxAudiobook {
    pub inner: DomainAudiobook,
}

#[Object]
impl LibriVoxAudiobook {
    async fn id(&self) -> String {
        DomainAudiobookSource::LibriVox.format_id(self.inner.id)
    }

    async fn title(&self) -> &str {
        &self.inner.title
    }

    async fn source(&self) -> AudiobookSource {
        AudiobookSource::LibriVox
    }

    async fn source_id(&self) -> String {
        self.inner.id.to_string()
    }

    async fn description(&self) -> Option<String> {
        self.inner.description.clone()
    }

    async fn language(&self) -> Option<String> {
        self.inner.language.clone()
    }

    async fn authors(&self) -> Vec<AudiobookAuthor> {
        self.inner
            .authors
            .iter()
            .map(|a| AudiobookAuthor::from(a.clone()))
            .collect()
    }

    async fn num_sections(&self) -> Option<i32> {
        self.inner.num_sections
    }

    async fn total_time(&self) -> Option<String> {
        self.inner.total_time.clone()
    }

    async fn total_time_secs(&self) -> Option<i64> {
        self.inner.total_time_secs
    }

    async fn coverart_url(&self) -> Option<String> {
        self.inner.coverart_url.clone()
    }

    // LibriVox-specific fields

    async fn copyright_year(&self) -> Option<String> {
        self.inner.copyright_year.clone()
    }

    async fn url_text_source(&self) -> Option<String> {
        self.inner.url_text_source.clone()
    }

    async fn url_zip_file(&self) -> Option<String> {
        self.inner.url_zip_file.clone()
    }

    async fn url_librivox(&self) -> Option<String> {
        self.inner.url_librivox.clone()
    }

    async fn url_iarchive(&self) -> Option<String> {
        self.inner.url_iarchive.clone()
    }

    async fn coverart_thumbnail(&self) -> Option<String> {
        self.inner.coverart_thumbnail.clone()
    }
}

impl From<DomainAudiobook> for Audiobook {
    fn from(a: DomainAudiobook) -> Self {
        Audiobook::LibriVox(LibriVoxAudiobook { inner: a })
    }
}

impl From<DomainAudiobook> for LibriVoxAudiobook {
    fn from(a: DomainAudiobook) -> Self {
        LibriVoxAudiobook { inner: a }
    }
}
