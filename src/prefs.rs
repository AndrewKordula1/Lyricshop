//! App preferences, persisted to a small JSON file so choices survive
//! restarts. Currently just the theme; the file lives in the platform
//! config dir (`~/.config/lyricshop/preferences.json` on Linux/macOS,
//! `%APPDATA%\Lyricshop\preferences.json` on Windows).
//!
//! The waveform and transpose are listening preferences like in the
//! Kotlin app — they are kept in memory for the session, not saved with
//! the song file. (They are also not persisted here, matching the Kotlin
//! app, where they reset to Sine Wave / Standard on launch.)

use crate::theme::Theme;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preferences {
    #[serde(default = "default_theme_id")]
    pub theme: String,
}

fn default_theme_id() -> String {
    "dark".to_string()
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            theme: "dark".to_string(),
        }
    }
}

impl Preferences {
    pub fn theme(&self) -> Theme {
        Theme::parse(&self.theme)
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme.id().to_string();
    }
}

/// The preferences file path for this machine.
pub fn prefs_path() -> Option<PathBuf> {
    #[cfg(windows)]
    {
        std::env::var("APPDATA")
            .ok()
            .map(|a| PathBuf::from(a).join("Lyricshop").join("preferences.json"))
    }
    #[cfg(not(windows))]
    {
        std::env::var("HOME").ok().map(|h| {
            PathBuf::from(h)
                .join(".config")
                .join("lyricshop")
                .join("preferences.json")
        })
    }
}

pub fn load_preferences() -> Preferences {
    let Some(path) = prefs_path() else {
        return Preferences::default();
    };
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).unwrap_or_default(),
        Err(_) => Preferences::default(),
    }
}

pub fn save_preferences(prefs: &Preferences) {
    let Some(path) = prefs_path() else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(prefs) {
        let _ = std::fs::write(&path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_round_trips_through_json() {
        let mut p = Preferences::default();
        assert_eq!(p.theme(), Theme::Dark);
        p.set_theme(Theme::Light);
        let text = serde_json::to_string(&p).unwrap();
        let back: Preferences = serde_json::from_str(&text).unwrap();
        assert_eq!(back.theme(), Theme::Light);
    }

    #[test]
    fn unknown_theme_id_falls_back_to_dark() {
        let p: Preferences = serde_json::from_str(r#"{"theme":"neon"}"#).unwrap();
        assert_eq!(p.theme(), Theme::Dark);
    }

    #[test]
    fn missing_theme_field_defaults_to_dark() {
        let p: Preferences = serde_json::from_str("{}").unwrap();
        assert_eq!(p.theme(), Theme::Dark);
    }
}
