//! Song library: the songs saved in the same folder as the current song.
//! Port of the listing half of `SongLibraryDialog.kt`. Click a row to open
//! it. The folder is the current song's folder on purpose: no directory
//! picker, no settings, just "the songs next to this one".

use crate::serialization::decode_song;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct LibraryEntry {
    pub file: PathBuf,
    pub title: String,
    pub genre: String,
    pub last_modified: SystemTime,
}

/// Every `.json` song in `folder`, newest first. Files that don't decode
/// as songs are skipped.
pub fn list_library(folder: Option<&Path>) -> Vec<LibraryEntry> {
    let Some(folder) = folder else {
        return Vec::new();
    };
    if !folder.is_dir() {
        return Vec::new();
    }
    let mut entries: Vec<LibraryEntry> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(folder) {
        for entry in rd.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() || path.extension().map(|e| e != "json").unwrap_or(true) {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Some(song) = decode_song(&text) else {
                continue;
            };
            let last_modified = entry
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);
            entries.push(LibraryEntry {
                file: path,
                title: song.title,
                genre: song.genre,
                last_modified,
            });
        }
    }
    entries.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
    entries
}

/// Filter entries by title or genre (case-insensitive), like the Kotlin
/// search box.
pub fn filter_library<'a>(entries: &'a [LibraryEntry], query: &str) -> Vec<&'a LibraryEntry> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return entries.iter().collect();
    }
    entries
        .iter()
        .filter(|e| e.title.to_lowercase().contains(&q) || e.genre.to_lowercase().contains(&q))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::serialization::encode_song;
    use crate::song::Song;

    #[test]
    fn library_lists_and_searches() {
        let dir = std::env::temp_dir().join(format!(
            "lyricshop-lib-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let a = Song::new("Alpha Song", "Rock / Metal");
        let b = Song::new("Beta Ballad", "Country / Americana");
        std::fs::write(dir.join("a.json"), encode_song(&a)).unwrap();
        std::fs::write(dir.join("b.json"), encode_song(&b)).unwrap();
        std::fs::write(dir.join("notasong.json"), "{not json").unwrap();
        std::fs::write(dir.join("notes.txt"), "hello").unwrap();

        let entries = list_library(Some(&dir));
        assert_eq!(entries.len(), 2);
        let hits = filter_library(&entries, "beta");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Beta Ballad");
        let hits = filter_library(&entries, "rock");
        assert_eq!(hits.len(), 1);
        assert_eq!(filter_library(&entries, "").len(), 2);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn no_folder_means_empty() {
        assert!(list_library(None).is_empty());
    }
}
