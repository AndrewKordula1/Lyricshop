//! CMU rhyme engine. Port of `CmuRhymeEngine.kt`.
//!
//! The dictionary (`assets/cmudict.dict`, the CMU Pronouncing Dictionary,
//! same file the Kotlin app loads from resources) is **embedded** in the
//! binary with `include_str!`, so rhymes work with no data file to install
//! or lose. It is parsed lazily on first use, so startup stays fast; the
//! meter and rhyme-scheme features fall back to estimates until it loads,
//! exactly like the Kotlin app.

use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RhymeMode {
    Perfect,
    Slant,
    Assonance,
}

impl RhymeMode {
    pub fn label(&self) -> &'static str {
        match self {
            RhymeMode::Perfect => "Perfect",
            RhymeMode::Slant => "Slant",
            RhymeMode::Assonance => "Assonance",
        }
    }

    pub fn all() -> &'static [RhymeMode] {
        &[RhymeMode::Perfect, RhymeMode::Slant, RhymeMode::Assonance]
    }
}

#[derive(Debug, Clone)]
struct WordData {
    word: String,
    tail: Option<String>,
    stressed_vowel: Option<String>,
    syllable_count: usize,
}

pub(crate) struct Engine {
    word_data: HashMap<String, WordData>,
    tail_to_words: HashMap<String, Vec<String>>,
    vowel_to_words: HashMap<String, Vec<String>>,
}

static ENGINE: OnceLock<Engine> = OnceLock::new();

const COMMON_WORD_PRIORITIES: &[&str] = &[
    "day", "say", "way", "play", "stay", "may", "pay", "delay", "today", "away", "pray", "ray",
    "clay", "gray", "tray", "night", "light", "fight", "sight", "flight", "bright", "might",
    "white", "right", "tight", "height", "time", "prime", "climb", "crime", "chime", "dime",
    "heart", "part", "art", "start", "smart", "chart", "dart", "soul", "roll", "goal", "hole",
    "toll", "whole", "coal", "pain", "train", "gain", "brain", "chain", "plain", "stain", "reign",
    "lane", "main", "real", "deal", "heal", "steel", "wheel", "feel", "seal", "true", "new",
    "blue", "through", "you", "too", "view", "clue", "crew", "fire", "wire", "higher", "desire",
    "tire", "liar",
];

/// The CMU phonemes for a letter's *name* (stress digits stripped), so
/// an entry pronounced letter-by-letter can be recognised as an
/// initialism: FBI is "EH F B IY AY", exactly F + B + I. A real word
/// never matches its own letters' names spelled out ("bee" is "B IY",
/// not B + E + E = "B IY IY IY"), so this does not catch ordinary words.
/// Returns `None` for a character with no letter name.
fn letter_name_phonemes(c: char) -> Option<&'static [&'static str]> {
    Some(match c {
        'a' => &["EY"],
        'b' => &["B", "IY"],
        'c' => &["S", "IY"],
        'd' => &["D", "IY"],
        'e' => &["IY"],
        'f' => &["EH", "F"],
        'g' => &["JH", "IY"],
        'h' => &["EY", "CH"],
        'i' => &["AY"],
        'j' => &["JH", "EY"],
        'k' => &["K", "EY"],
        'l' => &["EH", "L"],
        'm' => &["EH", "M"],
        'n' => &["EH", "N"],
        'o' => &["OW"],
        'p' => &["P", "IY"],
        'q' => &["K", "Y", "UW"],
        'r' => &["AA", "R"],
        's' => &["EH", "S"],
        't' => &["T", "IY"],
        'u' => &["Y", "UW"],
        'v' => &["V", "IY"],
        'w' => &["D", "AH", "B", "AH", "L", "Y", "UW"],
        'x' => &["EH", "K", "S"],
        'y' => &["W", "AY"],
        'z' => &["Z", "IY"],
        _ => return None,
    })
}

/// Whether a dictionary entry is an initialism — a word pronounced by
/// spelling out its letters (FBI, CIA, NSA) rather than as a word. The
/// entry's phonemes, with stress digits stripped, must equal the
/// concatenation of its letters' names. Pure, so it is unit-testable.
fn is_initialism(word: &str, phonemes: &str) -> bool {
    if word.len() < 2 {
        return false;
    }
    let actual: Vec<&str> = phonemes
        .split(' ')
        .map(|p| p.trim_end_matches(['0', '1', '2']))
        .collect();
    let mut expected: Vec<&str> = Vec::new();
    for c in word.chars() {
        match letter_name_phonemes(c) {
            Some(phones) => expected.extend_from_slice(phones),
            None => return false,
        }
    }
    actual == expected
}

fn build_engine() -> Engine {
    let mut word_data: HashMap<String, WordData> = HashMap::new();
    let mut tail_to_words: HashMap<String, Vec<String>> = HashMap::new();
    let mut vowel_to_words: HashMap<String, Vec<String>> = HashMap::new();

    for line in include_str!("../assets/cmudict.dict").lines() {
        if line.starts_with(";;;") {
            continue;
        }
        let mut parts = line.splitn(2, char::is_whitespace);
        let raw_word = match parts.next() {
            Some(w) => w,
            None => continue,
        };
        let phonemes = match parts.next() {
            Some(p) => p.trim(),
            None => continue,
        };
        // Strip variant markers like "WORD(2)".
        let word: String = raw_word
            .to_lowercase()
            .chars()
            .take_while(|c| *c != '(')
            .collect::<String>()
            .trim()
            .to_string();
        if word.len() < 2 || !word.chars().all(|c| c.is_ascii_lowercase()) {
            continue;
        }
        // Skip initialisms (FBI, CIA, NSA): entries pronounced by
        // spelling out their letters, not words a songwriter would use.
        if is_initialism(&word, phonemes) {
            continue;
        }
        let tail = extract_rhyme_tail(phonemes);
        let stressed_vowel = extract_stressed_vowel(phonemes);
        let syllable_count = count_syllables(phonemes);
        let data = WordData {
            word: word.clone(),
            tail: tail.clone(),
            stressed_vowel: stressed_vowel.clone(),
            syllable_count,
        };
        word_data.insert(word.clone(), data);
        if let Some(t) = tail {
            let entry = tail_to_words.entry(t).or_default();
            if !entry.contains(&word) {
                entry.push(word.clone());
            }
        }
        if let Some(v) = stressed_vowel {
            let entry = vowel_to_words.entry(v).or_default();
            if !entry.contains(&word) {
                entry.push(word);
            }
        }
    }

    Engine {
        word_data,
        tail_to_words,
        vowel_to_words,
    }
}

/// The loaded engine. Parses the embedded dictionary on first call.
pub(crate) fn engine() -> &'static Engine {
    ENGINE.get_or_init(build_engine)
}

/// Number of words in the loaded dictionary (for tests / status).
#[allow(dead_code)]
pub fn word_count() -> usize {
    engine().word_data.len()
}

fn extract_rhyme_tail(phonemes: &str) -> Option<String> {
    let list: Vec<&str> = phonemes.split(' ').collect();
    let last_stress = list.iter().rposition(|p| p.contains('1'))?;
    Some(list[last_stress..].join(" "))
}

fn extract_stressed_vowel(phonemes: &str) -> Option<String> {
    phonemes
        .split(' ')
        .find(|p| p.contains('1'))
        .map(|s| s.to_string())
}

fn count_syllables(phonemes: &str) -> usize {
    phonemes
        .split(' ')
        .filter(|p| p.chars().any(|c| ('0'..='2').contains(&c)))
        .count()
}

/// Find rhymes for a word, ranked like the Kotlin app: syllable-count
/// match first, then common lyric words, then length, then alphabetical.
/// At most 30 results.
pub fn find_rhymes(input_word: &str, mode: RhymeMode) -> Vec<String> {
    let eng = engine();
    let clean = input_word.to_lowercase().trim().to_string();
    if clean.len() < 2 {
        return Vec::new();
    }
    let input_data = eng.word_data.get(&clean);
    let target_syllables = input_data.map(|d| d.syllable_count).unwrap_or(1);

    let raw_matches: Vec<String> = match mode {
        RhymeMode::Perfect => {
            let tail = input_data
                .and_then(|d| d.tail.clone())
                .or_else(|| get_fallback_tail(&clean));
            match tail {
                Some(t) => eng
                    .tail_to_words
                    .get(&t)
                    .map(|v| v.iter().filter(|w| *w != &clean).cloned().collect())
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        }
        RhymeMode::Assonance => match input_data.and_then(|d| d.stressed_vowel.clone()) {
            Some(v) => eng
                .vowel_to_words
                .get(&v)
                .map(|words| words.iter().filter(|w| *w != &clean).cloned().collect())
                .unwrap_or_default(),
            None => Vec::new(),
        },
        RhymeMode::Slant => {
            let vowel = input_data.and_then(|d| d.stressed_vowel.clone());
            let exact_tail = input_data.and_then(|d| d.tail.clone());
            match vowel {
                Some(v) => eng
                    .vowel_to_words
                    .get(&v)
                    .map(|words| {
                        words
                            .iter()
                            .filter(|w| {
                                *w != &clean
                                    && eng.word_data.get(*w).and_then(|d| d.tail.clone())
                                        != exact_tail
                            })
                            .cloned()
                            .collect()
                    })
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        }
    };

    let mut matches: Vec<String> = raw_matches
        .into_iter()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    matches.sort_by(|a, b| {
        let syll_a = eng.word_data.get(a).map(|d| d.syllable_count) == Some(target_syllables);
        let syll_b = eng.word_data.get(b).map(|d| d.syllable_count) == Some(target_syllables);
        let prio_a = COMMON_WORD_PRIORITIES.contains(&a.as_str());
        let prio_b = COMMON_WORD_PRIORITIES.contains(&b.as_str());
        let noise_a = a.len() == 2 && !prio_a;
        let noise_b = b.len() == 2 && !prio_b;
        (!syll_a)
            .cmp(&(!syll_b))
            .then((!prio_a).cmp(&(!prio_b)))
            .then(noise_a.cmp(&noise_b))
            .then(a.len().cmp(&b.len()))
            .then(a.cmp(b))
    });
    matches.truncate(30);
    matches
}

/// Syllable count for one word from the dictionary, or `None` when the
/// word is not in it (callers fall back to a vowel-group estimate).
pub fn syllable_count_for(word: &str) -> Option<usize> {
    let clean = word.to_lowercase();
    let clean = clean.trim();
    engine().word_data.get(clean).map(|d| d.syllable_count)
}

/// Whether a rhyme word's syllable count passes the rhyme panel's
/// filter. `None` = Any (show all). `Some(n)` for 1..=4 = exactly `n`
/// syllables; `Some(5)` = 5 or more syllables. A word whose count is
/// unknown (`None`, not in the dictionary) only passes the Any filter.
/// Pure, so it is unit-testable without loading the dictionary.
pub(crate) fn syllable_count_matches_filter(count: Option<usize>, filter: Option<usize>) -> bool {
    match filter {
        None => true,
        Some(n) => match count {
            Some(c) if n >= 5 => c >= 5,
            Some(c) => c == n,
            None => false,
        },
    }
}

/// Display label for a syllable filter value, for the rhyme panel's
/// ComboBox. The filter values are `None` (Any) and `Some(1)`..`Some(5)`,
/// with 5 meaning "5 or more".
pub(crate) fn syllable_filter_label(filter: Option<usize>) -> String {
    match filter {
        None => "Any".to_string(),
        Some(1) => "1 syllable".to_string(),
        Some(n) if n >= 5 => "5+ syllables".to_string(),
        Some(n) => format!("{n} syllables"),
    }
}

/// The filter values offered in the rhyme panel's ComboBox, in order.
pub(crate) const SYLLABLE_FILTER_OPTIONS: [Option<usize>; 6] =
    [None, Some(1), Some(2), Some(3), Some(4), Some(5)];

/// The perfect-rhyme key for a word: the phonemes from its last stressed
/// vowel onward. `None` when the word is unknown and no fallback tail
/// could be found.
pub fn rhyme_key_for(word: &str) -> Option<String> {
    let clean = word.to_lowercase();
    let clean = clean.trim();
    let eng = engine();
    eng.word_data
        .get(clean)
        .and_then(|d| d.tail.clone())
        .or_else(|| get_fallback_tail(clean))
}

fn get_fallback_tail(clean_word: &str) -> Option<String> {
    let suffix_len = clean_word.len().min(3);
    let suffix = &clean_word[clean_word.len() - suffix_len..];
    engine()
        .word_data
        .values()
        .find(|d| d.word.ends_with(suffix) && d.tail.is_some())
        .and_then(|d| d.tail.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dictionary_loads_with_many_words() {
        assert!(word_count() > 100_000);
    }

    #[test]
    fn initialism_detection() {
        // FBI is EH F + B IY + AY, spelled out letter by letter.
        assert!(is_initialism("fbi", "EH1 F B IY1 AY1"));
        assert!(is_initialism("cia", "S IY1 AY1 EY1"));
        // Real words never match their letters' names spelled out.
        assert!(!is_initialism("bee", "B IY1"));
        assert!(!is_initialism("cat", "K AE1 T"));
        assert!(!is_initialism("day", "D EY1"));
        assert!(!is_initialism("see", "S IY1"));
        assert!(!is_initialism("nasa", "N AE1 S AH0"));
    }

    #[test]
    fn acronyms_are_not_in_the_dictionary() {
        assert_eq!(syllable_count_for("fbi"), None);
        assert_eq!(syllable_count_for("cia"), None);
        assert_eq!(syllable_count_for("nsa"), None);
        // And they never show up as rhymes.
        let eye = find_rhymes("eye", RhymeMode::Perfect);
        assert!(!eye.contains(&"fbi".to_string()));
        assert!(!eye.contains(&"cia".to_string()));
        // Ordinary words are untouched.
        assert_eq!(syllable_count_for("bee"), Some(1));
        assert_eq!(syllable_count_for("day"), Some(1));
        assert!(find_rhymes("day", RhymeMode::Perfect).contains(&"say".to_string()));
    }

    #[test]
    fn perfect_rhymes_for_day() {
        let rhymes = find_rhymes("day", RhymeMode::Perfect);
        assert!(!rhymes.is_empty());
        assert!(rhymes.contains(&"say".to_string()));
        assert!(rhymes.contains(&"way".to_string()));
        assert!(rhymes.contains(&"play".to_string()));
        assert!(!rhymes.contains(&"day".to_string()));
        assert!(rhymes.len() <= 30);
    }

    #[test]
    fn perfect_rhymes_for_night() {
        let rhymes = find_rhymes("night", RhymeMode::Perfect);
        assert!(rhymes.contains(&"light".to_string()));
        assert!(rhymes.contains(&"fight".to_string()));
    }

    #[test]
    fn assonance_and_slant_differ() {
        let assonance = find_rhymes("day", RhymeMode::Assonance);
        let slant = find_rhymes("day", RhymeMode::Slant);
        assert!(!assonance.is_empty());
        assert!(!slant.is_empty());
        // Slant rhymes share the vowel but not the exact tail.
        let eng = engine();
        let day_tail = eng.word_data.get("day").and_then(|d| d.tail.clone());
        for w in &slant {
            let t = eng.word_data.get(w).and_then(|d| d.tail.clone());
            assert_ne!(t, day_tail, "{w} should not share day's tail");
        }
    }

    #[test]
    fn syllable_counts_from_dictionary() {
        assert_eq!(syllable_count_for("beautiful"), Some(3));
        assert_eq!(syllable_count_for("hello"), Some(2));
        assert_eq!(syllable_count_for("zzzzqqq"), None);
    }

    #[test]
    fn syllable_filter_any_passes_everything() {
        assert!(syllable_count_matches_filter(None, None));
        assert!(syllable_count_matches_filter(Some(1), None));
        assert!(syllable_count_matches_filter(Some(5), None));
    }

    #[test]
    fn syllable_filter_exact_counts() {
        assert!(syllable_count_matches_filter(Some(2), Some(2)));
        assert!(!syllable_count_matches_filter(Some(3), Some(2)));
        assert!(!syllable_count_matches_filter(Some(1), Some(4)));
        // Unknown count never matches a specific filter.
        assert!(!syllable_count_matches_filter(None, Some(2)));
    }

    #[test]
    fn syllable_filter_five_plus_means_five_or_more() {
        assert!(!syllable_count_matches_filter(Some(4), Some(5)));
        assert!(syllable_count_matches_filter(Some(5), Some(5)));
        assert!(syllable_count_matches_filter(Some(6), Some(5)));
        assert!(syllable_count_matches_filter(Some(9), Some(5)));
    }

    #[test]
    fn syllable_filter_labels() {
        assert_eq!(syllable_filter_label(None), "Any");
        assert_eq!(syllable_filter_label(Some(1)), "1 syllable");
        assert_eq!(syllable_filter_label(Some(2)), "2 syllables");
        assert_eq!(syllable_filter_label(Some(4)), "4 syllables");
        assert_eq!(syllable_filter_label(Some(5)), "5+ syllables");
        assert_eq!(SYLLABLE_FILTER_OPTIONS.len(), 6);
    }

    #[test]
    fn rhyme_keys_group_rhyming_words() {
        assert_eq!(rhyme_key_for("day"), rhyme_key_for("say"));
        assert_ne!(rhyme_key_for("day"), rhyme_key_for("night"));
    }

    #[test]
    fn short_or_unknown_word_gives_no_rhymes() {
        assert!(find_rhymes("a", RhymeMode::Perfect).is_empty());
        assert!(find_rhymes("zzzzqqq", RhymeMode::Perfect).is_empty());
    }
}
