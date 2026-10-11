//! Porter stemmer, NLTK variant.
//!
//! A port of `nltk.stem.porter.PorterStemmer` in its default `NLTK_EXTENSIONS`
//! mode, which is the stemmer `rouge-score` uses. Input is expected to be a
//! lowercase ASCII word, which is what the ROUGE tokenizer produces.

type Condition<'a> = &'a dyn Fn(&[u8]) -> bool;

struct Rule<'a> {
    suffix: &'a [u8],
    replacement: &'a [u8],
    condition: Option<Condition<'a>>,
}

impl<'a> Rule<'a> {
    fn new(suffix: &'a str, replacement: &'a str, condition: Option<Condition<'a>>) -> Self {
        Rule {
            suffix: suffix.as_bytes(),
            replacement: replacement.as_bytes(),
            condition,
        }
    }
}

const DOUBLE_CONSONANT: &[u8] = b"*d";

/// Stems a lowercase ASCII word.
#[cfg(test)]
pub(crate) fn stem(word: String) -> String {
    let mut w = word.into_bytes();
    stem_in_place(&mut w);
    String::from_utf8(w).expect("stemming keeps ASCII input ASCII")
}

/// Stems a lowercase ASCII word held in `word`.
pub(crate) fn stem_in_place(word: &mut Vec<u8>) {
    if let Some(stem) = std::str::from_utf8(word).ok().and_then(irregular_form) {
        word.clear();
        word.extend_from_slice(stem.as_bytes());
        return;
    }
    if word.len() <= 2 {
        return;
    }
    step1a(word);
    step1b(word);
    step1c(word);
    step2(word);
    step3(word);
    step4(word);
    step5a(word);
    step5b(word);
}

fn irregular_form(word: &str) -> Option<&'static str> {
    Some(match word {
        "sky" | "skies" => "sky",
        "dying" => "die",
        "lying" => "lie",
        "tying" => "tie",
        "news" => "news",
        "innings" | "inning" => "inning",
        "outings" | "outing" => "outing",
        "cannings" | "canning" => "canning",
        "howe" => "howe",
        "proceed" => "proceed",
        "exceed" => "exceed",
        "succeed" => "succeed",
        _ => return None,
    })
}

fn is_vowel(c: u8) -> bool {
    matches!(c, b'a' | b'e' | b'i' | b'o' | b'u')
}

/// Consonant flag for every letter; `y` is a consonant at the start of a
/// word or after a vowel, and a vowel after a consonant.
fn consonant_flags(word: &[u8]) -> impl Iterator<Item = bool> + '_ {
    let mut previous = false;
    word.iter().enumerate().map(move |(i, &c)| {
        let flag = if is_vowel(c) {
            false
        } else if c == b'y' {
            i == 0 || !previous
        } else {
            true
        };
        previous = flag;
        flag
    })
}

fn is_consonant(word: &[u8], i: usize) -> bool {
    consonant_flags(&word[..=i]).last().unwrap_or(false)
}

/// The number of vowel-consonant sequences, `m` in Porter's notation.
fn measure(stem: &[u8]) -> usize {
    let mut count = 0;
    let mut previous_is_vowel = false;
    for is_cons in consonant_flags(stem) {
        if is_cons && previous_is_vowel {
            count += 1;
        }
        previous_is_vowel = !is_cons;
    }
    count
}

fn has_positive_measure(stem: &[u8]) -> bool {
    measure(stem) > 0
}

fn measure_gt_1(stem: &[u8]) -> bool {
    measure(stem) > 1
}

fn contains_vowel(stem: &[u8]) -> bool {
    consonant_flags(stem).any(|is_cons| !is_cons)
}

fn ends_double_consonant(word: &[u8]) -> bool {
    let n = word.len();
    n >= 2 && word[n - 1] == word[n - 2] && is_consonant(word, n - 1)
}

fn ends_cvc(word: &[u8]) -> bool {
    let n = word.len();
    (n >= 3
        && is_consonant(word, n - 3)
        && !is_consonant(word, n - 2)
        && is_consonant(word, n - 1)
        && !matches!(word[n - 1], b'w' | b'x' | b'y'))
        || (n == 2 && !is_consonant(word, 0) && is_consonant(word, 1))
}

fn replace_tail(word: &mut Vec<u8>, stem_len: usize, replacement: &[u8]) {
    word.truncate(stem_len);
    word.extend_from_slice(replacement);
}

/// Applies the first rule whose suffix matches. A matching rule whose
/// condition fails stops the search and leaves the word unchanged.
fn apply_rules(word: &mut Vec<u8>, rules: &[Rule]) {
    for rule in rules {
        let stem_len = if rule.suffix == DOUBLE_CONSONANT && ends_double_consonant(word) {
            word.len() - 2
        } else if word.ends_with(rule.suffix) {
            word.len() - rule.suffix.len()
        } else {
            continue;
        };
        if rule
            .condition
            .is_none_or(|condition| condition(&word[..stem_len]))
        {
            replace_tail(word, stem_len, rule.replacement);
        }
        return;
    }
}

fn step1a(word: &mut Vec<u8>) {
    if word.len() == 4 && word.ends_with(b"ies") {
        replace_tail(word, 1, b"ie");
        return;
    }
    apply_rules(
        word,
        &[
            Rule::new("sses", "ss", None),
            Rule::new("ies", "i", None),
            Rule::new("ss", "ss", None),
            Rule::new("s", "", None),
        ],
    );
}

fn step1b(word: &mut Vec<u8>) {
    if word.ends_with(b"ied") {
        let replacement: &[u8] = if word.len() == 4 { b"ie" } else { b"i" };
        replace_tail(word, word.len() - 3, replacement);
        return;
    }

    if word.ends_with(b"eed") {
        let stem_len = word.len() - 3;
        if measure(&word[..stem_len]) > 0 {
            replace_tail(word, stem_len, b"ee");
        }
        return;
    }

    let suffix_len = if word.ends_with(b"ed") {
        2
    } else if word.ends_with(b"ing") {
        3
    } else {
        return;
    };
    if !contains_vowel(&word[..word.len() - suffix_len]) {
        return;
    }
    word.truncate(word.len() - suffix_len);

    let last = [word[word.len() - 1]];
    let keep_double = |_: &[u8]| !matches!(last[0], b'l' | b's' | b'z');
    let short_syllable = |stem: &[u8]| measure(stem) == 1 && ends_cvc(stem);
    apply_rules(
        word,
        &[
            Rule::new("at", "ate", None),
            Rule::new("bl", "ble", None),
            Rule::new("iz", "ize", None),
            Rule {
                suffix: DOUBLE_CONSONANT,
                replacement: &last,
                condition: Some(&keep_double),
            },
            Rule::new("", "e", Some(&short_syllable)),
        ],
    );
}

fn step1c(word: &mut Vec<u8>) {
    let condition = |stem: &[u8]| stem.len() > 1 && is_consonant(stem, stem.len() - 1);
    apply_rules(word, &[Rule::new("y", "i", Some(&condition))]);
}

fn step2(word: &mut Vec<u8>) {
    if word.ends_with(b"alli") && has_positive_measure(&word[..word.len() - 4]) {
        replace_tail(word, word.len() - 4, b"al");
        step2(word);
        return;
    }

    let positive: Condition = &has_positive_measure;
    let logi_applies = word.len() >= 3 && has_positive_measure(&word[..word.len() - 3]);
    let logi = |_: &[u8]| logi_applies;
    apply_rules(
        word,
        &[
            Rule::new("ational", "ate", Some(positive)),
            Rule::new("tional", "tion", Some(positive)),
            Rule::new("enci", "ence", Some(positive)),
            Rule::new("anci", "ance", Some(positive)),
            Rule::new("izer", "ize", Some(positive)),
            Rule::new("bli", "ble", Some(positive)),
            Rule::new("alli", "al", Some(positive)),
            Rule::new("entli", "ent", Some(positive)),
            Rule::new("eli", "e", Some(positive)),
            Rule::new("ousli", "ous", Some(positive)),
            Rule::new("ization", "ize", Some(positive)),
            Rule::new("ation", "ate", Some(positive)),
            Rule::new("ator", "ate", Some(positive)),
            Rule::new("alism", "al", Some(positive)),
            Rule::new("iveness", "ive", Some(positive)),
            Rule::new("fulness", "ful", Some(positive)),
            Rule::new("ousness", "ous", Some(positive)),
            Rule::new("aliti", "al", Some(positive)),
            Rule::new("iviti", "ive", Some(positive)),
            Rule::new("biliti", "ble", Some(positive)),
            Rule::new("fulli", "ful", Some(positive)),
            Rule::new("logi", "log", Some(&logi)),
        ],
    );
}

fn step3(word: &mut Vec<u8>) {
    let positive: Condition = &has_positive_measure;
    apply_rules(
        word,
        &[
            Rule::new("icate", "ic", Some(positive)),
            Rule::new("ative", "", Some(positive)),
            Rule::new("alize", "al", Some(positive)),
            Rule::new("iciti", "ic", Some(positive)),
            Rule::new("ical", "ic", Some(positive)),
            Rule::new("ful", "", Some(positive)),
            Rule::new("ness", "", Some(positive)),
        ],
    );
}

fn step4(word: &mut Vec<u8>) {
    let gt1: Condition = &measure_gt_1;
    let ion = |stem: &[u8]| measure(stem) > 1 && matches!(stem.last(), Some(b's' | b't'));
    apply_rules(
        word,
        &[
            Rule::new("al", "", Some(gt1)),
            Rule::new("ance", "", Some(gt1)),
            Rule::new("ence", "", Some(gt1)),
            Rule::new("er", "", Some(gt1)),
            Rule::new("ic", "", Some(gt1)),
            Rule::new("able", "", Some(gt1)),
            Rule::new("ible", "", Some(gt1)),
            Rule::new("ant", "", Some(gt1)),
            Rule::new("ement", "", Some(gt1)),
            Rule::new("ment", "", Some(gt1)),
            Rule::new("ent", "", Some(gt1)),
            Rule::new("ion", "", Some(&ion)),
            Rule::new("ou", "", Some(gt1)),
            Rule::new("ism", "", Some(gt1)),
            Rule::new("ate", "", Some(gt1)),
            Rule::new("iti", "", Some(gt1)),
            Rule::new("ous", "", Some(gt1)),
            Rule::new("ive", "", Some(gt1)),
            Rule::new("ize", "", Some(gt1)),
        ],
    );
}

fn step5a(word: &mut Vec<u8>) {
    if word.ends_with(b"e") {
        let stem = &word[..word.len() - 1];
        let m = measure(stem);
        if m > 1 || (m == 1 && !ends_cvc(stem)) {
            word.pop();
        }
    }
}

fn step5b(word: &mut Vec<u8>) {
    let applies = !word.is_empty() && measure(&word[..word.len() - 1]) > 1;
    let condition = |_: &[u8]| applies;
    apply_rules(word, &[Rule::new("ll", "l", Some(&condition))]);
}

#[cfg(test)]
mod tests {
    fn stem(word: &str) -> String {
        super::stem(word.to_owned())
    }

    #[test]
    fn stems_classic_examples() {
        let cases = [
            ("caresses", "caress"),
            ("ponies", "poni"),
            ("ties", "tie"),
            ("cats", "cat"),
            ("feed", "feed"),
            ("agreed", "agre"),
            ("plastered", "plaster"),
            ("motoring", "motor"),
            ("conflated", "conflat"),
            ("hopping", "hop"),
            ("falling", "fall"),
            ("filing", "file"),
            ("happy", "happi"),
            ("relational", "relat"),
            ("conditional", "condit"),
            ("generalization", "gener"),
            ("electrical", "electr"),
            ("adjustment", "adjust"),
            ("controlling", "control"),
            ("generously", "gener"),
        ];
        for (word, expected) in cases {
            assert_eq!(stem(word), expected, "stem({word:?})");
        }
    }

    #[test]
    fn uses_nltk_irregular_forms() {
        assert_eq!(stem("dying"), "die");
        assert_eq!(stem("skies"), "sky");
        assert_eq!(stem("innings"), "inning");
        assert_eq!(stem("succeed"), "succeed");
    }

    #[test]
    fn leaves_short_words_alone() {
        assert_eq!(stem("as"), "as");
        assert_eq!(stem("is"), "is");
    }
}
