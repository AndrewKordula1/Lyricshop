//! File save/load/open and TXT export. Port of `SongStorage.kt`.
//! The file dialogs are `rfd` native dialogs instead of AWT `FileDialog`.
//! Every function reports success or failure so the UI can say what
//! actually happened instead of failing silently.

use crate::pdf::export_song_to_pdf;
use crate::serialization::{decode_song, encode_song};
use crate::song::Song;
use std::path::PathBuf;

/// The file the song was last saved to or opened from, if any.
/// Snapshots and the library live next to it.
#[derive(Debug, Default)]
pub struct SongFile {
    pub current: Option<PathBuf>,
}

impl SongFile {
    pub fn has_current_file(&self) -> bool {
        self.current.is_some()
    }

    pub fn clear(&mut self) {
        self.current = None;
    }

    /// Load a song file directly into the song, replacing its contents,
    /// and make it the current file. Used by Open and the library.
    pub fn load_song_file(&mut self, song: &mut Song, file: &PathBuf) -> bool {
        match std::fs::read_to_string(file) {
            Ok(text) => match decode_song(&text) {
                Some(loaded) => {
                    song.copy_from(&loaded);
                    self.current = Some(file.clone());
                    true
                }
                None => false,
            },
            Err(_) => false,
        }
    }

    /// Save to the known file, or prompt for one (Save As) when there
    /// isn't one yet.
    pub fn save_song(&mut self, song: &Song) -> bool {
        if let Some(file) = &self.current {
            if std::fs::write(file, encode_song(song)).is_ok() {
                return true;
            }
            // Fall through to Save As on write failure.
        }
        self.save_song_as(song)
    }

    pub fn save_song_as(&mut self, song: &Song) -> bool {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Save Song As")
            .set_file_name(format!("{}.json", safe_filename(&song.title)))
            .add_filter("Song files", &["json"])
            .save_file()
        else {
            return false;
        };
        let path = with_extension(path, "json");
        match std::fs::write(&path, encode_song(song)) {
            Ok(()) => {
                self.current = Some(path);
                true
            }
            Err(_) => false,
        }
    }

    /// Write to the already-known file without prompting. Used by
    /// autosave. Returns false when there is no known file yet or the
    /// write fails.
    pub fn autosave(&self, song: &Song) -> bool {
        let Some(file) = &self.current else {
            return false;
        };
        std::fs::write(file, encode_song(song)).is_ok()
    }

    pub fn export_to_txt(&self, song: &Song) -> bool {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Export to TXT")
            .set_file_name(format!("{}.txt", safe_filename(&song.title)))
            .add_filter("Text files", &["txt"])
            .save_file()
        else {
            return false;
        };
        let path = with_extension(path, "txt");
        std::fs::write(&path, txt_export(song)).is_ok()
    }

    pub fn export_to_pdf(&self, song: &Song) -> bool {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Export to PDF")
            .set_file_name(format!("{}.pdf", safe_filename(&song.title)))
            .add_filter("PDF files", &["pdf"])
            .save_file()
        else {
            return false;
        };
        let path = with_extension(path, "pdf");
        export_song_to_pdf(song, &path)
    }

    pub fn open_song(&mut self, song: &mut Song) -> bool {
        let Some(path) = rfd::FileDialog::new()
            .set_title("Open Song")
            .add_filter("Song files", &["json"])
            .pick_file()
        else {
            return false;
        };
        self.load_song_file(song, &path)
    }
}

fn safe_filename(title: &str) -> String {
    let t = title.trim();
    if t.is_empty() {
        "song".to_string()
    } else {
        t.to_string()
    }
}

fn with_extension(mut path: PathBuf, ext: &str) -> PathBuf {
    if path.extension().map(|e| e != ext).unwrap_or(true) {
        path.set_extension(ext);
    }
    path
}

/// The TXT export text (pure, so it is testable without a dialog).
pub fn txt_export(song: &Song) -> String {
    let mut out = String::new();
    out.push_str(&format!("Title: {}\n", song.title));
    out.push_str(&format!(
        "Genre: {}\nKey: {} | BPM: {} | Groove: {}\n",
        song.genre,
        if song.key.trim().is_empty() {
            "N/A"
        } else {
            &song.key
        },
        song.bpm,
        song.groove
    ));
    out.push_str(&"=".repeat(40));
    out.push_str("\n\n");
    for section in &song.sections {
        let label = if section.label.trim().is_empty() {
            section.section_type.default_name().to_string()
        } else {
            section.label.clone()
        };
        let time_sig = match &section.time_signature {
            Some(ts) if !ts.trim().is_empty() => format!(" ({})", ts),
            _ => String::new(),
        };
        out.push_str(&format!("[{}{}]\n", label, time_sig));
        if let Some(chords) = &section.chords {
            if !chords.trim().is_empty() {
                out.push_str(&format!("Chords: {}\n", chords));
            }
        }
        if section.lyrics.trim().is_empty() {
            out.push_str("(Empty section)");
        } else {
            out.push_str(&section.lyrics);
        }
        out.push_str("\n\n");
        out.push_str(&"-".repeat(20));
        out.push_str("\n\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn txt_export_shape() {
        let mut song = Song::new("My Song", "Rock / Metal");
        song.key = "E Minor".to_string();
        song.bpm = 132;
        song.sections = crate::song::generate_sections("Rock / Metal");
        song.sections[1].lyrics = "hello world".to_string();
        song.sections[1].chords = Some("A5 C#5".to_string());
        let text = txt_export(&song);
        assert!(text.contains("Title: My Song"));
        assert!(text.contains("Key: E Minor | BPM: 132"));
        assert!(text.contains("[Verse 1]"));
        assert!(text.contains("Chords: A5 C#5"));
        assert!(text.contains("hello world"));
    }

    #[test]
    fn save_and_load_round_trip() {
        let dir = std::env::temp_dir().join(format!(
            "lyricshop-save-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("song.json");
        let mut song = Song::new("Round Trip", "Pop / Electronic");
        song.sections = crate::song::generate_sections("Pop / Electronic");
        let mut sf = SongFile::default();
        sf.current = Some(file.clone());
        assert!(sf.save_song(&song));
        let mut loaded = Song::default();
        assert!(sf.load_song_file(&mut loaded, &file));
        assert_eq!(loaded.title, "Round Trip");
        assert_eq!(loaded.sections.len(), 8);
        std::fs::remove_dir_all(&dir).ok();
    }
}
