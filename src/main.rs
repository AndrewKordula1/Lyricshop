#![windows_subsystem = "windows"]

//! Lyricshop — a lyrics and songwriting-structure tool.
//!
//! Chord playback is a listening aid while writing, not a music-writing
//! tool: it synthesizes each chord as mixed sine waves through rodio and
//! loops until stopped. Transpose is playback-only.

mod app;
mod chord_suggestions;
mod chords;
mod icons;
mod library;
mod pdf;
mod player;
mod prefs;
mod rhyme;
mod rhyme_scheme;
mod serialization;
mod snapshots;
mod song;
mod storage;
mod syllables;
mod theme;
mod ui;
mod wordcount;

pub use app::LyricshopApp;
use app::{SETUP_WINDOW_HEIGHT, SETUP_WINDOW_WIDTH};

/// Load the app icon (`assets/icon.png`, "Ls" on a red diagonal
/// gradient) as window icon data. Returns `None` if the embedded PNG
/// cannot be decoded, in which case eframe falls back to its default.
fn load_icon() -> Option<std::sync::Arc<egui::IconData>> {
    let bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(bytes).ok()?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    Some(std::sync::Arc::new(egui::IconData {
        rgba: rgba.into_raw(),
        width,
        height,
    }))
}

fn main() -> eframe::Result<()> {
    // The app always launches on the setup screen, which is a small
    // fixed dialog sized to the welcome card — not the large editor
    // window. When the user enters the editor, LyricshopApp::update
    // restores the large resizable window (see desired_viewport).
    let mut viewport = egui::ViewportBuilder::default()
        .with_inner_size([SETUP_WINDOW_WIDTH, SETUP_WINDOW_HEIGHT])
        .with_resizable(false)
        .with_title("Lyricshop");
    if let Some(icon) = load_icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "Lyricshop",
        options,
        Box::new(|_cc| Ok(Box::new(LyricshopApp::new()))),
    )
}
