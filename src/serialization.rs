//! JSON save/load for songs. Port of `SongSerialization.kt`.
//! The field names match the Kotlin DTOs exactly, so files saved by the
//! Kotlin app open here and vice versa. A missing section `id` gets a
//! fresh one; unknown fields are ignored.

use crate::song::{SectionType, Song, SongSection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SongSectionDto {
    #[serde(default)]
    id: String,
    #[serde(rename = "type", default = "default_section_type")]
    section_type: String,
    #[serde(default)]
    label: String,
    #[serde(default)]
    lyrics: String,
    #[serde(default)]
    chords: Option<String>,
    #[serde(rename = "timeSignature", default)]
    time_signature: Option<String>,
    #[serde(rename = "isLocked", default)]
    is_locked: bool,
}

fn default_section_type() -> String {
    "VERSE".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SongDto {
    #[serde(default = "default_title")]
    title: String,
    #[serde(default = "default_genre")]
    genre: String,
    #[serde(default = "default_key")]
    key: String,
    #[serde(default = "default_bpm")]
    bpm: i32,
    #[serde(default = "default_groove")]
    groove: String,
    #[serde(rename = "isMetadataLocked", default)]
    is_metadata_locked: bool,
    #[serde(default)]
    sections: Vec<SongSectionDto>,
}

fn default_title() -> String {
    "Untitled".to_string()
}
fn default_genre() -> String {
    "Rock / Metal".to_string()
}
fn default_key() -> String {
    "C Major".to_string()
}
fn default_bpm() -> i32 {
    120
}
fn default_groove() -> String {
    "Straight".to_string()
}

impl From<&Song> for SongDto {
    fn from(song: &Song) -> Self {
        SongDto {
            title: song.title.clone(),
            genre: song.genre.clone(),
            key: song.key.clone(),
            bpm: song.bpm,
            groove: song.groove.clone(),
            is_metadata_locked: song.is_metadata_locked,
            sections: song
                .sections
                .iter()
                .map(|s| SongSectionDto {
                    id: s.id.clone(),
                    section_type: format!("{:?}", s.section_type).to_uppercase(),
                    label: s.label.clone(),
                    lyrics: s.lyrics.clone(),
                    chords: s.chords.clone(),
                    time_signature: s.time_signature.clone(),
                    is_locked: s.is_locked,
                })
                .collect(),
        }
    }
}

impl From<SongDto> for Song {
    fn from(dto: SongDto) -> Self {
        Song {
            title: dto.title,
            genre: dto.genre,
            key: dto.key,
            bpm: dto.bpm,
            groove: dto.groove,
            is_metadata_locked: dto.is_metadata_locked,
            sections: dto
                .sections
                .into_iter()
                .map(|d| SongSection {
                    id: if d.id.is_empty() {
                        crate::song::fresh_id()
                    } else {
                        d.id
                    },
                    section_type: SectionType::parse(&d.section_type),
                    label: d.label,
                    lyrics: d.lyrics,
                    chords: d.chords,
                    time_signature: d.time_signature,
                    is_locked: d.is_locked,
                })
                .collect(),
        }
    }
}

/// Pretty-printed JSON, same shape as the Kotlin app's output.
pub fn encode_song(song: &Song) -> String {
    let dto = SongDto::from(song);
    serde_json::to_string_pretty(&dto).unwrap_or_else(|_| "{}".to_string())
}

/// Returns `None` when the text is not a valid song file, instead of
/// panicking — same contract as the Kotlin `decodeSong`.
pub fn decode_song(text: &str) -> Option<Song> {
    serde_json::from_str::<SongDto>(text).ok().map(Song::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::generate_sections;

    #[test]
    fn json_round_trip_keeps_everything() {
        let mut song = Song::new("Drain You", "Grunge");
        song.key = "E Minor".to_string();
        song.bpm = 132;
        song.sections = generate_sections("Rock / Metal");
        song.sections[1].lyrics = "One baby to another says\nI'm lucky to have met you".to_string();
        song.sections[1].chords = Some("A5 C#5 F#5 B5".to_string());
        song.sections[1].time_signature = Some("4/4".to_string());
        let id = song.sections[1].id.clone();

        let text = encode_song(&song);
        let back = decode_song(&text).expect("round trip decodes");
        assert_eq!(back.title, "Drain You");
        assert_eq!(back.bpm, 132);
        assert_eq!(back.sections.len(), 8);
        assert_eq!(back.sections[1].id, id);
        assert_eq!(back.sections[1].chords.as_deref(), Some("A5 C#5 F#5 B5"));
        assert_eq!(back.sections[1].time_signature.as_deref(), Some("4/4"));
    }

    #[test]
    fn json_with_quotes_and_braces_round_trips() {
        let mut song = Song::new("Test \"quoted\" } song", "Rock / Metal");
        song.sections = vec![SongSection::new(crate::song::SectionType::Verse, "Verse 1")];
        song.sections[0].lyrics = "He said \"hello\" \\ and left }".to_string();
        let text = encode_song(&song);
        let back = decode_song(&text).expect("decodes");
        assert_eq!(back.title, "Test \"quoted\" } song");
        assert_eq!(back.sections[0].lyrics, "He said \"hello\" \\ and left }");
    }

    #[test]
    fn invalid_json_returns_none() {
        assert!(decode_song("not json at all").is_none());
        assert!(decode_song("").is_none());
    }

    #[test]
    fn unknown_type_falls_back_to_verse() {
        let text = r#"{"title":"T","genre":"G","key":"C Major","bpm":120,"groove":"Straight","isMetadataLocked":false,"sections":[{"id":"x","type":"NOPE","label":"L","lyrics":"","chords":null,"timeSignature":null,"isLocked":false}]}"#;
        let back = decode_song(text).expect("decodes");
        assert_eq!(back.sections[0].section_type, SectionType::Verse);
    }

    #[test]
    fn kotlin_saved_file_opens() {
        // A file in the exact shape the Kotlin app writes.
        let text = r#"{
  "title": "Old Song",
  "genre": "Rock / Metal",
  "key": "C Major",
  "bpm": 120,
  "groove": "Straight",
  "isMetadataLocked": false,
  "sections": [
    {
      "id": "abc-123",
      "type": "CHORUS",
      "label": "Chorus",
      "lyrics": "sing it loud",
      "chords": "C G Am F",
      "timeSignature": "4/4",
      "isLocked": false
    }
  ]
}"#;
        let back = decode_song(text).expect("decodes");
        assert_eq!(back.title, "Old Song");
        assert_eq!(back.sections[0].section_type, SectionType::Chorus);
        assert_eq!(back.sections[0].chords.as_deref(), Some("C G Am F"));
    }
}
