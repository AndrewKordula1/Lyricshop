//! Rhyme-scheme detection. Port of `RhymeScheme.kt`.
//! The last word of each non-blank line is grouped by perfect rhyme
//! (same CMU rhyme tail) and labelled A, B, C... in order of first
//! appearance. Words the dictionary doesn't know fall back to matching
//! only themselves, so identical line endings still group together.

use crate::rhyme;

/// e.g. lines ending "day / night / away / light" -> ["A", "B", "A", "B"]
pub fn detect_rhyme_scheme(lyrics: &str) -> Vec<String> {
    let last_words: Vec<String> = lyrics
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(|line| {
            let word = line
                .trim()
                .split_whitespace()
                .last()?
                .trim_matches(|c| {
                    matches!(
                        c,
                        ',' | '.' | '!' | '?' | ';' | ':' | '"' | '\'' | '(' | ')'
                    )
                })
                .to_lowercase();
            if word.is_empty() {
                None
            } else {
                Some(word)
            }
        })
        .collect();

    let mut key_to_letter: Vec<(String, String)> = Vec::new();
    let mut next_letter = b'A';
    last_words
        .iter()
        .map(|word| {
            let key = rhyme::rhyme_key_for(word).unwrap_or_else(|| word.clone());
            if let Some((_, letter)) = key_to_letter.iter().find(|(k, _)| *k == key) {
                letter.clone()
            } else {
                let letter = if next_letter > b'Z' {
                    "?".to_string()
                } else {
                    (next_letter as char).to_string()
                };
                next_letter += 1;
                key_to_letter.push((key, letter.clone()));
                letter
            }
        })
        .collect()
}

/// "A B A B", or "" when the section has fewer than two non-blank lines.
pub fn rhyme_scheme_label(lyrics: &str) -> String {
    let scheme = detect_rhyme_scheme(lyrics);
    if scheme.len() >= 2 {
        scheme.join(" ")
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn abab_scheme() {
        let lyrics = "end of the day\ninto the night\nfar far away\nturn out the light";
        assert_eq!(rhyme_scheme_label(lyrics), "A B A B");
    }

    #[test]
    fn identical_endings_group() {
        let lyrics = "same word here\nsomething else\nsame word here";
        let scheme = detect_rhyme_scheme(lyrics);
        assert_eq!(scheme, vec!["A", "B", "A"]);
    }

    #[test]
    fn fewer_than_two_lines_is_empty() {
        assert_eq!(rhyme_scheme_label("only one line"), "");
        assert_eq!(rhyme_scheme_label(""), "");
    }
}
