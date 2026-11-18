//! Post-transcription formatting (TASK-VTT208): writes spoken sterling amounts
//! as figures ("ten pounds" -> "£10") and puts the accents back on a short
//! list of loanwords ("touche" -> "touché"). Parakeet and Whisper write both
//! as plain words most of the time, so without this step every dictated price
//! or loanword needs a hand rewrite.

use regex::{Captures, Regex};
use std::sync::OnceLock;

/// Formats a finished transcript. `money_figures` turns spoken amounts of
/// pounds into £ figures; `accents` restores accents on the built-in loanword
/// list. Each step is skipped when its flag is false.
pub fn apply(text: &str, money_figures: bool, accents: bool) -> String {
    let mut out = text.to_string();
    if money_figures {
        out = write_money(&out);
    }
    if accents {
        out = restore_accents(&out);
    }
    out
}

const UNITS: &str = "one|two|three|four|five|six|seven|eight|nine";
const TEENS: &str =
    "ten|eleven|twelve|thirteen|fourteen|fifteen|sixteen|seventeen|eighteen|nineteen";
const TENS: &str = "twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety";
const SCALES: &str = "hundred|thousand|million";

/// An amount (digits, or number words) followed by "pound" or "pounds", then
/// either "of" (a weight, left alone) or an optional pence part.
fn money_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let word = format!(r"(?:{TEENS}|{TENS}|{UNITS}|{SCALES})\b");
        let words = format!(r"(?:\ba\s+(?:{SCALES})\b|\b{word})(?:(?:\s+and\s+|[\s-]+){word})*");
        let digits = r"\b(?:\d{1,3}(?:,\d{3})+|\d+)(?:\.\d{2})?\b";
        let pence = format!(r"\d{{1,2}}\b|(?:{TENS})\b(?:[\s-]+(?:{UNITS})\b)?|(?:{TEENS}|{UNITS})\b");
        Regex::new(&format!(
            r"(?i)(?P<sign>£)?(?P<num>{digits}|{words})\s+pounds?\b(?:(?P<of>\s+of\b)|(?P<tail>\s+(?:and\s+)?(?P<pence>{pence})(?:\s+(?P<unit>pence|p)\b)?))?"
        ))
        .expect("money pattern compiles")
    })
}

fn write_money(text: &str) -> String {
    money_pattern()
        .replace_all(text, |c: &Captures| {
            let whole = &c[0];
            if c.name("of").is_some() {
                return whole.to_string();
            }
            let num = &c["num"];
            let pounds = if num.starts_with(|ch: char| ch.is_ascii_digit()) {
                num.to_string()
            } else {
                match words_to_number(num) {
                    Some(n) => group_thousands(n),
                    None => return whole.to_string(),
                }
            };
            let tail = c.name("tail").map_or("", |m| m.as_str());
            let pence = c
                .name("pence")
                .and_then(|p| spoken_pence(p.as_str(), c.name("unit").is_some()))
                .filter(|_| !pounds.contains('.'));
            match pence {
                Some(p) => format!("£{pounds}.{p:02}"),
                None => format!("£{pounds}{tail}"),
            }
        })
        .into_owned()
}

/// Pence after the pounds, 1 to 99. Without a "pence" or "p" after it only
/// 10 to 99 counts, so "ten pounds two times" keeps its "two".
fn spoken_pence(text: &str, has_unit: bool) -> Option<u64> {
    let n = if text.starts_with(|ch: char| ch.is_ascii_digit()) {
        text.parse().ok()?
    } else {
        words_to_number(text)?
    };
    let floor = if has_unit { 1 } else { 10 };
    (floor..=99).contains(&n).then_some(n)
}

/// Parses a run of English number words ("two hundred and forty") into its
/// value. A run that is not a well-formed number ("five ten") returns None,
/// so the text is left as spoken rather than guessed at.
fn words_to_number(text: &str) -> Option<u64> {
    #[derive(PartialEq, Clone, Copy)]
    enum Last {
        Start,
        Unit,
        Teen,
        Tens,
        Hundred,
        Scale,
    }
    let mut last = Last::Start;
    let mut total: u64 = 0;
    let mut group: u64 = 0;
    let mut last_scale = u64::MAX;
    let lower = text.to_lowercase();
    let tokens = lower
        .split(|ch: char| ch.is_whitespace() || ch == '-')
        .filter(|t| !t.is_empty());
    for token in tokens {
        if token == "and" {
            if !matches!(last, Last::Hundred | Last::Scale) {
                return None;
            }
            continue;
        }
        if token == "a" {
            if last != Last::Start {
                return None;
            }
            group = 1;
            last = Last::Unit;
            continue;
        }
        if let Some(v) = position(UNITS, token) {
            if !matches!(last, Last::Start | Last::Tens | Last::Hundred | Last::Scale) {
                return None;
            }
            group += v + 1;
            last = Last::Unit;
        } else if let Some(v) = position(TEENS, token) {
            if !matches!(last, Last::Start | Last::Hundred | Last::Scale) {
                return None;
            }
            group += v + 10;
            last = Last::Teen;
        } else if let Some(v) = position(TENS, token) {
            if !matches!(last, Last::Start | Last::Hundred | Last::Scale) {
                return None;
            }
            group += (v + 2) * 10;
            last = Last::Tens;
        } else if token == "hundred" {
            if group >= 100 || last == Last::Hundred {
                return None;
            }
            group = group.max(1) * 100;
            last = Last::Hundred;
        } else {
            let scale = match token {
                "thousand" => 1_000,
                "million" => 1_000_000,
                _ => return None,
            };
            if scale >= last_scale || last == Last::Scale {
                return None;
            }
            total += group.max(1) * scale;
            group = 0;
            last_scale = scale;
            last = Last::Scale;
        }
    }
    (last != Last::Start).then_some(total + group)
}

/// Index of `token` in a `|`-separated word list.
fn position(list: &str, token: &str) -> Option<u64> {
    list.split('|').position(|w| w == token).map(|i| i as u64)
}

/// 1500 -> "1,500".
fn group_thousands(n: u64) -> String {
    let reversed: Vec<char> = n.to_string().chars().rev().collect();
    let groups: Vec<String> = reversed
        .chunks(3)
        .rev()
        .map(|g| g.iter().rev().collect())
        .collect();
    groups.join(",")
}

/// Loanwords whose unaccented spelling is not also an English word, so the
/// accent can go back on without guessing. "resume" is left out: it is a
/// plain English verb. Each pair has the same number of characters, which is
/// what lets the case of the spoken word be copied across letter by letter.
const LOANWORDS: [(&str, &str); 7] = [
    ("touche", "touché"),
    ("cafe", "café"),
    ("nee", "née"),
    ("fiancee", "fiancée"),
    ("fiance", "fiancé"),
    ("cliche", "cliché"),
    ("deja vu", "déjà vu"),
];

fn loanword_pattern() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        let words: Vec<_> = LOANWORDS.iter().map(|(plain, _)| *plain).collect();
        Regex::new(&format!(r"(?i)\b(?:{})\b", words.join("|"))).expect("loanword pattern compiles")
    })
}

fn restore_accents(text: &str) -> String {
    loanword_pattern()
        .replace_all(text, |c: &Captures| {
            let spoken = &c[0];
            let lower = spoken.to_lowercase();
            let Some((_, accented)) = LOANWORDS.iter().find(|(plain, _)| *plain == lower) else {
                return spoken.to_string();
            };
            spoken
                .chars()
                .zip(accented.chars())
                .map(|(s, a)| {
                    if s.is_uppercase() {
                        a.to_uppercase().collect::<String>()
                    } else {
                        a.to_string()
                    }
                })
                .collect()
        })
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn money(text: &str) -> String {
        apply(text, true, false)
    }

    fn accent(text: &str) -> String {
        apply(text, false, true)
    }

    #[test]
    fn spoken_whole_pounds_become_figures() {
        assert_eq!(money("ten pounds"), "£10");
        assert_eq!(money("two hundred and forty pounds"), "£240");
        assert_eq!(money("two hundred and fifty pounds"), "£250");
        assert_eq!(money("six hundred and fifty pounds"), "£650");
        assert_eq!(money("one pound"), "£1");
    }

    #[test]
    fn one_pound_fifty_becomes_pounds_and_pence() {
        assert_eq!(money("one pound fifty"), "£1.50");
    }

    #[test]
    fn the_dictated_list_from_the_card_types_as_figures() {
        assert_eq!(
            money("Two hundred and fifty pounds, six hundred and fifty pounds, one pound, one pound fifty"),
            "£250, £650, £1, £1.50"
        );
    }

    #[test]
    fn the_dictated_sentence_from_the_card_types_as_figures() {
        assert_eq!(
            money("Total cost ten pounds. Am I giving two hundred and forty pounds?"),
            "Total cost £10. Am I giving £240?"
        );
    }

    #[test]
    fn digits_followed_by_pounds_take_the_pound_sign() {
        assert_eq!(money("250 pounds"), "£250");
        assert_eq!(money("it was 1,500 pounds"), "it was £1,500");
        assert_eq!(money("2.50 pounds"), "£2.50");
    }

    #[test]
    fn an_amount_already_written_with_the_sign_is_left_alone() {
        assert_eq!(money("£250"), "£250");
        assert_eq!(money("Total cost £10."), "Total cost £10.");
        assert_eq!(
            money("£250 pounds"),
            "£250",
            "a doubled sign and word must not become ££250"
        );
    }

    #[test]
    fn a_weight_is_left_alone() {
        assert_eq!(money("two pounds of flour"), "two pounds of flour");
        assert_eq!(money("2 pounds of flour"), "2 pounds of flour");
    }

    #[test]
    fn pound_as_a_verb_is_left_alone() {
        assert_eq!(money("pound the dough"), "pound the dough");
        assert_eq!(money("Pound the dough well"), "Pound the dough well");
    }

    #[test]
    fn pounds_with_no_number_before_it_is_left_alone() {
        assert_eq!(money("pounds"), "pounds");
        assert_eq!(
            money("the pounds and euros fell"),
            "the pounds and euros fell"
        );
    }

    #[test]
    fn a_number_word_inside_a_longer_word_is_not_an_amount() {
        assert_eq!(money("someone pounds"), "someone pounds");
    }

    #[test]
    fn a_capitalised_amount_at_the_start_of_a_sentence_converts() {
        assert_eq!(money("Ten pounds."), "£10.");
    }

    #[test]
    fn spoken_pence_join_the_figure() {
        assert_eq!(money("five pounds and twenty pence"), "£5.20");
        assert_eq!(money("three pounds five p"), "£3.05");
        assert_eq!(money("ten pounds ninety-nine"), "£10.99");
    }

    #[test]
    fn a_small_number_after_pounds_without_pence_is_not_taken_as_pence() {
        assert_eq!(money("ten pounds two times"), "£10 two times");
    }

    #[test]
    fn larger_amounts_take_a_thousands_separator() {
        assert_eq!(money("one thousand five hundred pounds"), "£1,500");
        assert_eq!(money("a hundred pounds"), "£100");
        assert_eq!(money("a thousand pounds"), "£1,000");
        assert_eq!(money("twenty-five pounds"), "£25");
        assert_eq!(money("two million pounds"), "£2,000,000");
    }

    #[test]
    fn a_jumbled_run_of_number_words_is_left_alone() {
        assert_eq!(money("five ten pounds"), "five ten pounds");
    }

    #[test]
    fn money_step_off_leaves_amounts_as_words() {
        assert_eq!(apply("ten pounds", false, true), "ten pounds");
    }

    #[test]
    fn loanwords_get_their_accents_back() {
        assert_eq!(accent("touche"), "touché");
        assert_eq!(accent("a cafe"), "a café");
        assert_eq!(accent("Jane Smith nee Brown"), "Jane Smith née Brown");
        assert_eq!(accent("my fiance"), "my fiancé");
        assert_eq!(accent("my fiancee"), "my fiancée");
        assert_eq!(accent("a cliche"), "a cliché");
        assert_eq!(accent("deja vu"), "déjà vu");
    }

    #[test]
    fn accent_restoration_keeps_case() {
        assert_eq!(accent("Touche"), "Touché");
        assert_eq!(accent("TOUCHE"), "TOUCHÉ");
        assert_eq!(accent("Deja vu again"), "Déjà vu again");
        assert_eq!(accent("Cafe Nero"), "Café Nero");
    }

    #[test]
    fn resume_is_left_alone_because_it_is_also_plain_english() {
        assert_eq!(accent("resume the call"), "resume the call");
    }

    #[test]
    fn a_word_that_merely_contains_a_loanword_is_left_alone() {
        assert_eq!(accent("the cafeteria"), "the cafeteria");
        assert_eq!(accent("kneel down"), "kneel down");
        assert_eq!(accent("she touches it"), "she touches it");
    }

    #[test]
    fn an_already_accented_word_is_left_alone() {
        assert_eq!(accent("Touché"), "Touché");
        assert_eq!(accent("café"), "café");
    }

    #[test]
    fn accent_step_off_leaves_words_unaccented() {
        assert_eq!(apply("Touche", true, false), "Touche");
    }

    #[test]
    fn both_steps_run_together() {
        assert_eq!(
            apply("Touche, the cafe charged ten pounds", true, true),
            "Touché, the café charged £10"
        );
    }

    #[test]
    fn text_with_neither_is_unchanged() {
        assert_eq!(apply("hello world", true, true), "hello world");
        assert_eq!(apply("", true, true), "");
    }
}
