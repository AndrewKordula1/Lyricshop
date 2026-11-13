//! Chord parsing and progression splitting.
//! Port of the parsing half of `ChordPlayer.kt`, including the power-chord
//! fix: "5" chords play root–fifth–octave (B5 = B–F#–B, MIDI 71, 78, 83),
//! not a major triad, and are never silently skipped.
//!
//! Notes are MIDI numbers around middle C (C4 = 60), exactly like the
//! Kotlin version, so the same chord symbols produce the same notes.

const ROOT_SEMITONES: [(char, i32); 7] = [
    ('C', 0),
    ('D', 2),
    ('E', 4),
    ('F', 5),
    ('G', 7),
    ('A', 9),
    ('B', 11),
];

fn root_semitone(letter: char) -> Option<i32> {
    ROOT_SEMITONES
        .iter()
        .find(|(c, _)| *c == letter)
        .map(|(_, s)| *s)
}

/// Parse one chord symbol ("C", "Am", "G7", "Dm7", "Bdim", "B5", "C/E",
/// "F#maj7") into MIDI note numbers. Returns `None` when the symbol is
/// not understood, so unparseable tokens are skipped, not crashed on.
pub fn parse_chord_notes(symbol: &str) -> Option<Vec<i32>> {
    let trimmed = symbol.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("N.C.") {
        return None;
    }

    let mut parts = trimmed.splitn(2, '/');
    let main = parts.next().unwrap_or("");
    let bass = parts.next();
    if main.is_empty() {
        return None;
    }

    let mut chars = main.chars();
    let root_letter = chars.next()?.to_ascii_uppercase();
    let mut semitone = root_semitone(root_letter)?;
    let mut index = 1;
    let main_chars: Vec<char> = main.chars().collect();
    if index < main_chars.len() && main_chars[index] == '#' {
        semitone += 1;
        index += 1;
    } else if index < main_chars.len() && (main_chars[index] == 'b' || main_chars[index] == '♭') {
        semitone -= 1;
        index += 1;
    }
    let suffix: String = main_chars[index..].iter().collect();

    // Longest suffixes first: "m" must not swallow "maj7" or "min7".
    let intervals: Vec<i32> = if suffix.starts_with("maj7") {
        vec![0, 4, 7, 11]
    } else if suffix.starts_with("m7") || suffix.starts_with("min7") {
        vec![0, 3, 7, 10]
    } else if suffix.starts_with("dim7") {
        vec![0, 3, 6, 9]
    } else if suffix.starts_with("dim") || suffix == "°" {
        vec![0, 3, 6]
    } else if suffix.starts_with("aug") || suffix == "+" {
        vec![0, 4, 8]
    } else if suffix.starts_with("sus4") || suffix == "sus" {
        vec![0, 5, 7]
    } else if suffix.starts_with("sus2") {
        vec![0, 2, 7]
    } else if suffix.starts_with("maj") {
        vec![0, 4, 7]
    } else if suffix.starts_with("min") {
        vec![0, 3, 7]
    } else if suffix.starts_with('m') {
        vec![0, 3, 7]
    } else if suffix.starts_with('7') {
        vec![0, 4, 7, 10]
    } else if suffix.starts_with('6') {
        vec![0, 4, 7, 9]
    } else if suffix.starts_with('9') {
        vec![0, 4, 7, 10, 14]
    } else if suffix == "5" {
        // Power chord: root, fifth, and octave — no third, which is the
        // whole point. "B5" is B-F#-B, not B major.
        vec![0, 7, 12]
    } else if suffix.is_empty() {
        vec![0, 4, 7]
    } else {
        return None;
    };

    let root = 60 + semitone.rem_euclid(12);
    let mut notes: Vec<i32> = intervals.iter().map(|i| root + i).collect();

    // Slash chord: put the bass note an octave below the chord.
    if let Some(bass) = bass {
        let bass_text = bass.trim();
        if let Some(bass_letter) = bass_text.chars().next().map(|c| c.to_ascii_uppercase()) {
            if let Some(mut bass_semi) = root_semitone(bass_letter) {
                let rest = &bass_text[1..];
                if rest.starts_with('#') {
                    bass_semi += 1;
                } else if rest.starts_with('b') {
                    bass_semi -= 1;
                }
                notes.insert(0, 48 + bass_semi.rem_euclid(12));
            }
        }
    }
    Some(notes)
}

/// Split a chord field ("C - G - Am - F", "C, G, Am, F", "C G Am F") into
/// one note list per chord, dropping anything unparseable.
pub fn parse_progression(text: &str) -> Vec<Vec<i32>> {
    text.split(|c: char| c == ',' || c == '-' || c.is_whitespace())
        .filter(|t| !t.trim().is_empty())
        .filter_map(parse_chord_notes)
        .collect()
}

/// Split a chord field into bars. "|" marks a bar line; the chords inside
/// a bar share it evenly. With no "|" at all, every chord is its own bar.
///
/// "C G Am F | Dm G C" -> [[C, G, Am, F], [Dm, G, C]]
/// "C - G - Am - F"    -> [[C], [G], [Am], [F]]
pub fn parse_bars(text: &str) -> Vec<Vec<Vec<i32>>> {
    let bar_texts: Vec<&str> = text.split('|').collect();
    if bar_texts.len() == 1 {
        return parse_progression(text)
            .into_iter()
            .map(|c| vec![c])
            .collect();
    }
    bar_texts
        .into_iter()
        .map(parse_progression)
        .filter(|b| !b.is_empty())
        .collect()
}

/// Quarter-note beats per bar from a time signature: "4/4" -> 4, "3/4" -> 3,
/// "6/8" -> 3, "12/8" -> 6. Anything missing or unparseable is 4/4.
pub fn quarter_beats_per_bar(time_signature: Option<&str>) -> f64 {
    let text = match time_signature {
        Some(t) => t.trim(),
        None => return 4.0,
    };
    if text.is_empty() {
        return 4.0;
    }
    let mut parts = text.split('/');
    let top: f64 = match parts.next().and_then(|p| p.parse().ok()) {
        Some(v) => v,
        None => return 4.0,
    };
    let bottom: f64 = match parts.next() {
        Some(p) => match p.parse().ok() {
            Some(v) => v,
            None => return 4.0,
        },
        None => return top,
    };
    if bottom == 0.0 {
        return 4.0;
    }
    top * 4.0 / bottom
}

/// Shift a note list by semitones (playback transpose), clamping to the
/// MIDI range. The written chords are unchanged.
pub fn transpose_notes(notes: &[i32], semitones: i32) -> Vec<i32> {
    notes
        .iter()
        .map(|n| (n + semitones).clamp(0, 127))
        .collect()
}

/// MIDI note number to frequency in Hz (A4 = 440).
pub fn midi_to_freq(midi: i32) -> f64 {
    440.0 * 2f64.powf((midi as f64 - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_chord_is_root_fifth_octave() {
        // B5 = B–F#–B (MIDI 71, 78, 83) — no third.
        assert_eq!(parse_chord_notes("B5"), Some(vec![71, 78, 83]));
        assert_eq!(parse_chord_notes("A5"), Some(vec![69, 76, 81]));
        assert_eq!(parse_chord_notes("C#5"), Some(vec![61, 68, 73]));
        assert_eq!(parse_chord_notes("F#5"), Some(vec![66, 73, 78]));
    }

    #[test]
    fn plain_major_is_a_full_triad() {
        // A plain "B" still plays the full major triad B–D#–F#.
        assert_eq!(parse_chord_notes("B"), Some(vec![71, 75, 78]));
        assert_eq!(parse_chord_notes("C"), Some(vec![60, 64, 67]));
    }

    #[test]
    fn drain_you_verse_parses() {
        // Nirvana's "Drain You" verse: A5–C#5–F#5–B5. The Kotlin bug was
        // that these were silently skipped, so the song played nothing.
        let prog = parse_progression("A5 C#5 F#5 B5");
        assert_eq!(prog.len(), 4);
        assert_eq!(prog[0], vec![69, 76, 81]);
        assert_eq!(prog[3], vec![71, 78, 83]);
    }

    #[test]
    fn chord_variants() {
        assert_eq!(parse_chord_notes("Am"), Some(vec![69, 72, 76]));
        assert_eq!(parse_chord_notes("G7"), Some(vec![67, 71, 74, 77]));
        assert_eq!(parse_chord_notes("Dm7"), Some(vec![62, 65, 69, 72]));
        assert_eq!(parse_chord_notes("Bdim"), Some(vec![71, 74, 77]));
        assert_eq!(parse_chord_notes("F#maj7"), Some(vec![66, 70, 73, 77]));
        assert_eq!(parse_chord_notes("C/E"), Some(vec![52, 60, 64, 67]));
        assert_eq!(parse_chord_notes("Csus"), Some(vec![60, 65, 67]));
        assert_eq!(parse_chord_notes("C6"), Some(vec![60, 64, 67, 69]));
        assert_eq!(parse_chord_notes("C9"), Some(vec![60, 64, 67, 70, 74]));
    }

    #[test]
    fn unparseable_is_skipped_not_crash() {
        assert_eq!(parse_chord_notes(""), None);
        assert_eq!(parse_chord_notes("N.C."), None);
        assert_eq!(parse_chord_notes("Zz9"), None);
        assert_eq!(parse_chord_notes("H"), None);
        let prog = parse_progression("C Zz9 G");
        assert_eq!(prog.len(), 2);
    }

    #[test]
    fn bars_split_on_pipe() {
        let bars = parse_bars("C G Am F | Dm G C");
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].len(), 4);
        assert_eq!(bars[1].len(), 3);
        // No pipe: every chord is its own bar.
        let bars = parse_bars("C - G - Am - F");
        assert_eq!(bars.len(), 4);
        assert!(bars.iter().all(|b| b.len() == 1));
    }

    #[test]
    fn time_signature_beats() {
        assert_eq!(quarter_beats_per_bar(Some("4/4")), 4.0);
        assert_eq!(quarter_beats_per_bar(Some("3/4")), 3.0);
        assert_eq!(quarter_beats_per_bar(Some("6/8")), 3.0);
        assert_eq!(quarter_beats_per_bar(Some("12/8")), 6.0);
        assert_eq!(quarter_beats_per_bar(None), 4.0);
        assert_eq!(quarter_beats_per_bar(Some("bogus")), 4.0);
        assert_eq!(quarter_beats_per_bar(Some("")), 4.0);
    }

    #[test]
    fn transpose_shifts_and_clamps() {
        assert_eq!(transpose_notes(&[71, 78, 83], -2), vec![69, 76, 81]);
        assert_eq!(transpose_notes(&[71, 78, 83], 6), vec![77, 84, 89]);
        assert_eq!(transpose_notes(&[0, 1], -6), vec![0, 0]);
        assert_eq!(transpose_notes(&[126, 127], 6), vec![127, 127]);
    }

    #[test]
    fn midi_to_freq_a4_is_440() {
        assert!((midi_to_freq(69) - 440.0).abs() < 0.01);
        assert!((midi_to_freq(60) - 261.63).abs() < 0.1);
    }
}
