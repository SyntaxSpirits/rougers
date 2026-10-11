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
/// - [`Tokenizer::unicode`] (`unicode-v1`) lowercases, applies NFKC normalization
///   and keeps the words found by the Unicode word boundary rules (UAX #29) that
///   contain a letter or a number. It suits languages that separate words with
///   spaces, in any script.
/// - [`Tokenizer::character`] (`character-v1`) lowercases, applies NFKC
///   normalization and makes every grapheme cluster that contains a letter or a
///   number a token. Character-level ROUGE is a common convention for Chinese.
///
/// Both Unicode profiles use Unicode 17.0 data and never stem.
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
        match self.kind {
            Kind::RougeScore { use_stemmer } => rouge_score_tokens(text, use_stemmer),
            Kind::Unicode => normalize(text).unicode_words().map(str::to_owned).collect(),
            Kind::Character => normalize(text)
                .graphemes(true)
                .filter(|grapheme| grapheme.chars().any(char::is_alphanumeric))
                .map(str::to_owned)
                .collect(),
        }
    }
}

fn normalize(text: &str) -> String {
    text.to_lowercase().nfkc().collect()
}

fn rouge_score_tokens(text: &str, use_stemmer: bool) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut push = |ch: char, current: &mut String| {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            current.push(ch);
        } else {
            finish_token(current, &mut tokens, use_stemmer);
        }
    };
    for ch in text.chars() {
        if ch.is_ascii() {
            push(ch.to_ascii_lowercase(), &mut current);
        } else {
            // Some non-ASCII characters lowercase to ASCII letters, e.g.
            // KELVIN SIGN becomes "k" and "İ" becomes "i" plus a combining dot.
            for lower in ch.to_lowercase() {
                push(lower, &mut current);
            }
        }
    }
    finish_token(&mut current, &mut tokens, use_stemmer);
    tokens
}

fn finish_token(current: &mut String, tokens: &mut Vec<String>, use_stemmer: bool) {
    if current.is_empty() {
        return;
    }
    let token = std::mem::take(current);
    if use_stemmer && token.len() > 3 {
        let stemmed = porter::stem(token);
        if !stemmed.is_empty() {
            tokens.push(stemmed);
        }
    } else {
        tokens.push(token);
    }
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
        assert_eq!(unicode.tokenize("Ελληνικά ΟΔΟΣ"), ["ελληνικά", "οδος"]);
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
