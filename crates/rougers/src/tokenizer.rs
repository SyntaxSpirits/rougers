use crate::porter;

/// The tokenizer used by `rouge-score`.
///
/// Text is lowercased, every character outside `[a-z0-9]` acts as a
/// separator, and, with stemming enabled, tokens longer than three characters
/// go through the Porter stemmer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Tokenizer {
    use_stemmer: bool,
}

impl Tokenizer {
    pub fn new(use_stemmer: bool) -> Self {
        Tokenizer { use_stemmer }
    }

    pub fn use_stemmer(&self) -> bool {
        self.use_stemmer
    }

    pub fn tokenize(&self, text: &str) -> Vec<String> {
        let mut tokens = Vec::new();
        let mut current = String::new();
        for ch in text.chars() {
            if ch.is_ascii() {
                self.push_char(ch.to_ascii_lowercase(), &mut current, &mut tokens);
            } else {
                // Some non-ASCII characters lowercase to ASCII letters, e.g.
                // KELVIN SIGN becomes "k" and "İ" becomes "i" plus a combining dot.
                for lower in ch.to_lowercase() {
                    self.push_char(lower, &mut current, &mut tokens);
                }
            }
        }
        self.finish_token(&mut current, &mut tokens);
        tokens
    }

    fn push_char(&self, ch: char, current: &mut String, tokens: &mut Vec<String>) {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() {
            current.push(ch);
        } else {
            self.finish_token(current, tokens);
        }
    }

    fn finish_token(&self, current: &mut String, tokens: &mut Vec<String>) {
        if current.is_empty() {
            return;
        }
        let token = std::mem::take(current);
        if self.use_stemmer && token.len() > 3 {
            let stemmed = porter::stem(token);
            if !stemmed.is_empty() {
                tokens.push(stemmed);
            }
        } else {
            tokens.push(token);
        }
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
}
