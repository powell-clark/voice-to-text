//! Post-transcription correction dictionary (FEAT-VTT037): deterministic
//! whole-word/phrase substitutions applied after Whisper output, for words
//! and phrases the model reliably mishears regardless of `initial_prompt`
//! (which only biases inference — it doesn't guarantee a fix).

use regex::{NoExpand, RegexBuilder};

/// Applies each `(misheard, correct)` pair in order as a case-insensitive,
/// whole-word/phrase replacement. The match is case-insensitive; the
/// replacement is written exactly as configured, verbatim (no regex
/// expansion of `$` in the replacement text). An empty list is a no-op.
pub fn apply(text: &str, corrections: &[(String, String)]) -> String {
    let mut result = text.to_string();
    for (from, to) in corrections {
        if from.trim().is_empty() {
            continue;
        }
        let pattern = format!(r"\b{}\b", regex::escape(from.trim()));
        let Ok(re) = RegexBuilder::new(&pattern).case_insensitive(true).build() else {
            continue;
        };
        result = re.replace_all(&result, NoExpand(to)).into_owned();
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corrects_a_whole_word() {
        assert_eq!(
            apply("I said ard", &[("ard".into(), "odd".into())]),
            "I said odd"
        );
    }

    #[test]
    fn match_is_case_insensitive_replacement_uses_configured_case() {
        assert_eq!(
            apply("I said ARD", &[("ard".into(), "odd".into())]),
            "I said odd"
        );
    }

    #[test]
    fn does_not_corrupt_a_larger_word_containing_the_pattern() {
        assert_eq!(
            apply("that's hard work", &[("ard".into(), "odd".into())]),
            "that's hard work"
        );
    }

    #[test]
    fn corrects_a_multi_word_phrase() {
        assert_eq!(
            apply(
                "talking about amala vajrayana today",
                &[("amala vajrayana".into(), "Amala Vijnana".into())]
            ),
            "talking about Amala Vijnana today"
        );
    }

    #[test]
    fn empty_corrections_list_is_a_noop() {
        assert_eq!(apply("hello world", &[]), "hello world");
    }

    #[test]
    fn dollar_sign_in_replacement_is_written_literally() {
        assert_eq!(
            apply("I said moneys", &[("moneys".into(), "$100".into())]),
            "I said $100"
        );
    }

    #[test]
    fn corrections_apply_in_list_order() {
        let pairs = [
            ("ard".to_string(), "odd".to_string()),
            ("odd".to_string(), "even".to_string()),
        ];
        assert_eq!(apply("I said ard", &pairs), "I said even");
    }

    #[test]
    fn blank_misheard_entry_is_skipped_not_a_pattern_that_matches_everything() {
        assert_eq!(
            apply("hello world", &[(" ".into(), "x".into())]),
            "hello world"
        );
    }
}
