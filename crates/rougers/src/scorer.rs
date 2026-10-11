use std::ops::Range;

use rustc_hash::FxHashMap;

use crate::tokenizer::Tokens;
use crate::{metrics, Error, RougeType, Score, Tokenizer};

/// Tokenized text, ready for scoring.
///
/// `tokens` feed `rougeN` and `rougeL`; `sentences` feed `rougeLsum`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Document {
    pub tokens: Vec<String>,
    pub sentences: Vec<Vec<String>>,
}

/// Scores for each requested ROUGE variant, in request order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Scores(Vec<(RougeType, Score)>);

impl Scores {
    pub fn get(&self, rouge_type: RougeType) -> Option<Score> {
        self.0
            .iter()
            .find(|(t, _)| *t == rouge_type)
            .map(|&(_, score)| score)
    }

    pub fn iter(&self) -> impl Iterator<Item = (RougeType, Score)> + '_ {
        self.0.iter().copied()
    }
}

impl IntoIterator for Scores {
    type Item = (RougeType, Score);
    type IntoIter = std::vec::IntoIter<(RougeType, Score)>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Computes ROUGE scores between a target (reference) and a prediction.
///
/// ```
/// use rougers::{RougeScorer, RougeType};
///
/// let scorer = RougeScorer::new(["rouge1", "rougeL"], true)?;
/// let scores = scorer.score(
///     "The quick brown fox jumps over the lazy dog",
///     "The quick brown dog jumps on the log.",
/// );
/// let rouge1 = scores.get(RougeType::N(1)).unwrap();
/// assert!((rouge1.fmeasure - 0.7059).abs() < 1e-4);
/// # Ok::<(), rougers::Error>(())
/// ```
#[derive(Clone, Debug)]
pub struct RougeScorer {
    rouge_types: Vec<RougeType>,
    tokenizer: Tokenizer,
}

impl RougeScorer {
    /// Creates a scorer from variant names such as `"rouge1"` or `"rougeLsum"`.
    pub fn new<I, S>(rouge_types: I, use_stemmer: bool) -> Result<Self, Error>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let rouge_types = rouge_types
            .into_iter()
            .map(|name| name.as_ref().parse())
            .collect::<Result<Vec<RougeType>, Error>>()?;
        Self::with_tokenizer(rouge_types, Tokenizer::new(use_stemmer))
    }

    /// Creates a scorer from typed variants. Fails if an n-gram size is
    /// outside `1..=9`.
    pub fn with_tokenizer(
        rouge_types: Vec<RougeType>,
        tokenizer: Tokenizer,
    ) -> Result<Self, Error> {
        let mut unique = Vec::with_capacity(rouge_types.len());
        for rouge_type in rouge_types {
            if let RougeType::N(n) = rouge_type {
                if !(1..=9).contains(&n) {
                    return Err(Error::InvalidRougeType(rouge_type.to_string()));
                }
            }
            if !unique.contains(&rouge_type) {
                unique.push(rouge_type);
            }
        }
        Ok(RougeScorer {
            rouge_types: unique,
            tokenizer,
        })
    }

    pub fn rouge_types(&self) -> &[RougeType] {
        &self.rouge_types
    }

    pub fn tokenizer(&self) -> &Tokenizer {
        &self.tokenizer
    }

    /// Tokenizes `text` for the requested variants. Sentences for
    /// `rougeLsum` are the non-empty lines of the text.
    pub fn document(&self, text: &str) -> Document {
        if !self.rouge_types.contains(&RougeType::LSum) {
            return Document {
                tokens: self.tokenizer.tokenize(text),
                sentences: Vec::new(),
            };
        }
        // A newline always separates tokens, so the tokens of the whole
        // text are the concatenation of the tokens of its lines.
        let sentences: Vec<Vec<String>> = text
            .split('\n')
            .filter(|line| !line.is_empty())
            .map(|line| self.tokenizer.tokenize(line))
            .collect();
        Document {
            tokens: sentences.concat(),
            sentences,
        }
    }

    pub fn score(&self, target: &str, prediction: &str) -> Scores {
        self.score_tokenized(&self.tokens(target), &self.tokens(prediction))
    }

    pub fn score_documents(&self, target: &Document, prediction: &Document) -> Scores {
        let mut vocabulary = Vocabulary::default();
        let target_tokens = vocabulary.intern(target.tokens.iter().map(String::as_str));
        let prediction_tokens = vocabulary.intern(prediction.tokens.iter().map(String::as_str));
        let lsum = self.rouge_types.contains(&RougeType::LSum);
        let target_sentences = if lsum {
            vocabulary.intern_sentences(&target.sentences)
        } else {
            Vec::new()
        };
        let prediction_sentences = if lsum {
            vocabulary.intern_sentences(&prediction.sentences)
        } else {
            Vec::new()
        };
        self.score_ids(
            Encoded::new(&target_tokens, &target_sentences),
            Encoded::new(&prediction_tokens, &prediction_sentences),
            vocabulary.len(),
        )
    }

    /// Tokens of `text`, with the token ranges of its non-empty lines when
    /// `rougeLsum` is requested. A newline always separates tokens, so the
    /// tokens of the whole text are the concatenation of those of its lines.
    fn tokens(&self, text: &str) -> TokenizedText {
        let mut tokens = Tokens::with_capacity(text.len());
        let mut sentences = Vec::new();
        if self.rouge_types.contains(&RougeType::LSum) {
            for line in text.split('\n').filter(|line| !line.is_empty()) {
                let start = tokens.len();
                self.tokenizer.tokenize_into(line, &mut tokens);
                sentences.push(start..tokens.len());
            }
        } else {
            self.tokenizer.tokenize_into(text, &mut tokens);
        }
        TokenizedText { tokens, sentences }
    }

    fn score_tokenized(&self, target: &TokenizedText, prediction: &TokenizedText) -> Scores {
        let mut vocabulary =
            Vocabulary::with_capacity(target.tokens.len() + prediction.tokens.len());
        let target_ids = vocabulary.intern(target.tokens.iter());
        let prediction_ids = vocabulary.intern(prediction.tokens.iter());
        let target_sentences = slices(&target_ids, &target.sentences);
        let prediction_sentences = slices(&prediction_ids, &prediction.sentences);
        self.score_ids(
            Encoded {
                tokens: &target_ids,
                sentences: target_sentences,
            },
            Encoded {
                tokens: &prediction_ids,
                sentences: prediction_sentences,
            },
            vocabulary.len(),
        )
    }

    fn score_ids(&self, target: Encoded, prediction: Encoded, vocabulary: usize) -> Scores {
        let scores = self
            .rouge_types
            .iter()
            .map(|&rouge_type| {
                let score = match rouge_type {
                    RougeType::N(n) => {
                        metrics::rouge_n(target.tokens, prediction.tokens, n.into(), vocabulary)
                    }
                    RougeType::L => metrics::rouge_l(target.tokens, prediction.tokens, vocabulary),
                    RougeType::LSum => {
                        metrics::rouge_lsum(&target.sentences, &prediction.sentences, vocabulary)
                    }
                };
                (rouge_type, score)
            })
            .collect();
        Scores(scores)
    }

    /// Scores the prediction against each target and keeps, for every
    /// variant separately, the target with the highest F1 (the first one on
    /// ties). Returns `None` when `targets` is empty.
    pub fn score_multi<S: AsRef<str>>(&self, targets: &[S], prediction: &str) -> Option<Scores> {
        let prediction = self.tokens(prediction);
        let all: Vec<Scores> = targets
            .iter()
            .map(|target| self.score_tokenized(&self.tokens(target.as_ref()), &prediction))
            .collect();
        best_per_type(&self.rouge_types, &all)
    }
}

fn best_per_type(rouge_types: &[RougeType], all: &[Scores]) -> Option<Scores> {
    let (first, rest) = all.split_first()?;
    let mut best = first.clone();
    for scores in rest {
        for (slot, (_, candidate)) in best.0.iter_mut().zip(scores.iter()) {
            if candidate.fmeasure > slot.1.fmeasure {
                slot.1 = candidate;
            }
        }
    }
    debug_assert!(best.0.iter().map(|(t, _)| t).eq(rouge_types));
    Some(best)
}

struct TokenizedText {
    tokens: Tokens,
    sentences: Vec<Range<usize>>,
}

/// Token ids of one text and its `rougeLsum` sentences.
struct Encoded<'a> {
    tokens: &'a [u32],
    sentences: Vec<&'a [u32]>,
}

impl<'a> Encoded<'a> {
    fn new(tokens: &'a [u32], sentences: &'a [Vec<u32>]) -> Self {
        Encoded {
            tokens,
            sentences: sentences.iter().map(Vec::as_slice).collect(),
        }
    }
}

fn slices<'a>(ids: &'a [u32], ranges: &[Range<usize>]) -> Vec<&'a [u32]> {
    ranges.iter().map(|range| &ids[range.clone()]).collect()
}

#[derive(Default)]
struct Vocabulary<'a> {
    ids: FxHashMap<&'a str, u32>,
}

impl<'a> Vocabulary<'a> {
    fn with_capacity(tokens: usize) -> Self {
        Vocabulary {
            ids: FxHashMap::with_capacity_and_hasher(tokens, Default::default()),
        }
    }

    fn intern(&mut self, tokens: impl Iterator<Item = &'a str>) -> Vec<u32> {
        tokens
            .map(|token| {
                let next = self.ids.len() as u32;
                *self.ids.entry(token).or_insert(next)
            })
            .collect()
    }

    fn intern_sentences(&mut self, sentences: &'a [Vec<String>]) -> Vec<Vec<u32>> {
        sentences
            .iter()
            .map(|sentence| self.intern(sentence.iter().map(String::as_str)))
            .collect()
    }

    fn len(&self) -> usize {
        self.ids.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_rouge_score_readme_example() {
        let scorer = RougeScorer::new(["rouge1", "rougeL"], true).unwrap();
        let scores = scorer.score(
            "The quick brown fox jumps over the lazy dog",
            "The quick brown dog jumps on the log.",
        );
        let rouge1 = scores.get(RougeType::N(1)).unwrap();
        assert_eq!(rouge1.precision, 0.75);
        assert_eq!(rouge1.recall, 0.6666666666666666);
        assert_eq!(rouge1.fmeasure, 0.7058823529411765);
        let rouge_l = scores.get(RougeType::L).unwrap();
        assert_eq!(rouge_l.precision, 0.625);
        assert_eq!(rouge_l.recall, 0.5555555555555556);
        assert_eq!(rouge_l.fmeasure, 0.5882352941176471);
    }

    #[test]
    fn keeps_first_occurrence_of_duplicate_types() {
        let scorer = RougeScorer::new(["rougeL", "rouge1", "rougeL"], false).unwrap();
        assert_eq!(scorer.rouge_types(), [RougeType::L, RougeType::N(1)]);
    }

    #[test]
    fn rejects_out_of_range_ngram_sizes() {
        for n in [0, 10] {
            let result = RougeScorer::with_tokenizer(vec![RougeType::N(n)], Tokenizer::default());
            assert!(matches!(result, Err(Error::InvalidRougeType(_))));
        }
    }

    #[test]
    fn splits_lsum_sentences_on_newlines() {
        let scorer = RougeScorer::new(["rougeLsum"], false).unwrap();
        let document = scorer.document("one two\n\nthree");
        assert_eq!(document.sentences, [vec!["one", "two"], vec!["three"]]);
        assert_eq!(document.tokens, ["one", "two", "three"]);
    }

    #[test]
    fn score_multi_picks_best_target_per_type() {
        let scorer = RougeScorer::new(["rouge1", "rouge2"], false).unwrap();
        let best = scorer.score_multi(&["a b c d", "a c b"], "a b c").unwrap();
        assert_eq!(best.get(RougeType::N(1)).unwrap().fmeasure, 1.0);
        assert_eq!(best.get(RougeType::N(2)).unwrap().recall, 2.0 / 3.0);
        assert!(scorer.score_multi::<&str>(&[], "a").is_none());
    }
}
