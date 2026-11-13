//! The editor screen: header, section cards, rhyme panel, and dialogs.
//! Methods on `LyricshopApp` (the struct lives in `app.rs`).

use crate::chord_suggestions::{diatonic_chords, parse_key, suggested_progressions};
use crate::chords::parse_progression;
use crate::icons::{draw_icon_at_left, icon_button, Icon};
use crate::library::filter_library;
use crate::rhyme::{
    find_rhymes, syllable_count_for, syllable_count_matches_filter, syllable_filter_label,
    RhymeMode, SYLLABLE_FILTER_OPTIONS,
};
use crate::rhyme_scheme::rhyme_scheme_label;
use crate::snapshots::{delete_snapshot, load_snapshot};
use crate::song::{
    generate_sections, SectionType, Song, SongSection, ADVANCED_GENRES, CORE_GENRES,
};
use crate::syllables::analyze_meter;
use crate::wordcount::count_words;
use crate::LyricshopApp;

/// A text label vertically centered in a row of `height` px, sized to its
/// content. The header and section-card rows are top-aligned (a centered
/// horizontal row lets later controls drift down), so a plain label would
/// sit at the top of the row; this wrapper centers it next to the 28/30px
/// fields and buttons instead. The width is measured from the text (plus
/// a little slack, since strong text renders slightly wider than the
/// measured regular weight), so the label never clips.
fn centered_label(
    ui: &mut egui::Ui,
    text: &str,
    font_id: egui::FontId,
    color: egui::Color32,
    height: f32,
    strong: bool,
) {
    let w = ui.fonts(|f| {
        f.layout_no_wrap(text.to_string(), font_id.clone(), egui::Color32::WHITE)
            .size()
            .x
    }) + 4.0;
    ui.allocate_ui_with_layout(
        egui::vec2(w, height),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            let mut rt = egui::RichText::new(text).font(font_id).color(color);
            if strong {
                rt = rt.strong();
            }
            ui.label(rt);
        },
    );
}

impl LyricshopApp {
    // ------------------------------------------------------------------
    // Setup dialog (first screen for a new song).
    // ------------------------------------------------------------------
    pub fn show_setup_dialog(&mut self, ctx: &egui::Context) {
        let pal = self.theme.palette();
        // The panel fill is the card color, not the background color:
        // the setup window is sized to the card, so the panel is the
        // card — there is no visible seam between the two even if a
        // few pixels of margin remain around the frame.
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(egui::Margin::same(8))
                    .fill(pal.card),
            )
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    // No extra space above the card: the panel's 8px
                    // margin is the only gap, so the window fits the
                    // card snugly with no empty band at the top.
                    // The welcome form sits in a card surface, exactly like
                    // the section cards in the editor: same card fill, 1px
                    // border, 4px radius, 12px padding. The form column is
                    // 320 wide (matching the advanced-genre list) so the
                    // card sizes to its content and stays centered.
                    let card_frame = egui::Frame::group(ui.style())
                        .fill(pal.card)
                        .stroke(egui::Stroke::new(1.0_f32, pal.border))
                        .corner_radius(egui::CornerRadius::same(4))
                        .inner_margin(egui::Margin::same(12));
                    card_frame.show(ui, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.heading("Lyricshop");
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new(
                                    "Start a new song, or open one you already saved.",
                                )
                                .color(pal.text_muted),
                            );
                            ui.add_space(24.0);

                            ui.label(
                                egui::RichText::new("Song title")
                                    .size(15.6)
                                    .color(pal.text_muted),
                            );
                            // 30px tall like the Genre dropdown and the
                            // buttons below, text centered and at the
                            // 15.6pt button-label size so it fills the
                            // field — the same treatment as the Title
                            // field in the editor header.
                            ui.add_sized(
                                egui::vec2(320.0, 30.0),
                                egui::TextEdit::singleline(&mut self.setup_title)
                                    .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                                    .horizontal_align(egui::Align::Center)
                                    .vertical_align(egui::Align::Center),
                            );
                            ui.add_space(16.0);

                            ui.label(
                                egui::RichText::new("Genre")
                                    .size(15.6)
                                    .color(pal.text_muted),
                            );
                            // The ComboBox draws itself in an internal
                            // full-width horizontal strip, so it will not
                            // center on its own — confine it to a fixed-size
                            // block that vertical_centered can center. The
                            // block is top-aligned: a centered block lets
                            // the ComboBox drift 6px down inside its own
                            // 30px, the same offset the header dropdowns
                            // had before the top-aligned row fix.
                            ui.allocate_ui_with_layout(
                                egui::vec2(220.0, 30.0),
                                egui::Layout::left_to_right(egui::Align::Min),
                                |ui| {
                                    egui::ComboBox::from_id_salt("setup_genre")
                                        .width(220.0)
                                        .selected_text(&self.setup_genre)
                                        .show_ui(ui, |ui| {
                                            for genre in CORE_GENRES {
                                                ui.selectable_value(
                                                    &mut self.setup_genre,
                                                    genre.to_string(),
                                                    *genre,
                                                );
                                            }
                                        });
                                },
                            );
                            ui.add_space(8.0);
                            if ui
                                .button("Browse Advanced Genres (100+ Styles)...")
                                .clicked()
                            {
                                self.show_advanced_genres = !self.show_advanced_genres;
                            }
                            if self.show_advanced_genres {
                                // The ScrollArea anchors itself at the
                                // left edge of the available rect — it
                                // ignores the centered layout (see
                                // ScrollArea::begin: inner_rect starts
                                // at available_outer.min) — so confine
                                // it to a fixed-size block that
                                // vertical_centered can center, the
                                // same trick as the Genre ComboBox
                                // above. The scroll bar stays always
                                // visible (not just on hover) so it is
                                // obvious there are more genres to
                                // scroll to, and the list is a fixed
                                // width so the bar sits right next to
                                // the selections instead of at the
                                // window edge. The list is 640 wide —
                                // twice the 320-wide form — so it fills
                                // the window instead of sitting in one
                                // half of it. The search box above the
                                // list filters the 141 genres as you
                                // type (case-insensitive substring);
                                // empty search shows them all.
                                ui.allocate_ui_with_layout(
                                    egui::vec2(640.0, 192.0),
                                    egui::Layout::top_down(egui::Align::LEFT),
                                    |ui| {
                                        // 30px tall like the other fields,
                                        // text centered and at the 15.6pt
                                        // button-label size.
                                        ui.add_sized(
                                            egui::vec2(640.0, 30.0),
                                            egui::TextEdit::singleline(
                                                &mut self.advanced_genre_search,
                                            )
                                            .font(egui::FontId::new(
                                                15.6,
                                                egui::FontFamily::Proportional,
                                            ))
                                            .horizontal_align(egui::Align::Center)
                                            .vertical_align(egui::Align::Center)
                                            .hint_text("Search genres..."),
                                        );
                                        ui.add_space(8.0);
                                        let query = self
                                            .advanced_genre_search
                                            .trim()
                                            .to_lowercase();
                                        let mut shown = 0;
                                        egui::ScrollArea::vertical()
                                            .max_height(160.0)
                                            .max_width(640.0)
                                            .scroll_bar_visibility(
                                                egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                            )
                                            .show(ui, |ui| {
                                                for genre in ADVANCED_GENRES.iter().filter(|g| {
                                                    query.is_empty()
                                                        || g.to_lowercase().contains(&query)
                                                }) {
                                                    shown += 1;
                                                    if ui
                                                        .selectable_label(
                                                            self.setup_genre == *genre,
                                                            *genre,
                                                        )
                                                        .clicked()
                                                    {
                                                        self.setup_genre = genre.to_string();
                                                    }
                                                }
                                                if shown == 0 {
                                                    ui.label(
                                                        egui::RichText::new("No genres match.")
                                                            .color(pal.text_muted),
                                                    );
                                                }
                                            });
                                    },
                                );
                            }

                            ui.add_space(24.0);
                            let mut start = false;
                            let mut open = false;
                            // Stacked, not side by side: a horizontal row
                            // takes the full width in egui (ui.horizontal and
                            // horizontal_centered both left-align their
                            // contents), so the only way to get the buttons
                            // truly centered is to add each one on its own
                            // line — vertical_centered centers each widget.
                            if ui.button("Start Writing").clicked() {
                                start = true;
                            }
                            ui.add_space(8.0);
                            if ui.button("Open File...").clicked() {
                                open = true;
                            }
                            if open {
                                if self.song_file.open_song(&mut self.song) {
                                    self.dirty = false;
                                    self.bpm_text = self.song.bpm.to_string();
                                    self.setup_title = self.song.title.clone();
                                    self.setup_genre = self.song.genre.clone();
                                    self.show_setup = false;
                                    self.show_status(format!("Opened \"{}\"", self.song.title));
                                } else {
                                    self.show_status("Could not open file");
                                }
                            }
                            if start {
                                let title = self.setup_title.trim().to_string();
                                let title = if title.is_empty() {
                                    "New Song".to_string()
                                } else {
                                    title
                                };
                                self.song = Song::new(&title, &self.setup_genre);
                                self.song.sections = generate_sections(&self.setup_genre);
                                self.song_file.clear();
                                self.bpm_text = "120".to_string();
                                self.dirty = false;
                                self.edit_version = 0;
                                self.last_edit = None;
                                self.show_setup = false;
                            }
                        });
                    });
                });
            });
    }

    // ------------------------------------------------------------------
    // Editor screen: header + scrolling section list + rhyme panel.
    // ------------------------------------------------------------------
    pub fn show_editor(&mut self, ctx: &egui::Context) {
        self.show_header(ctx);
        if self.show_rhyme {
            self.show_rhyme_panel(ctx);
        }
        let bg = self.theme.palette().bg;
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .inner_margin(egui::Margin::same(8))
                    .fill(bg),
            )
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let count = self.song.sections.len();
                    for i in 0..count {
                        // Each card gets its own ID scope, so the repeated
                        // icon buttons (play, lock, move, delete) and text
                        // fields don't clash across sections — egui needs
                        // every widget's ID to be unique.
                        let section_id = self.song.sections[i].id.clone();
                        let deleted = ui
                            .push_id(section_id, |ui| self.show_section_card(ui, i))
                            .inner;
                        if deleted {
                            // A section was deleted; indices shifted, so stop
                            // drawing cards this frame.
                            break;
                        }
                        ui.add_space(12.0);
                    }
                    if ui
                        .button("＋ Add Section")
                        .on_hover_text("Add a new section to the song")
                        .clicked()
                    {
                        self.add_label = SectionType::Verse.default_name().to_string();
                        self.add_type = SectionType::Verse;
                        self.show_add_section = true;
                    }
                });
            });
    }

    fn show_header(&mut self, ctx: &egui::Context) {
        let pal = self.theme.palette();
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            // Top-aligned row. A centered horizontal row centers each
            // control against the row height *at the time it is placed*,
            // so controls added later drift down — the Genre/Groove
            // ComboBoxes (which wrap their button in a second inner
            // horizontal layout) sat 6px lower than the Title/Key/BPM
            // fields and stretched the bar to 36px. Top-aligning pins
            // every 30px control to the same top edge. The small labels
            // are wrapped in their own 30px centered boxes so they still
            // read as vertically centered next to the fields.
            ui.horizontal_top(|ui| {
                // A label vertically centered in the 30px row (see
                // `centered_label` above).
                let vcenter_label = |ui: &mut egui::Ui, text: &str| {
                    centered_label(
                        ui,
                        text,
                        egui::TextStyle::Body.resolve(ui.style()),
                        pal.text_muted,
                        30.0,
                        false,
                    );
                };
                // Title.
                vcenter_label(ui, "Title:");
                let title_changed = ui
                    .add_enabled_ui(!self.song.is_metadata_locked, |ui| {
                        let mut title = self.song.title.clone();
                        let resp = ui.add_sized(
                            egui::vec2(180.0, 30.0),
                            egui::TextEdit::singleline(&mut title)
                                // Same size as the button labels (15.6pt),
                                // so the text fills the 30px field.
                                .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center),
                        );
                        if resp.changed() {
                            self.song.title = title;
                            true
                        } else {
                            false
                        }
                    })
                    .inner;
                if title_changed {
                    self.mark_changed();
                }

                // Genre.
                vcenter_label(ui, "Genre:");
                let mut genre = self.song.genre.clone();
                egui::ComboBox::from_id_salt("header_genre")
                    .selected_text(&genre)
                    .show_ui(ui, |ui| {
                        for g in CORE_GENRES.iter().chain(ADVANCED_GENRES.iter()) {
                            ui.selectable_value(&mut genre, g.to_string(), *g);
                        }
                    });
                if genre != self.song.genre {
                    self.song.genre = genre;
                    self.mark_changed();
                }

                // Key.
                vcenter_label(ui, "Key:");
                let key_changed = ui
                    .add_enabled_ui(!self.song.is_metadata_locked, |ui| {
                        let mut key = self.song.key.clone();
                        let resp = ui.add_sized(
                            egui::vec2(90.0, 30.0),
                            egui::TextEdit::singleline(&mut key)
                                // Same size as the button labels (15.6pt),
                                // so the text fills the 30px field.
                                .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center),
                        );
                        if resp.changed() {
                            self.song.key = key;
                            true
                        } else {
                            false
                        }
                    })
                    .inner;
                if key_changed {
                    self.mark_changed();
                }

                // BPM: only commits when the value parses to 20..400.
                vcenter_label(ui, "BPM:");
                let bpm_changed = ui
                    .add_enabled_ui(!self.song.is_metadata_locked, |ui| {
                        ui.add_sized(
                            egui::vec2(48.0, 30.0),
                            egui::TextEdit::singleline(&mut self.bpm_text)
                                // Same size as the button labels (15.6pt),
                                // so the text fills the 30px field.
                                .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center),
                        )
                        .changed()
                    })
                    .inner;
                if bpm_changed {
                    if let Ok(bpm) = self.bpm_text.trim().parse::<i32>() {
                        if (20..=400).contains(&bpm) && bpm != self.song.bpm {
                            self.song.bpm = bpm;
                            self.mark_changed();
                        }
                    }
                }

                // Groove.
                vcenter_label(ui, "Groove:");
                let mut groove = self.song.groove.clone();
                egui::ComboBox::from_id_salt("groove")
                    .selected_text(&groove)
                    .show_ui(ui, |ui| {
                        for g in crate::song::GROOVE_OPTIONS {
                            ui.selectable_value(&mut groove, g.to_string(), *g);
                        }
                    });
                if groove != self.song.groove {
                    self.song.groove = groove;
                    self.mark_changed();
                }

                // Rhymes toggle + metadata lock. Both are icon + text
                // buttons: the note / padlock is drawn with the painter
                // (the emoji glyphs are not in egui's default font).
                let rhymes_btn = ui
                    .add_sized(
                        egui::vec2(104.0, 30.0),
                        egui::Button::new("Rhymes").selected(self.show_rhyme),
                    )
                    .on_hover_text("Show or hide the rhyme dictionary panel");
                draw_icon_at_left(ui.painter(), rhymes_btn.rect, Icon::Note, pal.text);
                if rhymes_btn.clicked() {
                    self.show_rhyme = !self.show_rhyme;
                }
                let (lock_icon, lock_label) = if self.song.is_metadata_locked {
                    (Icon::Lock, "Locked")
                } else {
                    (Icon::Unlock, "Unlocked")
                };
                let lock_tip = if self.song.is_metadata_locked {
                    "Unlock song details (title, genre, key, BPM)"
                } else {
                    "Lock song details (title, genre, key, BPM)"
                };
                let lock_btn = ui
                    .add_sized(egui::vec2(120.0, 30.0), egui::Button::new(lock_label))
                    .on_hover_text(lock_tip);
                draw_icon_at_left(ui.painter(), lock_btn.rect, lock_icon, pal.text);
                if lock_btn.clicked() {
                    self.song.is_metadata_locked = !self.song.is_metadata_locked;
                    self.mark_changed();
                }
            });
        });
    }

    // ------------------------------------------------------------------
    // One section card. Returns true when the section was deleted, so the
    // caller stops drawing cards (indices shifted).
    // ------------------------------------------------------------------
    fn show_section_card(&mut self, ui: &mut egui::Ui, index: usize) -> bool {
        // Snapshot the read-only bits first so the borrow of the section
        // never overlaps a call that needs &mut self.
        let (
            section_id,
            section_label,
            section_type_name,
            is_locked,
            chords_text,
            time_sig_text,
            lyrics_snapshot,
        ) = {
            let s = &self.song.sections[index];
            (
                s.id.clone(),
                s.label.clone(),
                s.section_type.default_name().to_string(),
                s.is_locked,
                s.chords.clone().unwrap_or_default(),
                s.time_signature.clone().unwrap_or_default(),
                s.lyrics.clone(),
            )
        };
        let can_play = !parse_progression(&chords_text).is_empty();
        let playing_here = self.player.playing_section_id() == Some(section_id.clone())
            && self.player.is_playing();

        // Delete is deferred until after the card finishes drawing, so the
        // card never reads a section that is already gone. Duplicate is
        // deferred for the same reason (it inserts into the section list).
        let mut delete_requested = false;
        let mut duplicate_requested = false;
        let pal = self.theme.palette();
        // One clean unit: the card surface is a step above the editor
        // background, with a 1px border, 4px rounding, and 12px of
        // breathing room inside. A locked card gets a subtle accent edge
        // so the state reads at a glance, not just from the lock icon.
        let card_frame = egui::Frame::group(ui.style())
            .fill(pal.card)
            .stroke(egui::Stroke::new(1.0_f32, pal.border))
            .corner_radius(egui::CornerRadius::same(4))
            .inner_margin(egui::Margin::same(12));
        let card_frame = if is_locked {
            card_frame.stroke(egui::Stroke::new(1.0_f32, pal.accent.gamma_multiply(0.6)))
        } else {
            card_frame
        };
        card_frame.show(ui, |ui| {
            // Header row: label, time signature, chords, play, lock,
            // move, delete. Top-aligned, like the song header: a centered
            // row lets later controls drift down. The text fields are
            // 28px tall to match the 28×28 icon buttons, with their text
            // centered and at the 15.6pt button-label size so it fills
            // the field; the name heading and type tag are centered in
            // 28px boxes so they read as centered next to the fields.
            ui.horizontal_top(|ui| {
                let display_label = if section_label.trim().is_empty() {
                    section_type_name.clone()
                } else {
                    section_label.clone()
                };
                // The section name reads as a heading (strong, larger);
                // the type tag is quiet and muted.
                centered_label(
                    ui,
                    &display_label,
                    egui::FontId::new(16.0, egui::FontFamily::Proportional),
                    pal.text,
                    28.0,
                    true,
                );
                centered_label(
                    ui,
                    &format!("({})", section_type_name),
                    egui::TextStyle::Body.resolve(ui.style()),
                    pal.text_muted,
                    28.0,
                    false,
                );

                // Time signature. Disabled when the section is locked:
                // a locked section's content is read-only (the Kotlin
                // app shows it as plain text instead of a field).
                let mut ts = time_sig_text.clone();
                let ts_changed = ui
                    .add_enabled_ui(!is_locked, |ui| {
                        ui.add_sized(
                            egui::vec2(44.0, 28.0),
                            egui::TextEdit::singleline(&mut ts)
                                .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center)
                                .hint_text("4/4"),
                        )
                        .changed()
                    })
                    .inner;
                if ts_changed {
                    self.song.sections[index].time_signature =
                        if ts.trim().is_empty() { None } else { Some(ts) };
                    self.mark_changed();
                }

                // Label. Disabled when the section is locked, same as
                // the time signature and the lyrics below.
                let mut label = section_label.clone();
                let label_changed = ui
                    .add_enabled_ui(!is_locked, |ui| {
                        ui.add_sized(
                            egui::vec2(120.0, 28.0),
                            egui::TextEdit::singleline(&mut label)
                                .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center)
                                .hint_text("Label"),
                        )
                        .changed()
                    })
                    .inner;
                if label_changed {
                    self.song.sections[index].label = label;
                    self.mark_changed();
                }

                // Chords. Disabled when the section is locked, same as
                // the label and the lyrics below.
                let mut chords = chords_text.clone();
                let chords_changed = ui
                    .add_enabled_ui(!is_locked, |ui| {
                        ui.add_sized(
                            egui::vec2(220.0, 28.0),
                            egui::TextEdit::singleline(&mut chords)
                                .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center)
                                .hint_text("Chords, | splits bars (e.g., C G Am F | Dm G C)"),
                        )
                        .changed()
                    })
                    .inner;
                if chords_changed {
                    self.song.sections[index].chords = if chords.trim().is_empty() {
                        None
                    } else {
                        Some(chords)
                    };
                    self.mark_changed();
                }

                // The icon-only buttons (play, stop, lock, move, delete)
                // are a uniform 28×28 so they read as one tidy toolbar.
                // Each icon is drawn with the painter — the font glyphs
                // for play/move/delete/lock are not all in egui's
                // default font, so they would render as empty squares.
                let icon_size = egui::vec2(28.0, 28.0);

                // Play / stop.
                if playing_here {
                    if icon_button(ui, icon_size, Icon::Stop, pal.text)
                        .on_hover_text("Stop playback")
                        .clicked()
                    {
                        self.player.stop();
                    }
                } else {
                    let enabled = can_play && !self.player.unavailable();
                    let icon_color = if enabled { pal.text } else { pal.text_disabled };
                    let play_btn = ui
                        .add_enabled_ui(enabled, |ui| {
                            icon_button(ui, icon_size, Icon::Play, icon_color)
                        })
                        .inner
                        .on_hover_text("Play this section's chords");
                    if play_btn.clicked() {
                        let section = self.song.sections[index].clone();
                        let bpm = self.song.bpm;
                        self.player.play(&section, bpm);
                    }
                    if self.player.unavailable() {
                        play_btn.on_disabled_hover_text("Audio is not available on this device");
                    }
                }
                // Lock.
                let lock_icon = if is_locked { Icon::Lock } else { Icon::Unlock };
                let lock_tip = if is_locked {
                    "Unlock section (allow editing)"
                } else {
                    "Lock section (prevent editing)"
                };
                if icon_button(ui, icon_size, lock_icon, pal.text)
                    .on_hover_text(lock_tip)
                    .clicked()
                {
                    self.song.sections[index].is_locked = !is_locked;
                    self.mark_changed();
                }

                // Move up / down. Hidden when the section is locked:
                // in the Kotlin app the move buttons only exist on
                // unlocked cards — reordering a locked section is
                // editing it. The triangles are drawn with the
                // painter: ▲/▼ are not in egui's default font and render
                // as empty squares.
                if !is_locked {
                    if icon_button(ui, icon_size, Icon::MoveUp, pal.text)
                        .on_hover_text("Move section up")
                        .clicked()
                        && index > 0
                    {
                        self.song.sections.swap(index, index - 1);
                        self.mark_changed();
                    }
                    if icon_button(ui, icon_size, Icon::MoveDown, pal.text)
                        .on_hover_text("Move section down")
                        .clicked()
                        && index + 1 < self.song.sections.len()
                    {
                        self.song.sections.swap(index, index + 1);
                        self.mark_changed();
                    }
                }

                // Delete (deferred until the card finishes drawing).
                // Hidden when the section is locked, same as the move
                // buttons: the Kotlin app shows no delete button on a
                // locked card.
                if !is_locked
                    && icon_button(ui, icon_size, Icon::Delete, pal.text)
                        .on_hover_text("Delete section")
                        .clicked()
                {
                    delete_requested = true;
                }

                // Duplicate (deferred until the card finishes drawing).
                // Shown even on a locked card: duplicating does not edit
                // the original, and the copy is unlocked, so it is a way
                // to get an editable copy of a locked section.
                if icon_button(ui, icon_size, Icon::Duplicate, pal.text)
                    .on_hover_text("Duplicate section")
                    .clicked()
                {
                    duplicate_requested = true;
                }
            });

            // A thin divider separates the header row from the lyrics area.
            ui.add_space(4.0);
            ui.separator();
            ui.add_space(4.0);

            // Diatonic chord chips for the song key + Fill button.
            // Disabled when the section is locked: tapping a chip or
            // filling a progression edits the section's chords (the
            // Kotlin app hides the chips and Fill on a locked card).
            let key = self.song.key.clone();
            if let Some(parsed) = parse_key(&key) {
                let diatonic = diatonic_chords(parsed);
                let progressions = suggested_progressions(parsed);
                ui.add_enabled_ui(!is_locked, |ui| {
                    // Top-aligned like the header row: the chord chips
                    // are 30px buttons, and a centered row lets later
                    // chips drift down. The "Chords:" label is centered
                    // in 30px so it reads as centered next to the chips.
                    ui.horizontal_top(|ui| {
                        centered_label(
                            ui,
                            "Chords:",
                            egui::TextStyle::Body.resolve(ui.style()),
                            pal.text_muted,
                            30.0,
                            false,
                        );
                        for chord in &diatonic {
                            if ui
                                .button(chord)
                                .on_hover_text(format!("Add {} to this section's chords", chord))
                                .clicked()
                            {
                                let current =
                                    self.song.sections[index].chords.clone().unwrap_or_default();
                                let next = if current.trim().is_empty() {
                                    chord.clone()
                                } else {
                                    format!("{} - {}", current.trim(), chord)
                                };
                                self.song.sections[index].chords = Some(next);
                                self.mark_changed();
                            }
                        }
                        if ui
                            .button("Fill")
                            .on_hover_text("Fill in a suggested chord progression")
                            .clicked()
                            && !progressions.is_empty()
                        {
                            let idx = *self.progression_index.get(&section_id).unwrap_or(&0)
                                % progressions.len();
                            self.song.sections[index].chords = Some(progressions[idx].clone());
                            self.progression_index.insert(section_id.clone(), idx + 1);
                            self.mark_changed();
                        }
                    })
                });
            }

            // Lyrics (egui TextEdit has built-in undo/redo: Ctrl+Z,
            // Ctrl+Shift+Z / Ctrl+Y). Disabled when the section is locked.
            ui.add_space(4.0);
            let lyrics_changed = {
                let section = &mut self.song.sections[index];
                let resp = ui.add_enabled(
                    !is_locked,
                    egui::TextEdit::multiline(&mut section.lyrics)
                        .desired_width(f32::INFINITY)
                        .desired_rows(6)
                        .font(egui::FontId::new(14.0, egui::FontFamily::Proportional)),
                );
                resp.changed()
            };
            if lyrics_changed {
                self.mark_changed();
                self.last_edited_section_id = Some(section_id.clone());
                // Seed the rhyme search with the last word typed.
                let lyrics = self.song.sections[index].lyrics.clone();
                if let Some(last) = lyrics.split_whitespace().last() {
                    let clean: String = last
                        .trim_matches(|c: char| !c.is_alphanumeric())
                        .to_lowercase();
                    if !clean.is_empty() {
                        self.rhyme_query = clean;
                        self.refresh_rhymes();
                    }
                }
            }

            // Meter + rhyme scheme + word count, as a quiet secondary line.
            ui.add_space(8.0);
            let meter = analyze_meter(&lyrics_snapshot);
            let mut info_parts: Vec<String> = Vec::new();
            if !meter.counts.is_empty() {
                let counts: Vec<String> = meter.counts.iter().map(|c| c.to_string()).collect();
                info_parts.push(format!("Syllables: {}", counts.join(" · ")));
                if let Some(common) = meter.most_common {
                    if !meter.outlier_lines.is_empty() {
                        let lines: Vec<String> =
                            meter.outlier_lines.iter().map(|l| l.to_string()).collect();
                        info_parts.push(format!(
                            "Lines {} differ from the usual {} syllables",
                            lines.join(", "),
                            common
                        ));
                    }
                }
            }
            let scheme = rhyme_scheme_label(&lyrics_snapshot);
            if !scheme.is_empty() {
                info_parts.push(format!("Rhyme scheme: {}", scheme));
            }
            info_parts.push(format!("{} words", count_words(&lyrics_snapshot)));
            ui.label(
                egui::RichText::new(info_parts.join("   "))
                    .small()
                    .color(pal.text_muted),
            );
        });
        if duplicate_requested {
            self.song.duplicate_section(index);
            self.mark_changed();
        }
        if delete_requested {
            self.player.stop();
            self.song.sections.remove(index);
            self.mark_changed();
            return true;
        }
        false
    }

    // ------------------------------------------------------------------
    // Rhyme assistant side panel.
    // ------------------------------------------------------------------
    fn show_rhyme_panel(&mut self, ctx: &egui::Context) {
        let pal = self.theme.palette();
        egui::SidePanel::right("rhyme_panel")
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.add_space(8.0);
                ui.heading("Rhyme Assistant");
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Word:").color(pal.text_muted));
                    let resp = ui.text_edit_singleline(&mut self.rhyme_query);
                    if resp.changed() {
                        self.refresh_rhymes();
                    }
                });
                ui.horizontal(|ui| {
                    for mode in RhymeMode::all() {
                        if ui
                            .selectable_label(self.rhyme_mode == *mode, mode.label())
                            .clicked()
                        {
                            self.rhyme_mode = *mode;
                            self.refresh_rhymes();
                        }
                    }
                });
                // Syllable-count filter for the displayed results. It only
                // filters what is shown — the ranking and the query are
                // unchanged — and the choice persists when the panel is
                // hidden and re-shown (it lives in the app state).
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Syllables:").color(pal.text_muted));
                    egui::ComboBox::from_id_salt("rhyme_syllable_filter")
                        .selected_text(syllable_filter_label(self.rhyme_syllable_filter))
                        .show_ui(ui, |ui| {
                            for filter in SYLLABLE_FILTER_OPTIONS {
                                if ui
                                    .selectable_label(
                                        self.rhyme_syllable_filter == filter,
                                        syllable_filter_label(filter),
                                    )
                                    .clicked()
                                {
                                    self.rhyme_syllable_filter = filter;
                                }
                            }
                        });
                });
                ui.separator();
                if self.rhyme_results.is_empty() {
                    ui.label("No rhymes found. Try another word.");
                } else {
                    // Filter the displayed results by syllable count.
                    // The ranking is untouched; only the visible words
                    // are restricted.
                    let filter = self.rhyme_syllable_filter;
                    let results: Vec<String> = self
                        .rhyme_results
                        .iter()
                        .filter(|word| {
                            syllable_count_matches_filter(syllable_count_for(word), filter)
                        })
                        .cloned()
                        .collect();
                    if results.is_empty() {
                        ui.label(
                            egui::RichText::new(format!(
                                "No rhymes with {}.",
                                syllable_filter_label(filter).to_lowercase()
                            ))
                            .color(pal.text_muted),
                        );
                    } else {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for word in results {
                                if ui
                                    .button(&word)
                                    .on_hover_text(format!("Insert '{}' into the lyrics", word))
                                    .clicked()
                                {
                                    self.insert_rhyme(&word);
                                }
                            }
                        });
                    }
                }
            });
    }

    fn refresh_rhymes(&mut self) {
        let query = self.rhyme_query.trim().to_lowercase();
        if query.len() < 2 {
            self.rhyme_results.clear();
            return;
        }
        self.rhyme_results = find_rhymes(&query, self.rhyme_mode);
    }

    fn insert_rhyme(&mut self, word: &str) {
        // Insert into the last-edited section, or the first section.
        let target = self
            .last_edited_section_id
            .as_ref()
            .and_then(|id| self.song.sections.iter().position(|s| &s.id == id))
            .unwrap_or(0);
        if let Some(section) = self.song.sections.get_mut(target) {
            if section.lyrics.is_empty() {
                section.lyrics = word.to_string();
            } else if section.lyrics.ends_with(' ') || section.lyrics.ends_with('\n') {
                section.lyrics.push_str(word);
            } else {
                section.lyrics.push(' ');
                section.lyrics.push_str(word);
            }
            self.mark_changed();
        }
    }

    // ------------------------------------------------------------------
    // Add Section dialog.
    // ------------------------------------------------------------------
    pub fn show_add_section_dialog(&mut self, ctx: &egui::Context) {
        let pal = self.theme.palette();
        egui::Window::new("Add Section")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Section type:").color(pal.text_muted));
                egui::ComboBox::from_id_salt("add_section_type")
                    .selected_text(self.add_type.default_name())
                    .show_ui(ui, |ui| {
                        for t in SectionType::all() {
                            ui.selectable_value(&mut self.add_type, *t, t.default_name());
                        }
                    });
                ui.label(egui::RichText::new("Label:").color(pal.text_muted));
                ui.text_edit_singleline(&mut self.add_label);
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        let section = SongSection::new(self.add_type, &self.add_label);
                        self.song.sections.push(section);
                        self.mark_changed();
                        self.show_add_section = false;
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_add_section = false;
                    }
                });
            });
    }

    // ------------------------------------------------------------------
    // About dialog. Opened from Help → About Lyricshop... (editor
    // only). The X button, the OK button, or Esc closes it.
    // ------------------------------------------------------------------
    pub fn show_about_dialog(&mut self, ctx: &egui::Context) {
        // Esc closes the dialog and returns to the editor.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_about = false;
            return;
        }
        let pal = self.theme.palette();
        // The window's X button writes to a local `open` flag: the
        // dialog closure borrows `&mut self`, so it cannot also borrow
        // `self.show_about` for `.open()`. Reconcile after.
        let mut open = self.show_about;
        egui::Window::new("About Lyricshop")
            .collapsible(false)
            .resizable(false)
            .default_width(320.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.heading("Lyricshop");
                ui.add_space(8.0);
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.label("Written by Andrew Kordula");
                ui.label(
                    egui::RichText::new(
                        "Licensed under the GNU General Public License v3.0 (or later).",
                    )
                    .color(pal.text_muted),
                );
                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        self.show_about = false;
                    }
                });
            });
        if !open {
            // The X button was clicked.
            self.show_about = false;
        }
    }

    // ------------------------------------------------------------------
    // Find & replace (all sections' lyrics). Case-insensitive substring
    // matching, replace-all only — no regex or whole-word toggles in
    // this version. Opened from Edit → Find & Replace... or Ctrl+H
    // (editor only). The X button, the Close button, or Esc returns to
    // the editor.
    // ------------------------------------------------------------------
    pub fn show_find_replace_dialog(&mut self, ctx: &egui::Context) {
        // Esc closes the dialog and returns to the editor.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.show_find_replace = false;
            return;
        }
        let pal = self.theme.palette();
        // The window's X button writes to a local `open` flag: the
        // dialog closure borrows `&mut self`, so it cannot also borrow
        // `self.show_find_replace` for `.open()`. Reconcile after.
        let mut open = self.show_find_replace;
        egui::Window::new("Find & Replace")
            .collapsible(false)
            .resizable(false)
            .default_width(360.0)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label(egui::RichText::new("Find:").color(pal.text_muted));
                let find_changed = ui.text_edit_singleline(&mut self.find_text).changed();
                ui.label(egui::RichText::new("Replace with:").color(pal.text_muted));
                let replace_changed = ui.text_edit_singleline(&mut self.replace_text).changed();
                if find_changed || replace_changed {
                    // The live match count takes over again once the
                    // user edits either field.
                    self.find_replace_message.clear();
                }
                ui.add_space(8.0);
                // Live match count across every section's lyrics.
                let matches: usize = self
                    .song
                    .sections
                    .iter()
                    .map(|s| count_matches_ci(&s.lyrics, &self.find_text))
                    .sum();
                let message = if !self.find_replace_message.is_empty() {
                    self.find_replace_message.clone()
                } else if self.find_text.is_empty() {
                    "Type text to find.".to_string()
                } else if matches == 0 {
                    "No matches".to_string()
                } else if matches == 1 {
                    "1 match".to_string()
                } else {
                    format!("{matches} matches")
                };
                ui.label(egui::RichText::new(message).color(pal.text_muted));
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button("Replace All").clicked() {
                        let mut total = 0;
                        for section in &mut self.song.sections {
                            let (new_lyrics, n) = replace_all_ci(
                                &section.lyrics,
                                &self.find_text,
                                &self.replace_text,
                            );
                            if n > 0 {
                                section.lyrics = new_lyrics;
                                total += n;
                            }
                        }
                        if total > 0 {
                            self.mark_changed();
                            self.show_status(format!("Replaced {total}"));
                            self.find_replace_message = format!("Replaced {total}");
                        } else {
                            self.find_replace_message = "No matches".to_string();
                        }
                    }
                    if ui.button("Close").clicked() {
                        self.show_find_replace = false;
                    }
                });
            });
        if !open {
            // The X button was clicked.
            self.show_find_replace = false;
        }
    }

    // ------------------------------------------------------------------
    // Snapshot dialog.
    // ------------------------------------------------------------------
    pub fn show_snapshot_dialog(&mut self, ctx: &egui::Context) {
        let pal = self.theme.palette();
        egui::Window::new("Snapshots")
            .collapsible(false)
            .resizable(true)
            .default_width(420.0)
            .show(ctx, |ui| {
                if self.snapshot_entries.is_empty() {
                    ui.label(egui::RichText::new("No snapshots yet. Save the song to a file first, then use File → Save Snapshot.").color(pal.text_muted));
                } else {
                    let entries = self.snapshot_entries.clone();
                    let mut restore: Option<std::path::PathBuf> = None;
                    let mut delete: Option<std::path::PathBuf> = None;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for entry in &entries {
                            ui.horizontal(|ui| {
                                ui.label(format!("{} — {}", entry.display_date, entry.title));
                                if ui.button("Restore").clicked() {
                                    restore = Some(entry.file.clone());
                                }
                                if ui.button("Delete").clicked() {
                                    delete = Some(entry.file.clone());
                                }
                            });
                        }
                    });
                    if let Some(file) = restore {
                        if load_snapshot(&mut self.song, &file) {
                            self.dirty = true;
                            self.bpm_text = self.song.bpm.to_string();
                            self.show_status("Snapshot restored");
                        } else {
                            self.show_status("Could not restore snapshot");
                        }
                        self.show_snapshots = false;
                    }
                    if let Some(file) = delete {
                        delete_snapshot(&file);
                        self.refresh_snapshots();
                    }
                }
                if ui.button("Close").clicked() {
                    self.show_snapshots = false;
                }
            });
    }

    // ------------------------------------------------------------------
    // Song library dialog.
    // ------------------------------------------------------------------
    pub fn show_library_dialog(&mut self, ctx: &egui::Context) {
        let pal = self.theme.palette();
        egui::Window::new("Song Library")
            .collapsible(false)
            .resizable(true)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.label(
                    egui::RichText::new("Songs in the same folder as the current song:")
                        .color(pal.text_muted),
                );
                let resp = ui.text_edit_singleline(&mut self.library_query);
                if resp.changed() {
                    // Filtering happens at draw time from library_entries.
                }
                let filtered = filter_library(&self.library_entries, &self.library_query);
                if filtered.is_empty() {
                    ui.label(egui::RichText::new("No songs found here yet.").color(pal.text_muted));
                } else {
                    let rows: Vec<(std::path::PathBuf, String, String)> = filtered
                        .iter()
                        .map(|e| (e.file.clone(), e.title.clone(), e.genre.clone()))
                        .collect();
                    let mut open: Option<std::path::PathBuf> = None;
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for (file, title, genre) in &rows {
                            if ui.button(format!("{} — {}", title, genre)).clicked() {
                                open = Some(file.clone());
                            }
                        }
                    });
                    if let Some(file) = open {
                        if self.song_file.load_song_file(&mut self.song, &file) {
                            self.dirty = false;
                            self.bpm_text = self.song.bpm.to_string();
                            self.show_status(format!("Opened \"{}\"", self.song.title));
                        } else {
                            self.show_status("Could not open file");
                        }
                        self.show_library = false;
                    }
                }
                if ui.button("Close").clicked() {
                    self.show_library = false;
                }
            });
    }

    // ------------------------------------------------------------------
    // Close confirmation (unsaved changes).
    // ------------------------------------------------------------------
    pub fn show_close_confirm_dialog(&mut self, ctx: &egui::Context) {
        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("You have unsaved changes. Save before closing?");
                ui.horizontal(|ui| {
                    if ui.button("Save").clicked() {
                        if self.song_file.save_song(&self.song) {
                            self.dirty = false;
                        }
                        self.close_confirmed = true;
                        self.show_close_confirm = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Don't Save").clicked() {
                        self.close_confirmed = true;
                        self.show_close_confirm = false;
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                    if ui.button("Cancel").clicked() {
                        self.show_close_confirm = false;
                    }
                });
            });
    }
}

// ------------------------------------------------------------------
// Find & replace matching (case-insensitive substring). Pure, so it
// is unit-testable. Matching is done on lowercased chars, one per
// original char, so match positions map back onto the original text
// without ever slicing on a non-char boundary.
// ------------------------------------------------------------------

/// Lowercased chars of `s`, one per original char (the first char of
/// each char's lowercase expansion). Keeps the vector aligned with the
/// original chars, so a match position is a position in the original.
fn lowercase_chars(s: &str) -> Vec<char> {
    s.chars()
        .map(|c| c.to_lowercase().next().unwrap_or(c))
        .collect()
}

/// Start indices (in chars) of the non-overlapping case-insensitive
/// occurrences of `find` in `text`. Empty when `find` is empty.
fn match_starts_ci(text: &str, find: &str) -> Vec<usize> {
    if find.is_empty() {
        return Vec::new();
    }
    let hay = lowercase_chars(text);
    let needle = lowercase_chars(find);
    if needle.is_empty() || needle.len() > hay.len() {
        return Vec::new();
    }
    let mut starts = Vec::new();
    let mut i = 0;
    while i + needle.len() <= hay.len() {
        if hay[i..i + needle.len()] == needle[..] {
            starts.push(i);
            i += needle.len();
        } else {
            i += 1;
        }
    }
    starts
}

/// Case-insensitive count of `find` in `text`. 0 when `find` is empty.
fn count_matches_ci(text: &str, find: &str) -> usize {
    match_starts_ci(text, find).len()
}

/// Case-insensitive replace-all. Returns the new text and the number of
/// replacements. Returns the text unchanged (count 0) when `find` is
/// empty. The replacement text is inserted exactly as typed.
fn replace_all_ci(text: &str, find: &str, replace: &str) -> (String, usize) {
    let starts = match_starts_ci(text, find);
    if starts.is_empty() {
        return (text.to_string(), 0);
    }
    let needle_len = lowercase_chars(find).len();
    let orig: Vec<char> = text.chars().collect();
    let mut out = String::new();
    let mut prev = 0;
    for &start in &starts {
        out.extend(orig[prev..start].iter().copied());
        out.push_str(replace);
        prev = start + needle_len;
    }
    out.extend(orig[prev..].iter().copied());
    (out, starts.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_is_case_insensitive() {
        assert_eq!(count_matches_ci("Hello hello HELLO", "hello"), 3);
        assert_eq!(count_matches_ci("Hello hello HELLO", "HELLO"), 3);
        assert_eq!(count_matches_ci("Hello hello HELLO", "HeLLo"), 3);
    }

    #[test]
    fn count_is_non_overlapping() {
        // "aaa" contains "aa" twice overlapping, but replace-all only
        // replaces once, so the count is 1.
        assert_eq!(count_matches_ci("aaa", "aa"), 1);
        assert_eq!(count_matches_ci("aaaa", "aa"), 2);
    }

    #[test]
    fn count_empty_find_is_zero() {
        assert_eq!(count_matches_ci("hello", ""), 0);
        assert_eq!(count_matches_ci("", "hello"), 0);
    }

    #[test]
    fn replace_is_case_insensitive_and_keeps_surrounding_case() {
        let (out, n) = replace_all_ci("Hello hello HELLO world", "hello", "bye");
        assert_eq!(out, "bye bye bye world");
        assert_eq!(n, 3);
    }

    #[test]
    fn replace_no_match_leaves_text_unchanged() {
        let (out, n) = replace_all_ci("hello world", "goodbye", "bye");
        assert_eq!(out, "hello world");
        assert_eq!(n, 0);
    }

    #[test]
    fn replace_empty_find_leaves_text_unchanged() {
        let (out, n) = replace_all_ci("hello world", "", "bye");
        assert_eq!(out, "hello world");
        assert_eq!(n, 0);
    }

    #[test]
    fn replace_works_across_lines() {
        let lyrics = "daylight fades\nDaylight breaks\ndaylight";
        let (out, n) = replace_all_ci(lyrics, "daylight", "sunlight");
        assert_eq!(out, "sunlight fades\nsunlight breaks\nsunlight");
        assert_eq!(n, 3);
    }

    #[test]
    fn replace_inserts_replacement_verbatim() {
        let (out, n) = replace_all_ci("aaa", "a", "BB");
        assert_eq!(out, "BBBBBB");
        assert_eq!(n, 3);
    }
}
