//! Dependency-free PDF export: builds a plain chord-chart PDF by hand.
//! Port of `PdfExporter.kt`. Letter page, Helvetica, title + metadata +
//! one block per section (label, chords, lyrics). No library needed: the
//! PDF format here is the simple subset every reader supports — a catalog,
//! a page tree, two base-14 fonts, and one content stream per page with
//! absolute-positioned text. Byte offsets for the cross-reference table
//! are computed from the real output at runtime, never hardcoded.

use crate::song::Song;

const PAGE_WIDTH: i32 = 612;
const PAGE_HEIGHT: i32 = 792;
const MARGIN: i32 = 56;
const TOP_Y: i32 = PAGE_HEIGHT - MARGIN;
const BOTTOM_Y: i32 = MARGIN;

#[derive(Debug, Clone)]
struct PdfLine {
    text: String,
    size: i32,
    bold: bool,
}

#[derive(Debug, Clone)]
struct PlacedLine {
    text: String,
    size: i32,
    bold: bool,
    y: i32,
}

pub fn export_song_to_pdf(song: &Song, path: &std::path::Path) -> bool {
    match std::fs::write(path, build_song_pdf(song)) {
        Ok(()) => true,
        Err(_) => false,
    }
}

fn build_song_pdf(song: &Song) -> Vec<u8> {
    let pages = paginate(build_lines(song));

    // Object numbers: 1 catalog, 2 pages, 3 Helvetica, 4 Helvetica-Bold,
    // then per page: page object 5 + 2i, content stream 6 + 2i.
    let page_obj_nums: Vec<usize> = (0..pages.len()).map(|i| 5 + 2 * i).collect();
    let content_obj_nums: Vec<usize> = (0..pages.len()).map(|i| 6 + 2 * i).collect();

    let mut objects: Vec<Vec<u8>> = Vec::new();
    objects.push(b"<< /Type /Catalog /Pages 2 0 R >>".to_vec());
    objects.push(
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            page_obj_nums
                .iter()
                .map(|n| format!("{} 0 R", n))
                .collect::<Vec<_>>()
                .join(" "),
            pages.len()
        )
        .into_bytes(),
    );
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_vec());
    objects.push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold >>".to_vec());

    for (index, lines) in pages.iter().enumerate() {
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {} {}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> >> /Contents {} 0 R >>",
                PAGE_WIDTH, PAGE_HEIGHT, content_obj_nums[index]
            )
            .into_bytes(),
        );
        let mut stream = String::new();
        for line in lines {
            let font = if line.bold { "F2" } else { "F1" };
            stream.push_str(&format!(
                "BT /{} {} Tf 1 0 0 1 {} {} Tm ({}) Tj ET\n",
                font,
                line.size,
                MARGIN,
                line.y,
                pdf_escape(&line.text)
            ));
        }
        let stream_bytes = stream.into_bytes();
        let mut obj = format!("<< /Length {} >>\nstream\n", stream_bytes.len()).into_bytes();
        obj.extend_from_slice(&stream_bytes);
        obj.extend_from_slice(b"\nendstream");
        objects.push(obj);
    }

    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n");
    // Binary marker comment, as the spec recommends.
    out.extend_from_slice(&[0x25, 0xE2, 0xE3, 0xCF, 0xD3, 0x0A]);

    let mut offsets: Vec<usize> = Vec::new();
    for (index, bytes) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", index + 1).as_bytes());
        out.extend_from_slice(bytes);
        out.extend_from_slice(b"\nendobj\n");
    }

    let xref_pos = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n", objects.len() + 1).as_bytes());
    out.extend_from_slice(b"0000000000 65535 f \n");
    for offset in &offsets {
        out.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{}\n%%EOF",
            objects.len() + 1,
            xref_pos
        )
        .as_bytes(),
    );
    out
}

fn build_lines(song: &Song) -> Vec<PdfLine> {
    let mut raw: Vec<PdfLine> = Vec::new();
    let title = if song.title.trim().is_empty() {
        "Untitled Song".to_string()
    } else {
        song.title.clone()
    };
    raw.push(PdfLine {
        text: title,
        size: 18,
        bold: true,
    });
    raw.push(PdfLine {
        text: format!(
            "Genre: {}  |  Key: {}  |  BPM: {}  |  Groove: {}",
            song.genre, song.key, song.bpm, song.groove
        ),
        size: 11,
        bold: false,
    });
    raw.push(PdfLine {
        text: String::new(),
        size: 12,
        bold: false,
    });
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
        raw.push(PdfLine {
            text: format!("[{}{}]", label, time_sig),
            size: 14,
            bold: true,
        });
        if let Some(chords) = &section.chords {
            if !chords.trim().is_empty() {
                raw.push(PdfLine {
                    text: format!("Chords: {}", chords),
                    size: 12,
                    bold: false,
                });
            }
        }
        let lyric_lines: Vec<&str> = section.lyrics.lines().collect();
        if lyric_lines.iter().all(|l| l.trim().is_empty()) {
            raw.push(PdfLine {
                text: "(Empty section)".to_string(),
                size: 12,
                bold: false,
            });
        } else {
            for l in lyric_lines {
                raw.push(PdfLine {
                    text: l.to_string(),
                    size: 12,
                    bold: false,
                });
            }
        }
        raw.push(PdfLine {
            text: String::new(),
            size: 12,
            bold: false,
        });
    }
    // Sanitize, then wrap lines that are too wide for the page.
    let mut out: Vec<PdfLine> = Vec::new();
    for line in raw {
        let clean = sanitize(&line.text);
        let max_chars = max_chars_for(line.size);
        if clean.chars().count() <= max_chars {
            out.push(PdfLine {
                text: clean,
                size: line.size,
                bold: line.bold,
            });
        } else {
            for w in wrap_text(&clean, max_chars) {
                out.push(PdfLine {
                    text: w,
                    size: line.size,
                    bold: line.bold,
                });
            }
        }
    }
    out
}

fn paginate(lines: Vec<PdfLine>) -> Vec<Vec<PlacedLine>> {
    let mut pages: Vec<Vec<PlacedLine>> = Vec::new();
    let mut current: Vec<PlacedLine> = Vec::new();
    let mut y = TOP_Y;
    for line in lines {
        let leading = (line.size as f64 * 1.4) as i32;
        if y - leading < BOTTOM_Y && !current.is_empty() {
            pages.push(std::mem::take(&mut current));
            y = TOP_Y;
        }
        y -= leading;
        current.push(PlacedLine {
            text: line.text,
            size: line.size,
            bold: line.bold,
            y,
        });
    }
    if !current.is_empty() {
        pages.push(current);
    }
    pages
}

/// Helvetica averages about half the font size per character.
fn max_chars_for(size: i32) -> usize {
    ((PAGE_WIDTH - 2 * MARGIN) as f64 / (size as f64 * 0.5)) as usize
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let words: Vec<&str> = text.split(' ').collect();
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in words {
        if current.is_empty() {
            if word.chars().count() > max_chars {
                let mut rest = word;
                while rest.chars().count() > max_chars {
                    let cut: String = rest.chars().take(max_chars).collect();
                    let cut_len = cut.len();
                    lines.push(cut);
                    rest = &rest[cut_len..];
                }
                current.push_str(rest);
            } else {
                current.push_str(word);
            }
        } else if current.chars().count() + 1 + word.chars().count() <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}

/// Base-14 Helvetica is WinAnsi: smart quotes, em dashes and accented
/// letters have to go (mapped where there is an obvious ASCII twin).
fn sanitize(text: &str) -> String {
    let mapped = text
        .replace('—', "-")
        .replace('–', "-")
        .replace('‘', "'")
        .replace('’', "'")
        .replace('“', "\"")
        .replace('”', "\"")
        .replace("…", "...");
    mapped
        .chars()
        .map(|c| if (' '..='~').contains(&c) { c } else { '?' })
        .collect()
}

fn pdf_escape(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            _ => out.push(ch),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::song::generate_sections;

    #[test]
    fn pdf_starts_with_header_and_has_xref() {
        let mut song = Song::new("Test Song", "Rock / Metal");
        song.sections = generate_sections("Rock / Metal");
        song.sections[1].lyrics = "One baby to another says".to_string();
        let bytes = build_song_pdf(&song);
        assert!(bytes.starts_with(b"%PDF-1.4"));
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.contains("Test Song"));
        assert!(text.contains("xref"));
        assert!(text.contains("%%EOF"));
        assert!(text.contains("/Helvetica"));
    }

    #[test]
    fn pdf_sanitizes_smart_quotes() {
        assert_eq!(sanitize("“hello” — it’s…"), "\"hello\" - it's...");
    }

    #[test]
    fn pdf_wraps_long_lines() {
        let wrapped = wrap_text("one two three four five six seven eight", 10);
        assert!(wrapped.len() > 1);
        assert!(wrapped.iter().all(|l| l.chars().count() <= 10));
    }
}
