use icu_casemap::CaseMapper;
use icu_properties::props::{Alphabetic, GeneralCategory, GeneralCategoryGroup};
use icu_properties::{CodePointMapData, CodePointSetData};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::porter;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    RougeScore { use_stemmer: bool },
    Unicode,
    Character,
}

/// Splits text into tokens for scoring.
///
/// Three profiles are available:
///
/// - [`Tokenizer::new`] is the tokenizer of `rouge-score` and the default. Text is
///   lowercased, every character outside `[a-z0-9]` acts as a separator, and, with
///   stemming enabled, tokens longer than three characters go through the Porter
///   stemmer. Text in other scripts produces no tokens.
/// - [`Tokenizer::unicode`] (`unicode-v1`) normalizes the text (NFKC, full case
///   folding, NFKC again) and keeps the words found by the Unicode word boundary
///   rules (UAX #29) that contain an alphabetic character or a number. It suits
///   languages that separate words with spaces, in any script.
/// - [`Tokenizer::character`] (`character-v1`) normalizes the text the same way
///   and makes every grapheme cluster that contains an alphabetic character or a
///   number a token. Character-level ROUGE is a common convention for Chinese.
///
/// Both Unicode profiles never stem and use Unicode 17.0 data from pinned
/// dependencies, so their output does not depend on the Rust version.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tokenizer {
    kind: Kind,
}

impl Default for Tokenizer {
    fn default() -> Self {
        Tokenizer::new(false)
    }
}

impl Tokenizer {
    /// The `rouge-score` tokenizer, optionally with the Porter stemmer.
    pub fn new(use_stemmer: bool) -> Self {
        Tokenizer {
            kind: Kind::RougeScore { use_stemmer },
        }
    }

    /// The `unicode-v1` profile: Unicode word boundaries.
    pub fn unicode() -> Self {
        Tokenizer {
            kind: Kind::Unicode,
        }
    }

    /// The `character-v1` profile: one token per grapheme cluster.
    pub fn character() -> Self {
        Tokenizer {
            kind: Kind::Character,
        }
    }

    pub fn use_stemmer(&self) -> bool {
        matches!(self.kind, Kind::RougeScore { use_stemmer: true })
    }

    /// The profile name: `rouge-score`, `unicode-v1` or `character-v1`.
    pub fn profile(&self) -> &'static str {
        match self.kind {
            Kind::RougeScore { .. } => "rouge-score",
            Kind::Unicode => "unicode-v1",
            Kind::Character => "character-v1",
        }
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Tokens::default();
        self.tokenize_into(text, &mut tokens);
        tokens.iter().map(str::to_owned).collect()
    }

    /// Appends the tokens of `text` to `tokens`.
    pub(crate) fn tokenize_into(&self, text: &str, tokens: &mut Tokens) {
        match self.kind {
            Kind::RougeScore { use_stemmer } => rouge_score_tokens(text, use_stemmer, tokens),
            Kind::Unicode => {
                for word in normalize(text).unicode_words() {
                    tokens.push(word);
                }
            }
            Kind::Character => {
                for grapheme in normalize(text).graphemes(true) {
                    if grapheme.chars().any(is_alphanumeric) {
                        tokens.push(grapheme);
                    }
                }
            }
        }
    }
}

/// Tokens stored back to back in one string, so that tokenizing allocates
/// once per text rather than once per token.
#[derive(Clone, Debug, Default)]
pub(crate) struct Tokens {
    text: String,
    ends: Vec<usize>,
}

impl Tokens {
    pub(crate) fn len(&self) -> usize {
        self.ends.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &str> + '_ {
        let starts = std::iter::once(0).chain(self.ends.iter().copied());
        starts
            .zip(&self.ends)
            .map(|(start, &end)| &self.text[start..end])
    }

    fn push(&mut self, token: &str) {
        self.text.push_str(token);
        self.ends.push(self.text.len());
    }

    /// Ends the token formed by the text written since the previous token.
    fn close(&mut self) {
        if self.text.len() > self.ends.last().copied().unwrap_or(0) {
            self.ends.push(self.text.len());
        }
    }

    fn current(&self) -> &str {
        &self.text[self.ends.last().copied().unwrap_or(0)..]
    }

    fn replace_current(&mut self, token: &[u8]) {
        let start = self.ends.last().copied().unwrap_or(0);
        self.text.truncate(start);
        self.text
            .push_str(std::str::from_utf8(token).expect("stems are ASCII"));
    }
}

/// NFKC, then full case folding, then NFKC again: case folding can undo the
/// normalization (𝐀 becomes A under NFKC and only then folds to a).
fn normalize(text: &str) -> String {
    let compatible: String = text.nfkc().collect();
    CaseMapper::new().fold_string(&compatible).nfkc().collect()
}

/// Alphabetic or a number, by the Unicode 17.0 data of `icu_properties`; the same
/// rule `unicode_words` applies with its own Unicode 17.0 tables.
fn is_alphanumeric(ch: char) -> bool {
    CodePointSetData::new::<Alphabetic>().contains(ch)
        || GeneralCategoryGroup::Number.contains(CodePointMapData::<GeneralCategory>::new().get(ch))
}

fn rouge_score_tokens(text: &str, use_stemmer: bool, tokens: &mut Tokens) {
    let mut stem_buffer = Vec::new();
    let mut push = |ch: char, tokens: &mut Tokens| {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            tokens.text.push(ch);
        } else {
            finish_token(tokens, use_stemmer, &mut stem_buffer);
        }
    };
    for ch in text.chars() {
        if ch.is_ascii() {
            push(ch.to_ascii_lowercase(), tokens);
        } else {
            // Some non-ASCII characters lowercase to ASCII letters, e.g.
            // KELVIN SIGN becomes "k" and "İ" becomes "i" plus a combining dot.
            for lower in ch.to_lowercase() {
                push(lower, tokens);
            }
        }
    }
    finish_token(tokens, use_stemmer, &mut stem_buffer);
}

fn finish_token(tokens: &mut Tokens, use_stemmer: bool, stem_buffer: &mut Vec<u8>) {
    if use_stemmer && tokens.current().len() > 3 {
        stem_buffer.clear();
        stem_buffer.extend_from_slice(tokens.current().as_bytes());
        porter::stem_in_place(stem_buffer);
        tokens.replace_current(stem_buffer);
    }
    tokens.close();
}

#[cfg(test)]
mod tests {
    use super::Tokenizer;

    #[test]
    fn splits_on_non_alphanumeric_characters() {
        let tokens = Tokenizer::new(false).tokenize("Hello, World! It's 2024-05-01.");
        assert_eq!(tokens, ["hello", "world", "it", "s", "2024", "05", "01"]);
    }

    #[test]
    fn drops_non_ascii_letters() {
        let tokens = Tokenizer::new(false).tokenize("café naïve Привет");
        assert_eq!(tokens, ["caf", "na", "ve"]);
    }

    #[test]
    fn follows_unicode_lowercasing() {
        let tokens = Tokenizer::new(false).tokenize("\u{212A}m İstanbul");
        assert_eq!(tokens, ["km", "i", "stanbul"]);
    }

    #[test]
    fn stems_only_long_tokens() {
        let tokens = Tokenizer::new(true).tokenize("The cats were running fast");
        assert_eq!(tokens, ["the", "cat", "were", "run", "fast"]);
    }

    #[test]
    fn unicode_profile_keeps_words_in_any_script() {
        let unicode = Tokenizer::unicode();
        assert_eq!(unicode.tokenize("Привет, мир!"), ["привет", "мир"]);
        assert_eq!(unicode.tokenize("مرحبا بالعالم"), ["مرحبا", "بالعالم"]);
        assert_eq!(unicode.tokenize("नमस्ते दुनिया"), ["नमस्ते", "दुनिया"]);
        assert_eq!(unicode.tokenize("Ελληνικά ΟΔΟΣ"), ["ελληνικά", "οδοσ"]);
    }

    #[test]
    fn unicode_profile_keeps_word_internal_punctuation() {
        let tokens = Tokenizer::unicode().tokenize("Don't pay $3.50 for e-mail, U.S. users.");
        assert_eq!(
            tokens,
            ["don't", "pay", "3.50", "for", "e", "mail", "u.s", "users"]
        );
    }

    #[test]
    fn unicode_profile_normalizes_equivalent_text() {
        let unicode = Tokenizer::unicode();
        assert_eq!(unicode.tokenize("Cafe\u{301}"), unicode.tokenize("Café"));
        assert_eq!(unicode.tokenize("ＧＰＵ ２０２４"), ["gpu", "2024"]);
        assert_eq!(unicode.tokenize("𝐀 ㎒ Straße"), ["a", "mhz", "strasse"]);
    }

    #[test]
    fn normalization_is_idempotent() {
        for text in [
            "𝐀𝐁 ㎒",
            "ﬁﬂ Straße ΣΑΣ",
            "İstanbul",
            "ǅemal",
            "\u{16EA0}\u{323B0}",
        ] {
            let once = super::normalize(text);
            assert_eq!(super::normalize(&once), once, "{text:?}");
        }
    }

    #[test]
    fn uses_unicode_17_independently_of_the_compiler() {
        // U+16EA0 BERIA ERFE CAPITAL LETTER ARKAB was added in Unicode 17.0.
        assert_eq!(Tokenizer::unicode().tokenize("\u{16EA0}"), ["\u{16EBB}"]);
        assert_eq!(Tokenizer::character().tokenize("\u{16EA0}"), ["\u{16EBB}"]);
    }

    #[test]
    fn unicode_profile_splits_ideographs() {
        assert_eq!(
            Tokenizer::unicode().tokenize("我爱北京"),
            ["我", "爱", "北", "京"]
        );
    }

    #[test]
    fn character_profile_uses_grapheme_clusters() {
        let character = Tokenizer::character();
        assert_eq!(character.tokenize("我爱北京。"), ["我", "爱", "北", "京"]);
        // Since Unicode 15.1 a conjunct such as "स्ते" is one grapheme cluster.
        assert_eq!(character.tokenize("नमस्ते"), ["न", "म", "स्ते"]);
        assert_eq!(character.tokenize("A1 b"), ["a", "1", "b"]);
    }

    #[test]
    fn reports_profiles() {
        assert_eq!(Tokenizer::new(true).profile(), "rouge-score");
        assert_eq!(Tokenizer::unicode().profile(), "unicode-v1");
        assert_eq!(Tokenizer::character().profile(), "character-v1");
        assert!(!Tokenizer::unicode().use_stemmer());
    }
}
