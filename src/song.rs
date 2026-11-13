//! Song data model: sections, genre templates.
//! Port of `SongData.kt` — same section types, same templates, same
//! generated section lists, so a song built in either app is the same song.

use serde::{Deserialize, Serialize};

/// Section types, with the same default labels as the Kotlin app.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SectionType {
    Intro,
    Verse,
    PreChorus,
    Chorus,
    PostChorus,
    Bridge,
    Solo,
    Outro,
    Custom,
}

impl SectionType {
    pub fn default_name(&self) -> &'static str {
        match self {
            SectionType::Intro => "Intro",
            SectionType::Verse => "Verse",
            SectionType::PreChorus => "Pre-Chorus",
            SectionType::Chorus => "Chorus",
            SectionType::PostChorus => "Post-Chorus",
            SectionType::Bridge => "Bridge",
            SectionType::Solo => "Solo",
            SectionType::Outro => "Outro",
            SectionType::Custom => "Custom",
        }
    }

    pub fn all() -> &'static [SectionType] {
        &[
            SectionType::Intro,
            SectionType::Verse,
            SectionType::PreChorus,
            SectionType::Chorus,
            SectionType::PostChorus,
            SectionType::Bridge,
            SectionType::Solo,
            SectionType::Outro,
            SectionType::Custom,
        ]
    }

    /// Parse a type name the way the Kotlin loader does: unknown names
    /// fall back to Verse instead of failing the load.
    pub fn parse(name: &str) -> SectionType {
        match name {
            "INTRO" => SectionType::Intro,
            "VERSE" => SectionType::Verse,
            "PRE_CHORUS" => SectionType::PreChorus,
            "CHORUS" => SectionType::Chorus,
            "POST_CHORUS" => SectionType::PostChorus,
            "BRIDGE" => SectionType::Bridge,
            "SOLO" => SectionType::Solo,
            "OUTRO" => SectionType::Outro,
            "CUSTOM" => SectionType::Custom,
            _ => SectionType::Verse,
        }
    }
}

fn new_id() -> String {
    // A short unique id (no uuid crate needed): time + a counter.
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let t = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}-{:x}", t, n)
}

/// A fresh unique section id (used when a loaded file has none).
pub fn fresh_id() -> String {
    new_id()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SongSection {
    pub id: String,
    #[serde(rename = "type")]
    pub section_type: SectionType,
    pub label: String,
    #[serde(default)]
    pub lyrics: String,
    #[serde(default)]
    pub chords: Option<String>,
    #[serde(rename = "timeSignature", default)]
    pub time_signature: Option<String>,
    #[serde(rename = "isLocked", default)]
    pub is_locked: bool,
}

impl SongSection {
    pub fn new(section_type: SectionType, label: impl Into<String>) -> Self {
        SongSection {
            id: new_id(),
            section_type,
            label: label.into(),
            lyrics: String::new(),
            chords: None,
            time_signature: None,
            is_locked: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Song {
    #[serde(default = "default_title")]
    pub title: String,
    #[serde(default = "default_genre")]
    pub genre: String,
    #[serde(default = "default_key")]
    pub key: String,
    #[serde(default = "default_bpm")]
    pub bpm: i32,
    #[serde(default = "default_groove")]
    pub groove: String,
    #[serde(rename = "isMetadataLocked", default)]
    pub is_metadata_locked: bool,
    #[serde(default)]
    pub sections: Vec<SongSection>,
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

impl Default for Song {
    fn default() -> Self {
        Song {
            title: "New Song".to_string(),
            genre: "Rock / Metal".to_string(),
            key: "C Major".to_string(),
            bpm: 120,
            groove: "Straight".to_string(),
            is_metadata_locked: false,
            sections: Vec::new(),
        }
    }
}

impl Song {
    pub fn new(title: impl Into<String>, genre: impl Into<String>) -> Self {
        Song {
            title: title.into(),
            genre: genre.into(),
            ..Song::default()
        }
    }

    /// Replace this song's contents with another's (used by snapshot
    /// restore, library open, and New Song). Mirrors Kotlin `copyFrom`.
    pub fn copy_from(&mut self, other: &Song) {
        self.title = other.title.clone();
        self.genre = other.genre.clone();
        self.key = other.key.clone();
        self.bpm = other.bpm;
        self.groove = other.groove.clone();
        self.is_metadata_locked = other.is_metadata_locked;
        self.sections = other.sections.clone();
    }

    /// Duplicate the section at `index`, inserting the copy immediately
    /// after the original. The copy keeps the label, type, time
    /// signature, chords, and lyrics, but gets a fresh id (ids drive
    /// `push_id` and playback, so reusing the original's would clash)
    /// and is unlocked, so it is immediately editable even when the
    /// original is locked. Does nothing when `index` is out of range.
    pub fn duplicate_section(&mut self, index: usize) {
        if index >= self.sections.len() {
            return;
        }
        let mut copy = self.sections[index].clone();
        copy.id = fresh_id();
        copy.is_locked = false;
        self.sections.insert(index + 1, copy);
    }
}

pub const CORE_GENRES: &[&str] = &[
    "Rock / Metal",
    "Pop / Electronic",
    "Hip-Hop / Rap",
    "Country / Americana",
    "Acoustic / Folk",
    "Jazz / Blues",
    "R&B / Soul",
    "Custom / Minimal",
];

pub const GROOVE_OPTIONS: &[&str] = &[
    "Straight",
    "Triplet / Shuffle",
    "Swing",
    "Half-Time",
    "Double-Time",
    "Syncopated",
];

pub const ADVANCED_GENRES: &[&str] = &[
    "8-Bar Blues",
    "12-Bar Blues",
    "16-Bar Blues",
    "Delta Blues",
    "Chicago Blues",
    "Texas Blues",
    "Piedmont Blues",
    "Jump Blues",
    "Soul Blues",
    "Blues Rock",
    "Bebop Jazz",
    "Cool Jazz",
    "Modal Jazz",
    "Free Jazz",
    "Hard Bop",
    "Fusion Jazz",
    "Gypsy Jazz",
    "Latin Jazz",
    "Smooth Jazz",
    "Ragtime",
    "Classic Rock",
    "Hard Rock",
    "Arena Rock",
    "Progressive Rock",
    "Psychedelic Rock",
    "Indie Rock",
    "Alternative Rock",
    "Post-Rock",
    "Grunge",
    "Shoegaze",
    "Punk Rock",
    "Pop Punk",
    "Post-Hardcore",
    "Heavy Metal",
    "Thrash Metal",
    "Death Metal",
    "Black Metal",
    "Doom Metal",
    "Power Metal",
    "Symphonic Metal",
    "Metalcore",
    "Deathcore",
    "Sludge Metal",
    "Stoner Rock",
    "Gothic Metal",
    "Math Rock",
    "Synth-Pop",
    "Dance-Pop",
    "Indie Pop",
    "Art Pop",
    "Electropop",
    "Dream Pop",
    "Baroque Pop",
    "House",
    "Deep House",
    "Tech House",
    "Progressive House",
    "Techno",
    "Minimal Techno",
    "Trance",
    "Psytrance",
    "Dubstep",
    "Drum and Bass",
    "Ambient",
    "Chillwave",
    "Synthwave",
    "Vaporwave",
    "IDM",
    "Hardstyle",
    "Future Bass",
    "Electro Swing",
    "Breakbeat",
    "Industrial",
    "Boom Bap",
    "Trap",
    "Melodic Trap",
    "Cloud Rap",
    "Drill",
    "UK Drill",
    "Conscious Hip-Hop",
    "Lo-Fi Hip-Hop",
    "G-Funk",
    "East Coast Rap",
    "West Coast Rap",
    "Southern Hip-Hop",
    "Alternative Hip-Hop",
    "Contemporary R&B",
    "Neo Soul",
    "Funk",
    "Disco",
    "Motown Soul",
    "Quiet Storm",
    "New Jack Swing",
    "Traditional Country",
    "Honky Tonk",
    "Outlaw Country",
    "Bluegrass",
    "Contemporary Country",
    "Country Pop",
    "Americana",
    "Folk Rock",
    "Contemporary Folk",
    "Traditional Folk",
    "Celtic Folk",
    "Bluegrass Gospel",
    "Western Swing",
    "Alt-Country",
    "Roots Rock",
    "Cajun / Zydeco",
    "Reggaeton",
    "Dancehall",
    "Roots Reggae",
    "Dub",
    "Ska",
    "Rocksteady",
    "Salsa",
    "Bachata",
    "Merengue",
    "Bossa Nova",
    "Samba",
    "Tango",
    "Mariachi",
    "Flamenco",
    "Afrobeats",
    "Amapiano",
    "Highlife",
    "K-Pop",
    "J-Pop",
    "C-Pop",
    "Bhangra",
    "Celtic",
    "Classical Symphony",
    "Baroque Concerto",
    "Romantic Sonata",
    "Opera",
    "Chamber Music",
    "Choral / Requiem",
    "Cinematic Orchestral",
    "Epic Trailer",
    "Minimalist Classical",
    "Ambient Score",
];

/// The same generated section lists as the Kotlin `GenreTemplates`.
pub fn generate_sections(genre: &str) -> Vec<SongSection> {
    let s = |t: SectionType, label: &str, ts: Option<&str>| {
        let mut sec = SongSection::new(t, label);
        sec.time_signature = ts.map(|x| x.to_string());
        sec
    };
    if genre == "Custom / Minimal" {
        vec![s(SectionType::Verse, "Verse 1", None)]
    } else if genre.contains("Blues") {
        vec![
            s(SectionType::Intro, "Intro (Turnaround)", Some("12/8")),
            s(SectionType::Verse, &format!("Verse 1 ({})", genre), None),
            s(SectionType::Verse, "Verse 2", None),
            s(SectionType::Solo, "Guitar/Harmonica Solo", None),
            s(SectionType::Verse, "Verse 3", None),
            s(SectionType::Outro, "Outro & Turnaround", None),
        ]
    } else if genre.contains("Jazz") {
        vec![
            s(SectionType::Intro, "Intro / Head In", Some("4/4")),
            s(SectionType::Verse, "A Section Theme", None),
            s(SectionType::Verse, "A Section Variation", None),
            s(SectionType::Bridge, "B Section (Bridge)", Some("3/4")),
            s(SectionType::Verse, "A Section Return", None),
            s(SectionType::Solo, "Improvisation Solos", None),
            s(SectionType::Outro, "Head Out / Coda", None),
        ]
    } else {
        vec![
            s(SectionType::Intro, "Intro", Some("4/4")),
            s(SectionType::Verse, "Verse 1", None),
            s(SectionType::Chorus, "Chorus", None),
            s(SectionType::Verse, "Verse 2", None),
            s(SectionType::Chorus, "Chorus", None),
            s(SectionType::Bridge, "Bridge", None),
            s(SectionType::Chorus, "Chorus", None),
            s(SectionType::Outro, "Outro", None),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_minimal_is_one_verse() {
        let secs = generate_sections("Custom / Minimal");
        assert_eq!(secs.len(), 1);
        assert_eq!(secs[0].section_type, SectionType::Verse);
    }

    #[test]
    fn default_template_is_eight_sections() {
        let secs = generate_sections("Rock / Metal");
        assert_eq!(secs.len(), 8);
        assert_eq!(secs[0].label, "Intro");
        assert_eq!(secs[2].section_type, SectionType::Chorus);
    }

    #[test]
    fn section_type_parse_falls_back_to_verse() {
        assert_eq!(SectionType::parse("BRIDGE"), SectionType::Bridge);
        assert_eq!(SectionType::parse("nonsense"), SectionType::Verse);
    }

    #[test]
    fn duplicate_section_copies_content_with_fresh_id_and_unlocked() {
        let mut song = Song::new("Test", "Rock / Metal");
        let mut section = SongSection::new(SectionType::Chorus, "My Chorus");
        section.lyrics = "hello world".to_string();
        section.chords = Some("C G Am F".to_string());
        section.time_signature = Some("4/4".to_string());
        section.is_locked = true;
        song.sections.push(section);
        let original_id = song.sections[0].id.clone();

        song.duplicate_section(0);

        assert_eq!(song.sections.len(), 2);
        // Original untouched.
        assert_eq!(song.sections[0].id, original_id);
        assert!(song.sections[0].is_locked);
        // Copy inserted right after, same content, fresh id, unlocked.
        let copy = &song.sections[1];
        assert_ne!(copy.id, original_id);
        assert!(!copy.id.is_empty());
        assert_eq!(copy.section_type, SectionType::Chorus);
        assert_eq!(copy.label, "My Chorus");
        assert_eq!(copy.lyrics, "hello world");
        assert_eq!(copy.chords.as_deref(), Some("C G Am F"));
        assert_eq!(copy.time_signature.as_deref(), Some("4/4"));
        assert!(!copy.is_locked);
    }

    #[test]
    fn duplicate_section_out_of_range_does_nothing() {
        let mut song = Song::new("Test", "Rock / Metal");
        song.sections
            .push(SongSection::new(SectionType::Verse, "V"));
        song.duplicate_section(5);
        assert_eq!(song.sections.len(), 1);
    }

    #[test]
    fn duplicate_section_keeps_ids_unique_across_many_copies() {
        let mut song = Song::new("Test", "Rock / Metal");
        song.sections
            .push(SongSection::new(SectionType::Verse, "V"));
        for _ in 0..5 {
            song.duplicate_section(0);
        }
        let mut ids: Vec<&str> = song.sections.iter().map(|s| s.id.as_str()).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), 6);
    }
}
