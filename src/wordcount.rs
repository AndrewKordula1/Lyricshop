//! Word counts. Port of `WordCount.kt`.

use crate::song::SongSection;

/// Words in one text (split on whitespace).
pub fn count_words(text: &str) -> usize {
    text.split_whitespace().count()
}

/// Words across all sections' lyrics.
pub fn count_words_total(sections: &[SongSection]) -> usize {
    sections.iter().map(|s| count_words(&s.lyrics)).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_words() {
        assert_eq!(count_words(""), 0);
        assert_eq!(count_words("hello world"), 2);
        assert_eq!(count_words("  one   two\nthree  "), 3);
    }
}
