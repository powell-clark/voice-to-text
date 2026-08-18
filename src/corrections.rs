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

/// Renders the correction list as the one-pair-per-line text the settings
/// dialog edits: `misheard => correct`. The inverse of [`parse_pairs`].
pub fn format_pairs(corrections: &[(String, String)]) -> String {
    corrections
        .iter()
        .map(|(from, to)| format!("{from} => {to}"))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Parses the settings dialog's textarea back into correction pairs, one
/// `misheard => correct` per line. Blank lines are skipped; a line with no
/// `=>` separator, an empty left-hand side, or an empty right-hand side is
/// dropped rather than saved, so a half-typed row can never silently
/// become a rule that eats a word. Order is preserved — corrections apply
/// in list order and the user controls that order by line order.
pub fn parse_pairs(text: &str) -> Vec<(String, String)> {
    text.lines()
        .filter_map(|line| {
            let (from, to) = line.split_once("=>")?;
            let (from, to) = (from.trim(), to.trim());
            if from.is_empty() || to.is_empty() {
                return None;
            }
            Some((from.to_string(), to.to_string()))
        })
        .collect()
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

    #[test]
    fn formats_one_pair_per_line() {
        assert_eq!(
            format_pairs(&[
                ("ard".into(), "odd".into()),
                ("amala vajrayana".into(), "Amala Vijnana".into()),
            ]),
            "ard => odd\namala vajrayana => Amala Vijnana"
        );
    }

    #[test]
    fn empty_list_formats_to_empty_text() {
        assert_eq!(format_pairs(&[]), "");
    }

    #[test]
    fn parses_one_pair_per_line_trimming_whitespace() {
        assert_eq!(
            parse_pairs("  ard  =>  odd  \namala vajrayana=>Amala Vijnana"),
            vec![
                ("ard".to_string(), "odd".to_string()),
                ("amala vajrayana".to_string(), "Amala Vijnana".to_string()),
            ]
        );
    }

    #[test]
    fn parse_skips_blank_lines_and_lines_without_a_separator() {
        assert_eq!(
            parse_pairs("ard => odd\n\nno separator here\n   \nfoo => bar"),
            vec![
                ("ard".to_string(), "odd".to_string()),
                ("foo".to_string(), "bar".to_string()),
            ]
        );
    }

    #[test]
    fn parse_drops_a_half_typed_row_rather_than_saving_an_empty_side() {
        assert!(parse_pairs("=> odd").is_empty());
        assert!(parse_pairs("ard =>").is_empty());
        assert!(parse_pairs("   =>   ").is_empty());
    }

    #[test]
    fn format_then_parse_round_trips() {
        let original = vec![
            ("ard".to_string(), "odd".to_string()),
            ("amala vajrayana".to_string(), "Amala Vijnana".to_string()),
        ];
        assert_eq!(parse_pairs(&format_pairs(&original)), original);
    }

    #[test]
    fn parse_keeps_only_the_first_separator_so_the_replacement_may_contain_one() {
        assert_eq!(
            parse_pairs("arrow => a => b"),
            vec![("arrow".to_string(), "a => b".to_string())]
        );
    }
}
