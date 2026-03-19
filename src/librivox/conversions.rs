use crate::audiobook::{Audiobook, AudiobookAuthor, Chapter};
use crate::librivox::types::{LibriVoxBook, LibriVoxSection};

pub fn audiobook_from_book(book: LibriVoxBook) -> Option<Audiobook> {
    let id: i64 = book.id.parse().ok()?;

    let authors = book
        .authors
        .unwrap_or_default()
        .into_iter()
        .filter_map(|a| {
            let author_id: i64 = a.id.parse().ok()?;
            Some(AudiobookAuthor {
                id: author_id,
                first_name: a.first_name,
                last_name: a.last_name,
                dob: a.dob,
                dod: a.dod,
            })
        })
        .collect();

    let num_sections = book
        .num_sections
        .as_deref()
        .and_then(|s| s.parse::<i32>().ok());

    Some(Audiobook {
        id,
        title: book.title,
        description: book.description,
        language: book.language,
        copyright_year: book.copyright_year,
        num_sections,
        total_time: book.totaltime,
        total_time_secs: book.totaltimesecs,
        authors,
        url_text_source: book.url_text_source,
        url_zip_file: book.url_zip_file,
        url_librivox: book.url_librivox,
        url_iarchive: book.url_iarchive,
        coverart_url: book.coverart_jpg,
        coverart_thumbnail: book.coverart_thumbnail,
    })
}

pub fn chapter_from_section(section: LibriVoxSection, audiobook_id: i64) -> Option<Chapter> {
    let id: i64 = section.id.parse().ok()?;
    let section_number: i32 = section.section_number.parse().ok()?;

    let duration_seconds = section.duration.as_deref().and_then(parse_duration_hms);

    let readers = section
        .readers
        .unwrap_or_default()
        .into_iter()
        .map(|r| r.display_name)
        .collect();

    Some(Chapter {
        id,
        audiobook_id,
        title: section.title,
        section_number,
        duration: section.duration,
        duration_seconds,
        listen_url: section.listen_url,
        language: section.language,
        readers,
    })
}

/// Converts "HH:MM:SS" to total seconds.
fn parse_duration_hms(hms: &str) -> Option<i32> {
    let parts: Vec<&str> = hms.split(':').collect();
    if parts.len() != 3 {
        return None;
    }
    let hours: i32 = parts[0].parse().ok()?;
    let minutes: i32 = parts[1].parse().ok()?;
    let seconds: i32 = parts[2].parse().ok()?;
    Some(hours * 3600 + minutes * 60 + seconds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::librivox::types::{LibriVoxAuthor, LibriVoxBook, LibriVoxReader, LibriVoxSection};

    #[test]
    fn test_audiobook_from_book() {
        let book = LibriVoxBook {
            id: "128".to_string(),
            title: "Pride and Prejudice".to_string(),
            description: Some("A novel".to_string()),
            language: Some("English".to_string()),
            copyright_year: Some("1813".to_string()),
            num_sections: Some("61".to_string()),
            totaltime: Some("11:54:29".to_string()),
            totaltimesecs: Some(42869),
            url_text_source: Some("https://gutenberg.org/1342".to_string()),
            url_zip_file: None,
            url_librivox: Some("https://librivox.org/pride".to_string()),
            url_iarchive: None,
            authors: Some(vec![LibriVoxAuthor {
                id: "1".to_string(),
                first_name: "Jane".to_string(),
                last_name: "Austen".to_string(),
                dob: Some("1775".to_string()),
                dod: Some("1817".to_string()),
            }]),
            coverart_jpg: Some("https://example.com/cover.jpg".to_string()),
            coverart_thumbnail: Some("https://example.com/thumb.jpg".to_string()),
        };

        let audiobook = audiobook_from_book(book).unwrap();
        assert_eq!(audiobook.id, 128);
        assert_eq!(audiobook.title, "Pride and Prejudice");
        assert_eq!(audiobook.num_sections, Some(61));
        assert_eq!(audiobook.total_time_secs, Some(42869));
        assert_eq!(audiobook.authors.len(), 1);
        assert_eq!(audiobook.authors[0].first_name, "Jane");
    }

    #[test]
    fn test_audiobook_from_book_invalid_id() {
        let book = LibriVoxBook {
            id: "not_a_number".to_string(),
            title: "Test".to_string(),
            description: None,
            language: None,
            copyright_year: None,
            num_sections: None,
            totaltime: None,
            totaltimesecs: None,
            url_text_source: None,
            url_zip_file: None,
            url_librivox: None,
            url_iarchive: None,
            authors: None,
            coverart_jpg: None,
            coverart_thumbnail: None,
        };

        assert!(audiobook_from_book(book).is_none());
    }

    #[test]
    fn test_chapter_from_section() {
        let section = LibriVoxSection {
            id: "1001".to_string(),
            title: "Chapter 1".to_string(),
            section_number: "1".to_string(),
            duration: Some("00:15:32".to_string()),
            listen_url: "https://example.com/ch1.mp3".to_string(),
            language: Some("English".to_string()),
            readers: Some(vec![LibriVoxReader {
                display_name: "Reader One".to_string(),
            }]),
        };

        let chapter = chapter_from_section(section, 128).unwrap();
        assert_eq!(chapter.id, 1001);
        assert_eq!(chapter.audiobook_id, 128);
        assert_eq!(chapter.section_number, 1);
        assert_eq!(chapter.duration_seconds, Some(932));
        assert_eq!(chapter.readers, vec!["Reader One"]);
    }

    #[test]
    fn test_parse_duration_hms() {
        assert_eq!(parse_duration_hms("01:23:45"), Some(5025));
        assert_eq!(parse_duration_hms("00:00:00"), Some(0));
        assert_eq!(parse_duration_hms("00:00:01"), Some(1));
        assert_eq!(parse_duration_hms("10:00:00"), Some(36000));
        assert_eq!(parse_duration_hms("00:15:32"), Some(932));
    }

    #[test]
    fn test_parse_duration_hms_invalid() {
        assert_eq!(parse_duration_hms("invalid"), None);
        assert_eq!(parse_duration_hms("00:00"), None);
        assert_eq!(parse_duration_hms(""), None);
        assert_eq!(parse_duration_hms("aa:bb:cc"), None);
    }
}
