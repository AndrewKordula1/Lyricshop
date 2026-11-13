//! Chord suggestions for a song's key. Port of `ChordSuggestions.kt`.
//! The key field is free text ("C Major", "E Minor", "F# minor"), so
//! parsing is deliberately forgiving.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParsedKey {
    pub root_semitone: i32,
    pub is_minor: bool,
}

const SHARP_NAMES: [&str; 12] = [
    "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
];

/// "C Major" -> C major, "E Minor" -> E minor, "A m" -> A minor,
/// "Bb" -> Bb major, "F# minor" -> F# minor. `None` when no note
/// letter leads.
pub fn parse_key(key_text: &str) -> Option<ParsedKey> {
    let trimmed = key_text.trim();
    if trimmed.is_empty() {
        return None;
    }
    let mut chars = trimmed.chars();
    let root_letter = chars.next()?.to_ascii_uppercase();
    let mut semitone: i32 = match root_letter {
        'C' => 0,
        'D' => 2,
        'E' => 4,
        'F' => 5,
        'G' => 7,
        'A' => 9,
        'B' => 11,
        _ => return None,
    };
    let mut rest = &trimmed[1..];
    if let Some(r) = rest.strip_prefix('#') {
        semitone += 1;
        rest = r;
    } else if let Some(r) = rest.strip_prefix('b') {
        semitone -= 1;
        rest = r;
    }
    let lower = rest.to_ascii_lowercase();
    let is_minor = lower.contains("min") || lower.trim() == "m";
    Some(ParsedKey {
        root_semitone: semitone.rem_euclid(12),
        is_minor,
    })
}

/// The seven diatonic chords, in scale order. C major: C Dm Em F G Am
/// Bdim. A minor: Am Bdim C Dm Em F G.
pub fn diatonic_chords(key: ParsedKey) -> Vec<String> {
    let (degrees, qualities): (&[i32], &[&str]) = if key.is_minor {
        (&[0, 2, 3, 5, 7, 8, 10], &["m", "dim", "", "m", "m", "", ""])
    } else {
        (&[0, 2, 4, 5, 7, 9, 11], &["", "m", "m", "", "", "m", "dim"])
    };
    degrees
        .iter()
        .enumerate()
        .map(|(i, d)| {
            format!(
                "{}{}",
                SHARP_NAMES[(key.root_semitone + d).rem_euclid(12) as usize],
                qualities[i]
            )
        })
        .collect()
}

/// Classic four-chord progressions, as "C - G - Am - F" strings, in order
/// of how often they show up. The UI cycles through them.
pub fn suggested_progressions(key: ParsedKey) -> Vec<String> {
    let chords = diatonic_chords(key);
    let patterns: &[&[usize]] = if key.is_minor {
        &[
            &[1, 6, 3, 7], // i - VI - III - VII
            &[1, 4, 6, 5], // i - iv - VI - v
            &[1, 7, 6, 7], // i - VII - VI - VII
            &[1, 3, 7, 6], // i - III - VII - VI
        ]
    } else {
        &[
            &[1, 5, 6, 4], // I - V - vi - IV
            &[1, 6, 4, 5], // I - vi - IV - V
            &[6, 4, 1, 5], // vi - IV - I - V
            &[1, 4, 5, 4], // I - IV - V - IV
        ]
    };
    patterns
        .iter()
        .map(|p| {
            p.iter()
                .map(|d| chords[d - 1].as_str())
                .collect::<Vec<_>>()
                .join(" - ")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_variants() {
        assert_eq!(
            parse_key("C Major"),
            Some(ParsedKey {
                root_semitone: 0,
                is_minor: false
            })
        );
        assert_eq!(
            parse_key("E Minor"),
            Some(ParsedKey {
                root_semitone: 4,
                is_minor: true
            })
        );
        assert_eq!(
            parse_key("F# minor"),
            Some(ParsedKey {
                root_semitone: 6,
                is_minor: true
            })
        );
        assert_eq!(
            parse_key("Bb"),
            Some(ParsedKey {
                root_semitone: 10,
                is_minor: false
            })
        );
        assert_eq!(parse_key(""), None);
        assert_eq!(parse_key("nonsense"), None);
    }

    #[test]
    fn diatonic_c_major() {
        let key = parse_key("C Major").unwrap();
        assert_eq!(
            diatonic_chords(key),
            vec!["C", "Dm", "Em", "F", "G", "Am", "Bdim"]
        );
    }

    #[test]
    fn diatonic_a_minor() {
        let key = parse_key("A Minor").unwrap();
        assert_eq!(
            diatonic_chords(key),
            vec!["Am", "Bdim", "C", "Dm", "Em", "F", "G"]
        );
    }

    #[test]
    fn progressions_cycle() {
        let key = parse_key("C Major").unwrap();
        let progs = suggested_progressions(key);
        assert_eq!(progs[0], "C - G - Am - F");
        assert_eq!(progs.len(), 4);
    }
}
