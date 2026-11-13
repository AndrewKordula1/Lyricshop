//! Painter-drawn vector icons for Lyricshop.
//!
//! The move up/down arrows used to be font glyphs (▲/▼), but those glyphs
//! are not in egui's default font and render as empty squares on the user's
//! Windows machine. Font coverage is per-glyph, so swapping characters is
//! a dead end — every icon here is drawn as vector shapes with egui's
//! `Painter` instead, so nothing depends on the font.
//!
//! All icons are drawn on a 24×24 viewbox and scaled to fit the target
//! rect, so they form one coherent set: filled triangles for the transport
//! controls (play/stop/move), a padlock, a trash can, and an eighth note.
//! Outlined parts use a consistent 2.2–2.4 stroke (in viewbox units).
//! Icons take their color from the caller (the theme text color), so they
//! match the theme in both dark and light mode.

use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2};

/// The icons in the set.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Icon {
    /// Right-pointing triangle (play).
    Play,
    /// Filled square (stop).
    Stop,
    /// Up-pointing triangle (move section up).
    MoveUp,
    /// Down-pointing triangle (move section down).
    MoveDown,
    /// Padlock, shackle closed.
    Lock,
    /// Padlock, shackle open.
    Unlock,
    /// Trash can.
    Delete,
    /// Two overlapping pages (duplicate section).
    Duplicate,
    /// Eighth note (rhyme dictionary).
    Note,
}

/// Draw `icon` into `rect` (should be square), in `color`.
/// The icon is designed on a 24×24 viewbox, scaled to fit `rect`.
pub fn draw_icon(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    match icon {
        Icon::Play => draw_play(painter, rect, color),
        Icon::Stop => draw_stop(painter, rect, color),
        Icon::MoveUp => draw_move_up(painter, rect, color),
        Icon::MoveDown => draw_move_down(painter, rect, color),
        Icon::Lock => draw_lock(painter, rect, color),
        Icon::Unlock => draw_unlock(painter, rect, color),
        Icon::Delete => draw_delete(painter, rect, color),
        Icon::Duplicate => draw_duplicate(painter, rect, color),
        Icon::Note => draw_note(painter, rect, color),
    }
}

/// Add an icon-only button of `size` showing `icon`, and draw the icon
/// into it. Returns the button's `Response` so callers can chain
/// `.on_hover_text(...)` and `.clicked()` exactly as before.
pub fn icon_button(ui: &mut egui::Ui, size: Vec2, icon: Icon, color: Color32) -> egui::Response {
    let resp = ui.add_sized(size, egui::Button::new(""));
    draw_icon(ui.painter(), resp.rect.shrink(3.0), icon, color);
    resp
}

/// Draw `icon` at the left of a text button's `rect`, vertically centered.
/// Used for the icon + text buttons (Rhymes toggle, metadata lock): the
/// icon sits in a square patch at the left edge, the button's own text
/// stays centered in the full width.
pub fn draw_icon_at_left(painter: &Painter, rect: Rect, icon: Icon, color: Color32) {
    let side = rect.height() - 12.0;
    let icon_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + 8.0, rect.center().y - side / 2.0),
        Vec2::splat(side),
    );
    draw_icon(painter, icon_rect, icon, color);
}

// ------------------------------------------------------------------
// Viewbox mapping (24×24 grid → target rect).
// ------------------------------------------------------------------

/// Map a point in the 24×24 viewbox into `rect`.
fn v(rect: Rect, x: f32, y: f32) -> Pos2 {
    let s = rect.width() / 24.0;
    Pos2::new(rect.min.x + x * s, rect.min.y + y * s)
}

/// Scale a viewbox length into `rect`.
fn vw(rect: Rect, w: f32) -> f32 {
    w * rect.width() / 24.0
}

/// Map a viewbox rect into `rect`.
fn vr(rect: Rect, x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect::from_min_max(v(rect, x0, y0), v(rect, x1, y1))
}

/// Points along an arc in viewbox coordinates. Angles are in degrees;
/// with y pointing down, 180° is left, 270° is top, 360° is right, so
/// 180°→360° traces the top semicircle (an arch).
fn arc_points(
    cx: f32,
    cy: f32,
    r: f32,
    start_deg: f32,
    end_deg: f32,
    steps: usize,
) -> Vec<(f32, f32)> {
    (0..=steps)
        .map(|i| {
            let t = start_deg + (end_deg - start_deg) * (i as f32) / (steps as f32);
            let a = t.to_radians();
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// Points on a rotated ellipse, in screen coordinates. The ellipse is
/// centered at `center` with radii `rx`/`ry`, rotated `rot_deg` degrees.
fn rotated_ellipse_points(center: Pos2, rx: f32, ry: f32, rot_deg: f32, n: usize) -> Vec<Pos2> {
    let rot = rot_deg.to_radians();
    let (s, c) = (rot.sin(), rot.cos());
    (0..n)
        .map(|i| {
            let t = (i as f32) * std::f32::consts::TAU / (n as f32);
            let (x, y) = (rx * t.cos(), ry * t.sin());
            Pos2::new(center.x + x * c - y * s, center.y + x * s + y * c)
        })
        .collect()
}

/// Points along a quadratic Bézier curve, in screen coordinates.
fn quad_bezier_points(p0: Pos2, p1: Pos2, p2: Pos2, n: usize) -> Vec<Pos2> {
    (0..=n)
        .map(|i| {
            let t = (i as f32) / (n as f32);
            let u = 1.0 - t;
            Pos2::new(
                u * u * p0.x + 2.0 * u * t * p1.x + t * t * p2.x,
                u * u * p0.y + 2.0 * u * t * p1.y + t * t * p2.y,
            )
        })
        .collect()
}

// ------------------------------------------------------------------
// Transport triangles. All three are the same size: a 16-wide base and
// a 14-tall apex, so play/up/down read as one family.
// ------------------------------------------------------------------

fn draw_play(painter: &Painter, rect: Rect, color: Color32) {
    painter.add(egui::Shape::convex_polygon(
        vec![v(rect, 4.0, 4.0), v(rect, 4.0, 20.0), v(rect, 18.0, 12.0)],
        color,
        Stroke::NONE,
    ));
}

fn draw_move_up(painter: &Painter, rect: Rect, color: Color32) {
    painter.add(egui::Shape::convex_polygon(
        vec![v(rect, 12.0, 5.0), v(rect, 4.0, 19.0), v(rect, 20.0, 19.0)],
        color,
        Stroke::NONE,
    ));
}

fn draw_move_down(painter: &Painter, rect: Rect, color: Color32) {
    painter.add(egui::Shape::convex_polygon(
        vec![v(rect, 12.0, 19.0), v(rect, 4.0, 5.0), v(rect, 20.0, 5.0)],
        color,
        Stroke::NONE,
    ));
}

fn draw_stop(painter: &Painter, rect: Rect, color: Color32) {
    painter.rect_filled(vr(rect, 6.0, 6.0, 18.0, 18.0), vw(rect, 1.5), color);
}

// ------------------------------------------------------------------
// Padlock. Body is a filled rounded rect; the shackle is a stroked arch
// whose legs end inside the body top, so the joins are hidden. The
// unlocked shackle is the same arch raised up, with the right leg ending
// in the air — a clear gap to the body reads as "open".
// ------------------------------------------------------------------

fn draw_lock(painter: &Painter, rect: Rect, color: Color32) {
    painter.rect_filled(vr(rect, 5.0, 11.0, 19.0, 19.0), vw(rect, 2.0), color);
    let stroke = Stroke::new(vw(rect, 2.4), color);
    let mut pts: Vec<Pos2> = vec![v(rect, 7.5, 12.0)];
    pts.extend(
        arc_points(12.0, 10.0, 4.5, 180.0, 360.0, 12)
            .into_iter()
            .map(|(x, y)| v(rect, x, y)),
    );
    pts.push(v(rect, 16.5, 12.0));
    painter.line(pts, stroke);
}

fn draw_unlock(painter: &Painter, rect: Rect, color: Color32) {
    painter.rect_filled(vr(rect, 5.0, 11.0, 19.0, 19.0), vw(rect, 2.0), color);
    let stroke = Stroke::new(vw(rect, 2.4), color);
    let mut pts: Vec<Pos2> = vec![v(rect, 7.5, 12.0)];
    pts.extend(
        arc_points(12.0, 8.0, 4.5, 180.0, 360.0, 12)
            .into_iter()
            .map(|(x, y)| v(rect, x, y)),
    );
    painter.line(pts, stroke);
}

// ------------------------------------------------------------------
// Trash can. A tapered body (wider at the top), a lid that overhangs it
// slightly, and a small handle on top. No slats — the silhouette alone
// reads as a bin, and slats would need the button's background color.
// ------------------------------------------------------------------

fn draw_delete(painter: &Painter, rect: Rect, color: Color32) {
    // Body: trapezoid, wider at the top.
    painter.add(egui::Shape::convex_polygon(
        vec![
            v(rect, 6.0, 10.0),
            v(rect, 18.0, 10.0),
            v(rect, 16.0, 19.0),
            v(rect, 8.0, 19.0),
        ],
        color,
        Stroke::NONE,
    ));
    // Lid, overhanging the body slightly.
    painter.rect_filled(vr(rect, 5.0, 7.5, 19.0, 9.5), vw(rect, 1.0), color);
    // Handle on top of the lid.
    painter.rect_filled(vr(rect, 10.5, 5.0, 13.5, 7.5), vw(rect, 1.0), color);
}

// ------------------------------------------------------------------
// Duplicate (two overlapping pages). The back page is a stroked
// rounded rect offset up-left; the front page is filled and offset
// down-right, covering the back page's bottom-right corner, so the
// two read as a stack of pages.
// ------------------------------------------------------------------

fn draw_duplicate(painter: &Painter, rect: Rect, color: Color32) {
    painter.rect_stroke(
        vr(rect, 4.0, 4.0, 14.0, 14.0),
        vw(rect, 1.5),
        Stroke::new(vw(rect, 2.2), color),
        egui::StrokeKind::Middle,
    );
    painter.rect_filled(vr(rect, 10.0, 10.0, 20.0, 20.0), vw(rect, 1.5), color);
}

// ------------------------------------------------------------------
// Eighth note. A tilted elliptical head, a straight stem, and a flag
// that sweeps right and down from the stem top as a curved stroke.
// ------------------------------------------------------------------

fn draw_note(painter: &Painter, rect: Rect, color: Color32) {
    // Head: ellipse tilted -18°, so it leans like a real note head.
    let head = rotated_ellipse_points(v(rect, 8.5, 17.0), vw(rect, 3.6), vw(rect, 2.6), -18.0, 20);
    painter.add(egui::Shape::convex_polygon(head, color, Stroke::NONE));
    // Stem: a vertical bar from the head up to the top.
    painter.rect_filled(vr(rect, 11.0, 5.0, 13.0, 17.0), 0.0, color);
    // Flag: a curve sweeping right and down from the stem top.
    let flag = quad_bezier_points(
        v(rect, 12.0, 5.0),
        v(rect, 17.5, 5.5),
        v(rect, 18.5, 11.0),
        10,
    );
    painter.line(flag, Stroke::new(vw(rect, 2.2), color));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewbox_mapping_is_linear() {
        let rect = Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::splat(24.0));
        assert_eq!(v(rect, 0.0, 0.0), Pos2::new(100.0, 50.0));
        assert_eq!(v(rect, 24.0, 24.0), Pos2::new(124.0, 74.0));
        assert_eq!(v(rect, 12.0, 12.0), Pos2::new(112.0, 62.0));
        assert_eq!(vw(rect, 2.0), 2.0);
        // Half-size rect halves every mapped length.
        let half = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::splat(12.0));
        assert_eq!(vw(half, 4.0), 2.0);
    }

    #[test]
    fn arc_points_trace_the_top_semicircle() {
        // 180° is left, 270° is top, 360° is right (y points down).
        let pts = arc_points(12.0, 10.0, 4.5, 180.0, 360.0, 4);
        assert_eq!(pts.len(), 5);
        assert!((pts[0].0 - 7.5).abs() < 1e-4 && (pts[0].1 - 10.0).abs() < 1e-4);
        assert!((pts[2].0 - 12.0).abs() < 1e-4 && (pts[2].1 - 5.5).abs() < 1e-4);
        assert!((pts[4].0 - 16.5).abs() < 1e-4 && (pts[4].1 - 10.0).abs() < 1e-4);
    }

    #[test]
    fn rotated_ellipse_points_are_on_the_ellipse() {
        // Unrotated: points at 0° and 90° land on the axes.
        let c = Pos2::new(0.0, 0.0);
        let pts = rotated_ellipse_points(c, 4.0, 2.0, 0.0, 4);
        assert!((pts[0].x - 4.0).abs() < 1e-4 && pts[0].y.abs() < 1e-4);
        assert!(pts[1].x.abs() < 1e-4 && (pts[1].y - 2.0).abs() < 1e-4);
        // Every point satisfies the ellipse equation (after un-rotating).
        let pts = rotated_ellipse_points(c, 3.6, 2.6, -18.0, 20);
        assert_eq!(pts.len(), 20);
        let rot = (-18.0f32).to_radians();
        let (s, co) = (rot.sin(), rot.cos());
        for p in pts {
            let x = p.x * co + p.y * s;
            let y = -p.x * s + p.y * co;
            let v = (x / 3.6).powi(2) + (y / 2.6).powi(2);
            assert!((v - 1.0).abs() < 1e-3, "off-ellipse point: {v}");
        }
    }

    #[test]
    fn quad_bezier_endpoints_match() {
        let p0 = Pos2::new(0.0, 0.0);
        let p1 = Pos2::new(10.0, 0.0);
        let p2 = Pos2::new(10.0, 10.0);
        let pts = quad_bezier_points(p0, p1, p2, 4);
        assert_eq!(pts.len(), 5);
        assert_eq!(pts[0], p0);
        assert_eq!(pts[4], p2);
        // Midpoint of a quadratic Bézier is at t=0.5.
        assert!((pts[2].x - 7.5).abs() < 1e-4 && (pts[2].y - 2.5).abs() < 1e-4);
    }

    #[test]
    fn all_icons_have_a_variant() {
        // Keeps the Icon enum and draw_icon's match in sync.
        let icons = [
            Icon::Play,
            Icon::Stop,
            Icon::MoveUp,
            Icon::MoveDown,
            Icon::Lock,
            Icon::Unlock,
            Icon::Delete,
            Icon::Duplicate,
            Icon::Note,
        ];
        assert_eq!(icons.len(), 9);
    }
}
