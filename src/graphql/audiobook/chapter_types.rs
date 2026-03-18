use async_graphql::{Interface, Object};

use crate::audiobook::{
    AudiobookSource as DomainAudiobookSource, Chapter as DomainChapter,
};

use super::audiobook_types::AudiobookSource;

#[allow(clippy::duplicated_attributes)]
#[derive(Interface)]
#[graphql(
    field(
        name = "id",
        ty = "String",
        desc = "Unique identifier (prefixed with source)"
    ),
    field(name = "title", ty = "String", desc = "Chapter title"),
    field(
        name = "source",
        ty = "AudiobookSource",
        desc = "Data source this record came from"
    ),
    field(name = "source_id", ty = "String", desc = "ID in the source system"),
    field(
        name = "audiobook_id",
        ty = "String",
        desc = "Audiobook ID (prefixed with source)"
    ),
    field(
        name = "section_number",
        ty = "i32",
        desc = "Section/chapter number"
    ),
    field(
        name = "duration",
        ty = "Option<String>",
        desc = "Duration as HH:MM:SS"
    ),
    field(
        name = "duration_seconds",
        ty = "Option<i32>",
        desc = "Duration in seconds"
    ),
    field(name = "listen_url", ty = "String", desc = "Audio stream URL")
)]
pub enum Chapter {
    LibriVox(LibriVoxChapter),
}

#[derive(Debug, Clone)]
pub struct LibriVoxChapter {
    pub inner: DomainChapter,
}

#[Object]
impl LibriVoxChapter {
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

    async fn audiobook_id(&self) -> String {
        DomainAudiobookSource::LibriVox.format_id(self.inner.audiobook_id)
    }

    async fn section_number(&self) -> i32 {
        self.inner.section_number
    }

    async fn duration(&self) -> Option<String> {
        self.inner.duration.clone()
    }

    async fn duration_seconds(&self) -> Option<i32> {
        self.inner.duration_seconds
    }

    async fn listen_url(&self) -> &str {
        &self.inner.listen_url
    }

    // LibriVox-specific fields

    async fn language(&self) -> Option<String> {
        self.inner.language.clone()
    }

    async fn readers(&self) -> Vec<String> {
        self.inner.readers.clone()
    }
}

impl From<DomainChapter> for Chapter {
    fn from(c: DomainChapter) -> Self {
        Chapter::LibriVox(LibriVoxChapter { inner: c })
    }
}

impl From<DomainChapter> for LibriVoxChapter {
    fn from(c: DomainChapter) -> Self {
        LibriVoxChapter { inner: c }
    }
}
