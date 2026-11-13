//! Theme for Lyricshop: a dark UI with layered grays and a light theme.
//!
//! The look is built with egui's own `Visuals`/`Style` — window
//! background, panel fill, text, selection, and button states are all
//! set explicitly from the palette, never left to egui's defaults.
//!
//! The design is depth through layered grays, not flat black:
//! the editor background is the darkest layer, section cards sit a step
//! above it, and the menu bar / header / status bar / side panel are a
//! step above the cards, with 1px borders a touch lighter than the
//! surface they edge. The palette stays dark, and the blue accent is kept.
//!
//! Palettes:
//! - Dark (the default): editor #111113, chrome #232328,
//!   card #1b1b1f, text #e8e8ea, accent #1a73e8.
//! - Light: editor #ffffff, chrome #e9e9ec, card #f4f4f6,
//!   text #1a1a1a, accent #1a73e8.

use egui::{Color32, Stroke, Visuals};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn label(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
        }
    }

    pub fn all() -> &'static [Theme] {
        &[Theme::Dark, Theme::Light]
    }

    pub fn parse(name: &str) -> Theme {
        match name {
            "light" => Theme::Light,
            _ => Theme::Dark,
        }
    }

    pub fn id(&self) -> &'static str {
        match self {
            Theme::Dark => "dark",
            Theme::Light => "light",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    /// Window / editor background (darkest layer).
    pub bg: Color32,
    /// Panels: menu bar, header, status bar, side panel.
    pub chrome: Color32,
    /// Section card surface (a step above bg, below chrome).
    pub card: Color32,
    /// Text fields, buttons.
    pub input: Color32,
    /// Hover fill for buttons and inputs.
    pub hover: Color32,
    /// Borders and subtle separators.
    pub border: Color32,
    /// Primary text.
    pub text: Color32,
    /// Secondary / muted text.
    pub text_muted: Color32,
    /// Disabled text.
    pub text_disabled: Color32,
    /// Accent blue: selections, active items, primary buttons.
    pub accent: Color32,
}

const DARK: Palette = Palette {
    bg: Color32::from_rgb(0x11, 0x11, 0x13),
    chrome: Color32::from_rgb(0x23, 0x23, 0x28),
    card: Color32::from_rgb(0x1b, 0x1b, 0x1f),
    input: Color32::from_rgb(0x2b, 0x2b, 0x30),
    hover: Color32::from_rgb(0x34, 0x34, 0x3a),
    border: Color32::from_rgb(0x40, 0x40, 0x46),
    text: Color32::from_rgb(0xe8, 0xe8, 0xea),
    text_muted: Color32::from_rgb(0x9e, 0x9e, 0xa6),
    text_disabled: Color32::from_rgb(0x6e, 0x6e, 0x73),
    accent: Color32::from_rgb(0x1a, 0x73, 0xe8),
};

const LIGHT: Palette = Palette {
    bg: Color32::from_rgb(0xff, 0xff, 0xff),
    chrome: Color32::from_rgb(0xe9, 0xe9, 0xec),
    card: Color32::from_rgb(0xf4, 0xf4, 0xf6),
    input: Color32::from_rgb(0xff, 0xff, 0xff),
    hover: Color32::from_rgb(0xe2, 0xe2, 0xe6),
    border: Color32::from_rgb(0xd2, 0xd2, 0xd7),
    text: Color32::from_rgb(0x1a, 0x1a, 0x1a),
    text_muted: Color32::from_rgb(0x6e, 0x6e, 0x73),
    text_disabled: Color32::from_rgb(0xa0, 0xa0, 0xa0),
    accent: Color32::from_rgb(0x1a, 0x73, 0xe8),
};

impl Theme {
    pub fn palette(&self) -> Palette {
        match self {
            Theme::Dark => DARK,
            Theme::Light => LIGHT,
        }
    }

    /// Build egui `Visuals` from this theme's palette. Everything is set
    /// explicitly so the look does not depend on egui's built-in themes.
    pub fn visuals(&self) -> Visuals {
        let p = self.palette();
        let mut v = Visuals::default();
        v.dark_mode = matches!(self, Theme::Dark);

        // Window and panel backgrounds. The chrome strips (menu bar,
        // header, status bar, side panel) use `panel_fill`, so they read
        // as a layer above the editor background; the editor's
        // CentralPanel overrides its own fill back to `bg` (see ui.rs).
        v.window_fill = p.chrome;
        v.panel_fill = p.chrome;
        v.extreme_bg_color = p.bg;
        v.faint_bg_color = p.chrome;
        v.code_bg_color = p.input;

        // Text.
        v.override_text_color = Some(p.text);
        v.warn_fg_color = p.text_muted;
        v.error_fg_color = Color32::from_rgb(0xd9, 0x53, 0x4f);

        // Selection (text selection and selected items).
        v.selection.bg_fill = p.accent;
        v.selection.stroke = Stroke::new(1.0_f32, p.accent);

        // Widgets: buttons, text fields, sliders.
        v.widgets.noninteractive.bg_fill = p.chrome;
        v.widgets.noninteractive.fg_stroke = Stroke::new(1.0_f32, p.text_muted);
        v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.border);

        v.widgets.inactive.bg_fill = p.input;
        v.widgets.inactive.fg_stroke = Stroke::new(1.0_f32, p.text);
        v.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, p.border);
        v.widgets.inactive.weak_bg_fill = p.input;
        v.widgets.inactive.corner_radius = egui::CornerRadius::same(4);

        v.widgets.hovered.bg_fill = p.hover;
        v.widgets.hovered.fg_stroke = Stroke::new(1.0_f32, p.text);
        v.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, p.border);
        v.widgets.hovered.weak_bg_fill = p.hover;
        v.widgets.hovered.corner_radius = egui::CornerRadius::same(4);

        v.widgets.active.bg_fill = p.accent;
        v.widgets.active.fg_stroke = Stroke::new(1.0_f32, Color32::WHITE);
        v.widgets.active.bg_stroke = Stroke::new(1.0_f32, p.accent);
        v.widgets.active.weak_bg_fill = p.accent;
        v.widgets.active.corner_radius = egui::CornerRadius::same(4);

        v.widgets.open.bg_fill = p.input;
        v.widgets.open.fg_stroke = Stroke::new(1.0_f32, p.text);
        v.widgets.open.bg_stroke = Stroke::new(1.0_f32, p.border);
        v.widgets.open.weak_bg_fill = p.input;
        v.widgets.open.corner_radius = egui::CornerRadius::same(4);

        // Separators, resize handles, hyperlink. Dialogs use the same
        // restrained 4px rounding as buttons and cards.
        v.window_stroke = Stroke::new(1.0_f32, p.border);
        v.window_corner_radius = egui::CornerRadius::same(4);
        v.window_shadow = egui::Shadow::NONE;
        v.popup_shadow = egui::Shadow::NONE;
        v.hyperlink_color = p.accent;

        v
    }
}

/// Apply a theme to the egui context (sets visuals + style).
///
/// Spacing follows an 8px grid: 8px gaps between widgets, 16px dialog
/// padding, 8px menu padding, 16px indent.
///
/// Typographic hierarchy: headings (start-screen title, panel titles)
/// are 24, body text stays at egui's 12.5, and secondary lines (card
/// footers, status bar) use Small at 11 in the muted colour.
///
/// Buttons are sized to match the reference app's pill buttons (30px
/// tall): the button font is 15.6 (up from egui's 12.5) and the button
/// padding is (5, 6), so a text button's height is the 15.6pt row
/// height (~17.9) plus 2 x 6 padding = ~30px. The icon buttons stay
/// 28x28, matching the reference's toolbar icons.
pub fn apply_theme(ctx: &egui::Context, theme: Theme) {
    ctx.set_visuals(theme.visuals());
    let mut style = (*ctx.style()).clone();
    style.text_styles.insert(
        egui::TextStyle::Button,
        egui::FontId::new(15.6, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Heading,
        egui::FontId::new(24.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Small,
        egui::FontId::new(11.0, egui::FontFamily::Proportional),
    );
    style.spacing.item_spacing = egui::vec2(8.0, 8.0);
    style.spacing.window_margin = egui::Margin::same(16);
    style.spacing.menu_margin = egui::Margin::same(8);
    style.spacing.indent = 16.0;
    style.spacing.button_padding = egui::vec2(5.0, 6.0);
    ctx.set_style(style);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_is_default_palette() {
        let p = Theme::Dark.palette();
        assert_eq!(p.bg, Color32::from_rgb(0x11, 0x11, 0x13));
        assert_eq!(p.chrome, Color32::from_rgb(0x23, 0x23, 0x28));
        assert_eq!(p.card, Color32::from_rgb(0x1b, 0x1b, 0x1f));
        assert_eq!(p.text, Color32::from_rgb(0xe8, 0xe8, 0xea));
        assert_eq!(p.accent, Color32::from_rgb(0x1a, 0x73, 0xe8));
    }

    #[test]
    fn light_palette_matches_novelshop_light() {
        let p = Theme::Light.palette();
        assert_eq!(p.bg, Color32::from_rgb(0xff, 0xff, 0xff));
        assert_eq!(p.chrome, Color32::from_rgb(0xe9, 0xe9, 0xec));
        assert_eq!(p.card, Color32::from_rgb(0xf4, 0xf4, 0xf6));
        assert_eq!(p.text, Color32::from_rgb(0x1a, 0x1a, 0x1a));
        assert_eq!(p.accent, Color32::from_rgb(0x1a, 0x73, 0xe8));
    }

    #[test]
    fn theme_parse_defaults_to_dark() {
        assert_eq!(Theme::parse("light"), Theme::Light);
        assert_eq!(Theme::parse("dark"), Theme::Dark);
        assert_eq!(Theme::parse("bogus"), Theme::Dark);
        assert_eq!(Theme::parse(""), Theme::Dark);
    }

    #[test]
    fn visuals_carry_the_palette() {
        let v = Theme::Dark.visuals();
        assert!(v.dark_mode);
        // Panels (menu bar, header, status bar) are the chrome layer,
        // a step above the editor background.
        assert_eq!(v.panel_fill, Color32::from_rgb(0x23, 0x23, 0x28));
        assert_eq!(v.extreme_bg_color, Color32::from_rgb(0x11, 0x11, 0x13));
        assert_eq!(v.selection.bg_fill, Color32::from_rgb(0x1a, 0x73, 0xe8));
        let v = Theme::Light.visuals();
        assert!(!v.dark_mode);
        assert_eq!(v.panel_fill, Color32::from_rgb(0xe9, 0xe9, 0xec));
        assert_eq!(v.extreme_bg_color, Color32::from_rgb(0xff, 0xff, 0xff));
    }

    #[test]
    fn card_is_between_bg_and_chrome() {
        // Layer order, dark: bg < card < chrome. Light: chrome < card < bg.
        let d = Theme::Dark.palette();
        assert!(d.bg.r() < d.card.r() && d.card.r() < d.chrome.r());
        let l = Theme::Light.palette();
        assert!(l.chrome.r() < l.card.r() && l.card.r() < l.bg.r());
    }

    #[test]
    fn text_buttons_are_30px_tall() {
        // Text buttons match the reference app's pill buttons (30px).
        // The height is the 15.6pt row height plus 2 x the vertical
        // button padding, measured in a headless egui frame.
        let ctx = egui::Context::default();
        apply_theme(&ctx, Theme::Dark);
        let mut height: f32 = 0.0;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                height = ui.button("Measure").rect.height();
            });
        });
        assert!((height - 30.0).abs() < 0.5, "button height {height}");
    }

    #[test]
    fn top_bar_controls_are_30px_tall() {
        // The menu bar, header text fields, and dropdowns are the same
        // height as the buttons (30px), so nothing looks big or small
        // by comparison. The menu bar needs the same button padding
        // override as app.rs, since egui's default menu style shrinks
        // it to (2, 0).
        let ctx = egui::Context::default();
        apply_theme(&ctx, Theme::Dark);
        let mut menu_h: f32 = 0.0;
        let mut textedit_h: f32 = 0.0;
        let mut combo_h: f32 = 0.0;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                    egui::MenuBar::new()
                        .style(|style: &mut egui::Style| {
                            style.spacing.button_padding = egui::vec2(5.0, 6.0);
                            style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
                            style.visuals.widgets.open.bg_stroke = egui::Stroke::NONE;
                            style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                            style.visuals.widgets.inactive.weak_bg_fill =
                                egui::Color32::TRANSPARENT;
                            style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                        })
                        .ui(ui, |ui| {
                            menu_h = ui
                                .menu_button("File", |ui| {
                                    ui.button("Save");
                                })
                                .response
                                .rect
                                .height();
                            let mut s = String::from("x");
                            textedit_h = ui
                                .add_sized(
                                    egui::vec2(180.0, 30.0),
                                    egui::TextEdit::singleline(&mut s),
                                )
                                .rect
                                .height();
                            let mut sel = String::from("Rock");
                            combo_h = egui::ComboBox::from_id_salt("m")
                                .selected_text(&sel)
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(&mut sel, "Rock".to_string(), "Rock");
                                })
                                .response
                                .rect
                                .height();
                        });
                });
            });
        });
        assert!((menu_h - 30.0).abs() < 0.5, "menu button height {menu_h}");
        assert!(
            (textedit_h - 30.0).abs() < 0.5,
            "textedit height {textedit_h}"
        );
        assert!((combo_h - 30.0).abs() < 0.5, "combo height {combo_h}");
    }

    #[test]
    fn header_controls_share_the_same_top_edge() {
        // The Genre/Groove ComboBoxes sat 6px lower than the Title/Key/BPM
        // fields. Findings: a fixed-height centered row does NOT fix it
        // (combo still 6px lower, button 3px lower); wrapping the ComboBox
        // in its own 30px box does NOT fix it either (still 6px lower).
        // horizontal_top DOES align them: text and combo share the top.
        // So the header row is top-aligned, and the small labels are
        // wrapped in their own 30px centered boxes so they still read
        // as vertically centered next to the 30px fields.
        let ctx = egui::Context::default();
        apply_theme(&ctx, Theme::Dark);
        let mut text_top = 0.0f32;
        let mut combo_top = 0.0f32;
        let mut btn_top = 0.0f32;
        let mut label_top = 0.0f32;
        let mut label_bot = 0.0f32;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.horizontal_top(|ui| {
                    // Label wrapped in a 30px centered box.
                    let font_id = egui::TextStyle::Body.resolve(ui.style());
                    let w = ui.fonts(|f| {
                        f.layout_no_wrap("Title:".to_string(), font_id, egui::Color32::WHITE)
                            .size()
                            .x
                    });
                    let r = ui
                        .allocate_ui_with_layout(
                            egui::vec2(w, 30.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| ui.label("Title:"),
                        )
                        .inner
                        .rect;
                    label_top = r.top();
                    label_bot = r.bottom();
                    let mut title = String::from("My Woman Left Me");
                    text_top = ui
                        .add_sized(
                            egui::vec2(180.0, 30.0),
                            egui::TextEdit::singleline(&mut title),
                        )
                        .rect
                        .top();
                    let mut genre = String::from("Rock / Blues");
                    combo_top = egui::ComboBox::from_id_salt("header_genre")
                        .selected_text(&genre)
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut genre,
                                "Rock / Blues".to_string(),
                                "Rock / Blues",
                            );
                        })
                        .response
                        .rect
                        .top();
                    btn_top = ui
                        .add_sized(egui::vec2(104.0, 30.0), egui::Button::new("Rhymes"))
                        .rect
                        .top();
                });
            });
        });
        assert!(
            (text_top - combo_top).abs() < 0.5,
            "combo top {combo_top} vs textedit top {text_top}"
        );
        assert!(
            (text_top - btn_top).abs() < 0.5,
            "button top {btn_top} vs textedit top {text_top}"
        );
        // The label is ~14px tall, centered in the 30px row, so its top
        // sits ~8px below the row top (same as the textedit top).
        let label_h = label_bot - label_top;
        assert!(
            (label_top - (text_top + (30.0 - label_h) / 2.0)).abs() < 0.5,
            "label top {label_top} (h {label_h}) vs textedit top {text_top}"
        );
    }

    #[test]
    fn section_card_header_controls_share_the_same_top_edge() {
        // Same treatment as the song header, applied to the section
        // cards: the header row is top-aligned, the time-signature /
        // label / chords fields are 28px tall to match the 28×28 icon
        // buttons, and the name heading + type tag are centered in 28px
        // boxes. All 28px controls share the same top edge, and the
        // chord chips row (30px buttons) is top-aligned the same way.
        let ctx = egui::Context::default();
        apply_theme(&ctx, Theme::Dark);
        let mut text_top = 0.0f32;
        let mut btn_top = 0.0f32;
        let mut chip_top = 0.0f32;
        let mut chip2_top = 0.0f32;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                // Section card header row.
                ui.horizontal_top(|ui| {
                    let mut ts = String::from("4/4");
                    text_top = ui
                        .add_sized(
                            egui::vec2(44.0, 28.0),
                            egui::TextEdit::singleline(&mut ts)
                                .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                                .horizontal_align(egui::Align::Center)
                                .vertical_align(egui::Align::Center),
                        )
                        .rect
                        .top();
                    btn_top = ui
                        .add_sized(egui::vec2(28.0, 28.0), egui::Button::new("▶"))
                        .rect
                        .top();
                });
                // Chord chips row.
                ui.horizontal_top(|ui| {
                    chip_top = ui.button("Em").rect.top();
                    chip2_top = ui.button("Fill").rect.top();
                });
            });
        });
        assert!(
            (text_top - btn_top).abs() < 0.5,
            "icon button top {btn_top} vs textedit top {text_top}"
        );
        assert!(
            (chip_top - chip2_top).abs() < 0.5,
            "chip top {chip_top} vs fill top {chip2_top}"
        );
    }

    #[test]
    fn setup_title_field_is_30px_with_centered_text() {
        // The song-title field on the setup screen gets the same
        // treatment as the Title field in the editor header: 30px tall,
        // text centered horizontally and vertically, 15.6pt font.
        let ctx = egui::Context::default();
        apply_theme(&ctx, Theme::Dark);
        let mut h = 0.0f32;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut title = String::from("My Woman Left Me");
                h = ui
                    .add_sized(
                        egui::vec2(320.0, 30.0),
                        egui::TextEdit::singleline(&mut title)
                            .font(egui::FontId::new(15.6, egui::FontFamily::Proportional))
                            .horizontal_align(egui::Align::Center)
                            .vertical_align(egui::Align::Center),
                    )
                    .rect
                    .height();
            });
        });
        assert!((h - 30.0).abs() < 0.5, "setup title height {h}");
    }

    #[test]
    fn menu_bar_combo_boxes_share_the_menu_button_top_edge() {
        // The Waveform/Transpose ComboBoxes in the menu bar sat 6px
        // lower than the File/Edit/View/Help buttons (a ComboBox wraps
        // its button in a second inner horizontal layout, and in the
        // menu bar's centered row that drifts it down). Each ComboBox
        // now sits in its own top-aligned 30px box, so it shares the
        // menu buttons' top edge. The box widths fit the longest
        // option: "Sawtooth Wave" is ~135px, "-2 (D std.)" is ~100px.
        let ctx = egui::Context::default();
        apply_theme(&ctx, Theme::Dark);
        let mut menu_top = 0.0f32;
        let mut combo_top = 0.0f32;
        let mut combo2_top = 0.0f32;
        let mut combo_w = 0.0f32;
        let mut trans_w = 0.0f32;
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                    egui::MenuBar::new()
                        .style(|style: &mut egui::Style| {
                            style.spacing.button_padding = egui::vec2(5.0, 6.0);
                            style.visuals.widgets.active.bg_stroke = egui::Stroke::NONE;
                            style.visuals.widgets.open.bg_stroke = egui::Stroke::NONE;
                            style.visuals.widgets.hovered.bg_stroke = egui::Stroke::NONE;
                            style.visuals.widgets.inactive.weak_bg_fill =
                                egui::Color32::TRANSPARENT;
                            style.visuals.widgets.inactive.bg_stroke = egui::Stroke::NONE;
                        })
                        .ui(ui, |ui| {
                            menu_top = ui
                                .menu_button("File", |ui| {
                                    ui.button("Save");
                                })
                                .response
                                .rect
                                .top();
                            ui.label("Waveform:");
                            let mut sel = String::from("Sawtooth Wave");
                            let r = ui
                                .allocate_ui_with_layout(
                                    egui::vec2(140.0, 30.0),
                                    egui::Layout::left_to_right(egui::Align::Min),
                                    |ui| {
                                        egui::ComboBox::from_id_salt("waveform")
                                            .selected_text(&sel)
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut sel,
                                                    "Sawtooth Wave".to_string(),
                                                    "Sawtooth Wave",
                                                );
                                            })
                                            .response
                                            .rect
                                    },
                                )
                                .inner;
                            combo_top = r.top();
                            combo_w = r.width();
                            ui.label("Transpose:");
                            let mut sel2 = String::from("-2 (D std.)");
                            let r2 = ui
                                .allocate_ui_with_layout(
                                    egui::vec2(105.0, 30.0),
                                    egui::Layout::left_to_right(egui::Align::Min),
                                    |ui| {
                                        egui::ComboBox::from_id_salt("transpose")
                                            .selected_text(&sel2)
                                            .show_ui(ui, |ui| {
                                                ui.selectable_value(
                                                    &mut sel2,
                                                    "-2 (D std.)".to_string(),
                                                    "-2 (D std.)",
                                                );
                                            })
                                            .response
                                            .rect
                                    },
                                )
                                .inner;
                            combo2_top = r2.top();
                            trans_w = r2.width();
                        });
                });
            });
        });
        assert!(
            (combo_top - menu_top).abs() < 0.5,
            "waveform combo top {combo_top} vs menu button top {menu_top}"
        );
        assert!(
            (combo2_top - menu_top).abs() < 0.5,
            "transpose combo top {combo2_top} vs menu button top {menu_top}"
        );
        assert!(
            combo_w <= 140.0,
            "waveform combo width {combo_w} exceeds its 140px box"
        );
        assert!(
            trans_w <= 105.0,
            "transpose combo width {trans_w} exceeds its 105px box"
        );
    }
}
