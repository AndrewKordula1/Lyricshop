//! Syllable counts per lyric line, for the meter display.
//! Port of `SyllableCounter.kt`. Words known to the CMU dictionary use
//! its real count; anything else gets a vowel-group estimate.

use crate::rhyme;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeterInfo {
    /// Syllables per non-blank line.
    pub counts: Vec<usize>,
    /// The section's usual line length, if one repeats.
    pub most_common: Option<usize>,
    /// 1-based positions (among non-blank lines) that differ.
    pub outlier_lines: Vec<usize>,
}

pub fn count_syllables_in_word(word: &str) -> usize {
    let clean = word
        .trim_matches(|c| {
            matches!(
                c,
                ',' | '.' | '!' | '?' | ';' | ':' | '"' | '\'' | '(' | ')'
            )
        })
        .to_lowercase();
    if clean.is_empty() {
        return 0;
    }
    rhyme::syllable_count_for(&clean).unwrap_or_else(|| estimate_syllables(&clean))
}

pub fn count_syllables_in_line(line: &str) -> usize {
    line.split_whitespace().map(count_syllables_in_word).sum()
}

/// Counts vowel groups ("beau-ti-ful" = 3). A silent trailing "e" doesn't
/// count, except after "l" ("table" keeps its second syllable). Good enough
/// for a meter hint; the dictionary wins whenever it knows the word.
pub fn estimate_syllables(word: &str) -> usize {
    let letters: String = word.chars().filter(|c| c.is_ascii_lowercase()).collect();
    if letters.is_empty() {
        return 0;
    }
    let mut groups = 0;
    let mut prev_vowel = false;
    for ch in letters.chars() {
        let is_vowel = matches!(ch, 'a' | 'e' | 'i' | 'o' | 'u' | 'y');
        if is_vowel && !prev_vowel {
            groups += 1;
        }
        prev_vowel = is_vowel;
    }
    if letters.ends_with('e') && groups > 1 && !letters.ends_with("le") {
        groups -= 1;
    }
    groups.max(1)
}

pub fn analyze_meter(lyrics: &str) -> MeterInfo {
    let counts: Vec<usize> = lyrics
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(count_syllables_in_line)
        .collect();
    let most_common = {
        let mut freq: Vec<(usize, usize)> = Vec::new();
        for &c in &counts {
            if let Some(e) = freq.iter_mut().find(|(v, _)| *v == c) {
                e.1 += 1;
            } else {
                freq.push((c, 1));
            }
        }
        freq.into_iter()
            .max_by_key(|(_, n)| *n)
            .filter(|(_, n)| *n > 1)
            .map(|(v, _)| v)
    };
    let outlier_lines = match most_common {
        Some(common) => counts
            .iter()
            .enumerate()
            .filter(|(_, &c)| c != common)
            .map(|(i, _)| i + 1)
            .collect(),
        None => Vec::new(),
    };
    MeterInfo {
        counts,
        most_common,
        outlier_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_vowel_groups() {
        assert_eq!(estimate_syllables("beautiful"), 3);
        assert_eq!(estimate_syllables("table"), 2);
        assert_eq!(estimate_syllables("love"), 1);
        assert_eq!(estimate_syllables("night"), 1);
        assert_eq!(estimate_syllables("hello"), 2);
    }

    #[test]
    fn meter_counts_and_outliers() {
        let lyrics = "Twinkle twinkle little star\nHow I wonder what you are\nUp above the world so high\nLike a diamond in the sky";
        let m = analyze_meter(lyrics);
        assert_eq!(m.counts.len(), 4);
        // All four lines are 6-7 syllables; the usual length repeats.
        assert!(m.most_common.is_some());
    }

    #[test]
    fn meter_single_line_has_no_usual() {
        let m = analyze_meter("one line only");
        assert_eq!(m.counts.len(), 1);
        assert_eq!(m.most_common, None);
        assert!(m.outlier_lines.is_empty());
    }
}
