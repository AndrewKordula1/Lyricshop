//! Lyricshop application: egui/eframe entry point and main window.
//!
//! Layout mirrors the Kotlin app: a top menu bar (File, View, plus the
//! waveform and transpose selectors), a header with the song metadata,
//! the section cards in a scrolling list, and a status bar. A setup
//! dialog opens first for a new song. The rhyme assistant is a side panel.

use crate::library::{list_library, LibraryEntry};
use crate::player::{transpose_label, ChordPlayer, Timbre};
use crate::prefs::{load_preferences, save_preferences, Preferences};
use crate::snapshots::{display_name, list_snapshots, save_snapshot};
use crate::song::{SectionType, Song, CORE_GENRES};
use crate::storage::SongFile;
use crate::theme::{apply_theme, Theme};
use crate::wordcount::count_words_total;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

pub struct LyricshopApp {
    pub song: Song,
    pub song_file: SongFile,
    pub prefs: Preferences,
    pub theme: Theme,

    // Setup screen.
    pub show_setup: bool,
    pub setup_title: String,
    pub setup_genre: String,
    pub show_advanced_genres: bool,
    pub advanced_genre_search: String,

    // Dirty tracking + autosave.
    pub dirty: bool,
    pub edit_version: u64,
    pub last_edit: Option<Instant>,

    // Status line.
    pub status: String,
    pub status_until: Option<Instant>,

    // Rhyme assistant.
    pub show_rhyme: bool,
    pub rhyme_query: String,
    pub rhyme_mode: crate::rhyme::RhymeMode,
    pub rhyme_results: Vec<String>,
    pub last_edited_section_id: Option<String>,
    // Syllable-count filter for the displayed rhyme results: None = Any,
    // Some(1..=4) = exactly that many syllables, Some(5) = 5 or more.
    // Display-only; it never changes the ranking or the query, and it
    // persists when the panel is hidden and re-shown.
    pub rhyme_syllable_filter: Option<usize>,

    // Dialogs.
    pub show_add_section: bool,
    pub add_type: SectionType,
    pub add_label: String,
    pub show_snapshots: bool,
    pub snapshot_entries: Vec<SnapshotEntry>,
    pub show_library: bool,
    pub library_query: String,
    pub library_entries: Vec<LibraryEntry>,
    // Find & replace (editor only). `find_replace_message` is the
    // dialog's own status line ("3 matches" / "Replaced 3"); it clears
    // whenever the find or replace text changes.
    pub show_find_replace: bool,
    pub find_text: String,
    pub replace_text: String,
    pub find_replace_message: String,
    // About dialog.
    pub show_about: bool,

    // Playback + header state.
    pub player: ChordPlayer,
    pub bpm_text: String,
    pub progression_index: HashMap<String, usize>,

    // Close confirm.
    pub show_close_confirm: bool,
    pub close_confirmed: bool,

    // Viewport bookkeeping: the setup screen is a small fixed dialog;
    // the editor is a large resizable window. Remembers the last
    // (width, height, resizable) sent to the OS so viewport commands
    // are only sent when the desired configuration changes.
    last_viewport: Option<(f32, f32, bool)>,
}

#[derive(Debug, Clone)]
pub struct SnapshotEntry {
    pub file: PathBuf,
    pub title: String,
    pub display_date: String,
}

impl Default for LyricshopApp {
    fn default() -> Self {
        let prefs = load_preferences();
        let theme = prefs.theme();
        LyricshopApp {
            song: Song::new("New Song", CORE_GENRES[0]),
            song_file: SongFile::default(),
            prefs,
            theme,
            show_setup: true,
            setup_title: "New Song".to_string(),
            setup_genre: CORE_GENRES[0].to_string(),
            show_advanced_genres: false,
            advanced_genre_search: String::new(),
            dirty: false,
            edit_version: 0,
            last_edit: None,
            status: String::new(),
            status_until: None,
            show_rhyme: false,
            rhyme_query: String::new(),
            rhyme_mode: crate::rhyme::RhymeMode::Perfect,
            rhyme_results: Vec::new(),
            last_edited_section_id: None,
            rhyme_syllable_filter: None,
            show_add_section: false,
            add_type: SectionType::Verse,
            add_label: SectionType::Verse.default_name().to_string(),
            show_snapshots: false,
            snapshot_entries: Vec::new(),
            show_library: false,
            library_query: String::new(),
            library_entries: Vec::new(),
            show_find_replace: false,
            find_text: String::new(),
            replace_text: String::new(),
            find_replace_message: String::new(),
            show_about: false,
            player: ChordPlayer::new(),
            bpm_text: "120".to_string(),
            progression_index: HashMap::new(),
            show_close_confirm: false,
            close_confirmed: false,
            last_viewport: None,
        }
    }
}

/// The editor window: the normal large resizable window.
pub(crate) const EDITOR_WINDOW_WIDTH: f32 = 1100.0;
pub(crate) const EDITOR_WINDOW_HEIGHT: f32 = 750.0;

/// The setup window is a small fixed dialog sized to the welcome card:
/// the 320-wide form column, 12px of card padding on each side, and
/// 8px of panel margin on each side. Taller when the advanced-genre
/// list is open — the list adds its 160px height plus one row of
/// item spacing. Heights are measured from the setup layout (see
/// `show_setup_dialog`); if the card ever grows, these need to grow
/// with it. The width fits the advanced-genre list: 640-wide list +
/// 12px card padding + 8px panel margin per side.
pub(crate) const SETUP_WINDOW_WIDTH: f32 = 680.0;
pub(crate) const SETUP_WINDOW_HEIGHT: f32 = 435.0;
pub(crate) const SETUP_WINDOW_HEIGHT_ADVANCED: f32 = 635.0;

/// Desired viewport for the current screen: (size, resizable). The
/// setup screen is a small fixed dialog; the editor is the large
/// resizable window. A pure function of the screen state, so it is
/// unit-testable.
fn desired_viewport(show_setup: bool, show_advanced_genres: bool) -> (egui::Vec2, bool) {
    if show_setup {
        let height = if show_advanced_genres {
            SETUP_WINDOW_HEIGHT_ADVANCED
        } else {
            SETUP_WINDOW_HEIGHT
        };
        (egui::vec2(SETUP_WINDOW_WIDTH, height), false)
    } else {
        (egui::vec2(EDITOR_WINDOW_WIDTH, EDITOR_WINDOW_HEIGHT), true)
    }
}

impl LyricshopApp {
    pub fn new() -> Self {
        LyricshopApp::default()
    }

    /// Mark the song changed (dirty + autosave clock). Mirrors the Kotlin
    /// `markChanged()` / `onContentChanged()`.
    pub fn mark_changed(&mut self) {
        self.edit_version += 1;
        self.dirty = true;
        self.last_edit = Some(Instant::now());
    }

    pub fn show_status(&mut self, message: impl Into<String>) {
        self.status = message.into();
        self.status_until = Some(Instant::now() + Duration::from_secs(4));
    }

    fn set_theme(&mut self, theme: Theme, ctx: &egui::Context) {
        self.theme = theme;
        self.prefs.set_theme(theme);
        save_preferences(&self.prefs);
        apply_theme(ctx, theme);
    }

    /// Start a new song (Ctrl+N): save first if there is a file and the
    /// song is dirty; never wipe unsaved work that has nowhere to go.
    fn new_song(&mut self) {
        if self.dirty {
            if !self.song_file.has_current_file() {
                return;
            }
            if !self.song_file.save_song(&self.song) {
                return;
            }
        }
        self.song = Song::new("New Song", CORE_GENRES[0]);
        self.song_file.clear();
        self.setup_title = "New Song".to_string();
        self.setup_genre = CORE_GENRES[0].to_string();
        self.bpm_text = "120".to_string();
        self.dirty = false;
        self.edit_version = 0;
        self.last_edit = None;
        self.show_setup = true;
    }

    pub(crate) fn refresh_snapshots(&mut self) {
        let dir_file = self.song_file.current.clone();
        self.snapshot_entries = list_snapshots(dir_file.as_deref())
            .into_iter()
            .map(|file| {
                let title = std::fs::read_to_string(&file)
                    .ok()
                    .and_then(|t| crate::serialization::decode_song(&t))
                    .map(|s| {
                        if s.title.trim().is_empty() {
                            "Untitled Song".to_string()
                        } else {
                            s.title
                        }
                    })
                    .unwrap_or_else(|| "Untitled Song".to_string());
                SnapshotEntry {
                    display_date: display_name(&file),
                    file,
                    title,
                }
            })
            .collect();
    }

    fn refresh_library(&mut self) {
        let folder = self
            .song_file
            .current
            .as_ref()
            .and_then(|f| f.parent().map(|p| p.to_path_buf()));
        self.library_entries = list_library(folder.as_deref());
    }
}

impl eframe::App for LyricshopApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply the theme on the first frame (and whenever it changes).
        // eframe gives us a fresh context each frame, so set visuals
        // every frame is cheap and always correct.
        apply_theme(ctx, self.theme);

        // --- Viewport: the setup screen is a small fixed dialog
        // (sized to the welcome card, not resizable); the editor is
        // the normal large resizable window. Commands are only sent
        // when the desired configuration changes, so the OS is not
        // spammed every frame. This also covers returning to the
        // setup screen (Ctrl+N) and toggling the advanced-genre list,
        // which grows the dialog by the list's height.
        let (size, resizable) = desired_viewport(self.show_setup, self.show_advanced_genres);
        if self.last_viewport != Some((size.x, size.y, resizable)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            ctx.send_viewport_cmd(egui::ViewportCommand::Resizable(resizable));
            self.last_viewport = Some((size.x, size.y, resizable));
        }

        // --- Close request: eframe 0.32 has no on_close_event hook; the
        // documented way is to check viewport().close_requested() and
        // answer with CancelClose. If there are unsaved changes, cancel
        // the close and ask first.
        if ctx.input(|i| i.viewport().close_requested()) && self.dirty && !self.close_confirmed {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_close_confirm = true;
        }

        // --- Global shortcuts: Ctrl/Cmd+S save, Ctrl/Cmd+O open,
        // Ctrl/Cmd+N new song, Ctrl/Cmd+H find & replace. Checked
        // before the UI consumes keys, so they work even while a text
        // field has focus.
        let mut do_save = false;
        let mut do_open = false;
        let mut do_new = false;
        let mut do_find_replace = false;
        ctx.input(|i| {
            if i.modifiers.command {
                if i.key_pressed(egui::Key::S) {
                    do_save = true;
                } else if i.key_pressed(egui::Key::O) {
                    do_open = true;
                } else if i.key_pressed(egui::Key::N) {
                    do_new = true;
                } else if i.key_pressed(egui::Key::H) {
                    do_find_replace = true;
                }
            }
        });
        if do_save && !self.show_setup {
            if self.song_file.save_song(&self.song) {
                self.dirty = false;
                self.show_status("Saved");
            } else {
                self.show_status("Save failed");
            }
        }
        if do_open && !self.show_setup {
            if self.song_file.open_song(&mut self.song) {
                self.dirty = false;
                self.bpm_text = self.song.bpm.to_string();
                self.show_status(format!("Opened \"{}\"", self.song.title));
            } else {
                self.show_status("Could not open file");
            }
        }
        if do_new {
            self.new_song();
        }
        if do_find_replace && !self.show_setup {
            self.show_find_replace = true;
        }

        // --- Autosave: 2 seconds after the last edit, write to the
        // already-known file. Nothing happens before the first manual
        // save, because there is no file yet.
        if self.dirty {
            if let Some(last) = self.last_edit {
                if last.elapsed() >= Duration::from_secs(2) {
                    if self.song_file.autosave(&self.song) {
                        self.dirty = false;
                        self.show_status("Autosaved");
                    }
                    // Reset the clock so a failed autosave does not spin.
                    self.last_edit = Some(Instant::now());
                }
            }
        }

        // --- Status messages clear themselves after a few seconds.
        if let Some(until) = self.status_until {
            if Instant::now() >= until {
                self.status.clear();
                self.status_until = None;
            }
        }

        // --- Menu bar (File, View, plus waveform/transpose). Hidden on
        // the start screen: that screen is only for opening a file or
        // starting a new song, so it needs no editor chrome.
        if !self.show_setup {
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                let muted = self.theme.palette().text_muted;
                // Menu bar items use the same 30px button sizing as the
                // rest of the app. egui's default menu style shrinks the
                // button padding to (2, 0), which would leave the menu
                // bar short while the other buttons are 30px tall.
                egui::MenuBar::new()
                    .style(|style: &mut egui::Style| {
                        style.spacing.button_padding = egui::vec2(5.0, 6.0);
                        style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
                        style.visuals.widgets.open.bg_stroke = egui::Stroke::NONE;
                        style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                        style.visuals.widgets.inactive.weak_bg_fill = egui::Color32::TRANSPARENT;
                        style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                    })
                    .ui(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("Save\t\tCtrl+S").clicked() {
                                if self.song_file.save_song(&self.song) {
                                    self.dirty = false;
                                    self.show_status("Saved");
                                } else {
                                    self.show_status("Save failed");
                                }
                                ui.close();
                            }
                            if ui.button("Save As...").clicked() {
                                if self.song_file.save_song_as(&self.song) {
                                    self.dirty = false;
                                    self.show_status("Saved");
                                } else {
                                    self.show_status("Save failed");
                                }
                                ui.close();
                            }
                            if ui.button("Export to TXT...").clicked() {
                                if self.song_file.export_to_txt(&self.song) {
                                    self.show_status("Exported to TXT");
                                } else {
                                    self.show_status("Export failed");
                                }
                                ui.close();
                            }
                            if ui.button("Open...\t\tCtrl+O").clicked() {
                                if self.song_file.open_song(&mut self.song) {
                                    self.dirty = false;
                                    self.bpm_text = self.song.bpm.to_string();
                                    self.show_status(format!("Opened \"{}\"", self.song.title));
                                } else {
                                    self.show_status("Could not open file");
                                }
                                ui.close();
                            }
                            ui.separator();
                            if ui.button("Save Snapshot").clicked() {
                                if save_snapshot(&self.song, self.song_file.current.as_deref()) {
                                    self.show_status("Snapshot saved");
                                } else {
                                    self.show_status(
                                        "Save the song to a file first to use snapshots",
                                    );
                                }
                                ui.close();
                            }
                            if ui.button("Restore Snapshot...").clicked() {
                                self.refresh_snapshots();
                                self.show_snapshots = true;
                                ui.close();
                            }
                            if ui.button("Export to PDF...").clicked() {
                                if self.song_file.export_to_pdf(&self.song) {
                                    self.show_status("Exported to PDF");
                                } else {
                                    self.show_status("Export failed");
                                }
                                ui.close();
                            }
                            if ui.button("Song Library...").clicked() {
                                self.refresh_library();
                                self.show_library = true;
                                ui.close();
                            }
                            ui.separator();
                            // Quit goes through the normal close flow: the
                            // Close command raises close_requested(), and the
                            // update() check cancels it and shows the confirm
                            // dialog when there are unsaved changes.
                            if ui.button("Quit").clicked() {
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                ui.close();
                            }
                        });
                        ui.menu_button("Edit", |ui| {
                            if ui.button("Find & Replace...\t\tCtrl+H").clicked() {
                                self.show_find_replace = true;
                                ui.close();
                            }
                        });
                        ui.menu_button("View", |ui| {
                            ui.menu_button("Theme", |ui| {
                                for theme in Theme::all() {
                                    let selected = self.theme == *theme;
                                    if ui.selectable_label(selected, theme.label()).clicked() {
                                        self.set_theme(*theme, ctx);
                                        ui.close();
                                    }
                                }
                            });
                            ui.separator();
                            // Rhyme dictionary: toggles the rhyme side panel
                            // (same panel as the Rhymes button in the header).
                            if ui
                                .selectable_label(self.show_rhyme, "Rhyme Dictionary")
                                .clicked()
                            {
                                self.show_rhyme = !self.show_rhyme;
                                ui.close();
                            }
                        });
                        ui.menu_button("Help", |ui| {
                            if ui.button("About Lyricshop...").clicked() {
                                self.show_about = true;
                                ui.close();
                            }
                        });

                        // Playback controls, in the menu bar: the waveform and
                        // transpose selectors (listening preferences, not saved).
                        // They used to live in the header; chord playback itself
                        // stays on each section's Play button. The waveform
                        // selector lists the three waveforms the synth actually
                        // produces (sine, triangle, sawtooth) — not instrument
                        // names, since there are no sampled instruments here.
                        // Each ComboBox sits in its own top-aligned 30px box:
                        // a ComboBox wraps its button in a second inner
                        // horizontal layout, and in the menu bar's centered
                        // row that drifts it 6px down below the File/Edit/
                        // View/Help buttons (the same offset the header
                        // dropdowns had). Top-aligning the box pins the
                        // button to the same top edge as the menu buttons.
                        // The box widths fit the longest option ("Sawtooth
                        // Wave" is ~135px; "-2 (D std.)" is ~100px).
                        ui.separator();
                        ui.label(egui::RichText::new("Waveform:").color(muted));
                        ui.allocate_ui_with_layout(
                            egui::vec2(140.0, 30.0),
                            egui::Layout::left_to_right(egui::Align::Min),
                            |ui| {
                                egui::ComboBox::from_id_salt("waveform")
                                    .selected_text(self.player.timbre.name())
                                    .show_ui(ui, |ui| {
                                        for timbre in Timbre::ALL {
                                            if ui
                                                .selectable_label(
                                                    self.player.timbre == timbre,
                                                    timbre.name(),
                                                )
                                                .clicked()
                                            {
                                                self.player.timbre = timbre;
                                            }
                                        }
                                    });
                            },
                        );
                        ui.label(egui::RichText::new("Transpose:").color(muted));
                        ui.allocate_ui_with_layout(
                            egui::vec2(105.0, 30.0),
                            egui::Layout::left_to_right(egui::Align::Min),
                            |ui| {
                                egui::ComboBox::from_id_salt("transpose")
                                    .selected_text(transpose_label(self.player.transpose_semitones))
                                    .show_ui(ui, |ui| {
                                        for t in -6..=6 {
                                            if ui
                                                .selectable_label(
                                                    self.player.transpose_semitones == t,
                                                    transpose_label(t),
                                                )
                                                .clicked()
                                            {
                                                self.player.transpose_semitones = t;
                                            }
                                        }
                                    });
                            },
                        );
                    });
            });
        }

        // --- Status bar. Also hidden on the start screen: before a song
        // exists its "0 words · 0 sections" is meaningless, and the
        // welcome screen reads cleaner without the editor chrome.
        if !self.show_setup {
            egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let muted = self.theme.palette().text_muted;
                    if !self.status.is_empty() {
                        ui.label(egui::RichText::new(&self.status).small());
                        ui.separator();
                    }
                    let save_state = if !self.song_file.has_current_file() {
                        "Not saved to a file yet"
                    } else if self.dirty {
                        "● Unsaved changes"
                    } else {
                        "All changes saved"
                    };
                    ui.label(egui::RichText::new(save_state).small());
                    ui.separator();
                    ui.label(
                        egui::RichText::new(format!(
                            "{} words · {} sections",
                            count_words_total(&self.song.sections),
                            self.song.sections.len()
                        ))
                        .small()
                        .color(muted),
                    );
                });
            });
        }

        // --- Setup dialog (first screen for a new song).
        if self.show_setup {
            self.show_setup_dialog(ctx);
        } else {
            self.show_editor(ctx);
        }

        // --- Dialogs.
        if self.show_add_section {
            self.show_add_section_dialog(ctx);
        }
        if self.show_find_replace {
            self.show_find_replace_dialog(ctx);
        }
        if self.show_snapshots {
            self.show_snapshot_dialog(ctx);
        }
        if self.show_library {
            self.show_library_dialog(ctx);
        }
        if self.show_about {
            self.show_about_dialog(ctx);
        }
        if self.show_close_confirm {
            self.show_close_confirm_dialog(ctx);
        }

        // Keep the autosave clock ticking (repaint so the 2s timer fires).
        if self.dirty {
            ctx.request_repaint_after(Duration::from_millis(250));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_is_small_and_fixed_on_setup() {
        let (size, resizable) = desired_viewport(true, false);
        assert_eq!(size, egui::vec2(680.0, 435.0));
        assert!(!resizable);
    }

    #[test]
    fn viewport_grows_for_the_advanced_genre_list() {
        let (size, resizable) = desired_viewport(true, true);
        assert_eq!(size, egui::vec2(680.0, 635.0));
        assert!(!resizable);
    }

    #[test]
    fn viewport_is_large_and_resizable_in_the_editor() {
        let (size, resizable) = desired_viewport(false, false);
        assert_eq!(size, egui::vec2(1100.0, 750.0));
        assert!(resizable);
    }
}
