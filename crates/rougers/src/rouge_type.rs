use std::fmt;
use std::str::FromStr;

use crate::Error;

/// A ROUGE variant, named as in `rouge-score`: `rouge1` to `rouge9`,
/// `rougeL` and `rougeLsum`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RougeType {
    /// N-gram overlap for `n` in `1..=9`.
    N(u8),
    /// Longest common subsequence over the whole text.
    L,
    /// Summary-level longest common subsequence over newline-separated sentences.
    LSum,
}

impl FromStr for RougeType {
    type Err = Error;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "rougeL" => Ok(RougeType::L),
            "rougeLsum" => Ok(RougeType::LSum),
            _ => match name.strip_prefix("rouge").map(str::as_bytes) {
                Some([digit @ b'1'..=b'9']) => Ok(RougeType::N(digit - b'0')),
                Some([b'0']) => Err(Error::NonPositiveN(name.to_owned())),
                _ => Err(Error::InvalidRougeType(name.to_owned())),
            },
        }
    }
}

impl fmt::Display for RougeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RougeType::N(n) => write!(f, "rouge{n}"),
            RougeType::L => f.write_str("rougeL"),
            RougeType::LSum => f.write_str("rougeLsum"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RougeType;
    use crate::Error;

    #[test]
    fn parses_names_used_by_rouge_score() {
        assert_eq!("rouge1".parse::<RougeType>().unwrap(), RougeType::N(1));
        assert_eq!("rouge9".parse::<RougeType>().unwrap(), RougeType::N(9));
        assert_eq!("rougeL".parse::<RougeType>().unwrap(), RougeType::L);
        assert_eq!("rougeLsum".parse::<RougeType>().unwrap(), RougeType::LSum);
    }

    #[test]
    fn rejects_unknown_names() {
        assert!(matches!(
            "rouge0".parse::<RougeType>(),
            Err(Error::NonPositiveN(_))
        ));
        for name in ["rouge10", "rougel", "rouge", "bleu", ""] {
            assert!(matches!(
                name.parse::<RougeType>(),
                Err(Error::InvalidRougeType(_))
            ));
        }
    }

    #[test]
    fn displays_original_name() {
        for name in ["rouge2", "rougeL", "rougeLsum"] {
            assert_eq!(name.parse::<RougeType>().unwrap().to_string(), name);
        }
    }
}
