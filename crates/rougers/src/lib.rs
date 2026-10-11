//! Fast ROUGE metrics whose scores are identical to Google's
//! [`rouge-score`](https://github.com/google-research/google-research/tree/master/rouge)
//! Python package, including its tokenizer, NLTK's Porter stemmer and the
//! summary-level `rougeLsum` variant.
//!
//! ```
//! use rougers::{RougeScorer, RougeType};
//!
//! let scorer = RougeScorer::new(["rouge1", "rouge2", "rougeLsum"], true)?;
//! let scores = scorer.score("the cat sat on the mat", "a cat was sitting on the mat");
//! for (rouge_type, score) in scores.iter() {
//!     println!("{rouge_type}: {:.4}", score.fmeasure);
//! }
//! # Ok::<(), rougers::Error>(())
//! ```

mod metrics;
mod porter;
mod rouge_type;
mod score;
mod scorer;
mod tokenizer;

use std::fmt;

pub use rouge_type::RougeType;
pub use score::{fmeasure, Score};
pub use scorer::{Document, RougeScorer, Scores};
pub use tokenizer::Tokenizer;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The name is not `rouge1`..`rouge9`, `rougeL` or `rougeLsum`.
    InvalidRougeType(String),
    /// `rouge0` was requested.
    NonPositiveN(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::InvalidRougeType(name) => write!(f, "Invalid rouge type: {name}"),
            Error::NonPositiveN(name) => write!(f, "rougen requires positive n: {name}"),
        }
    }
}

impl std::error::Error for Error {}
