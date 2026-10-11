/// Precision, recall and F1 for one ROUGE variant.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Score {
    pub precision: f64,
    pub recall: f64,
    pub fmeasure: f64,
}

impl Score {
    pub(crate) const ZERO: Score = Score {
        precision: 0.0,
        recall: 0.0,
        fmeasure: 0.0,
    };

    pub(crate) fn from_counts(hits: usize, prediction_len: usize, target_len: usize) -> Score {
        let precision = hits as f64 / prediction_len as f64;
        let recall = hits as f64 / target_len as f64;
        Score {
            precision,
            recall,
            fmeasure: fmeasure(precision, recall),
        }
    }
}

/// The harmonic mean of precision and recall, or zero when both are zero.
pub fn fmeasure(precision: f64, recall: f64) -> f64 {
    if precision + recall > 0.0 {
        2.0 * precision * recall / (precision + recall)
    } else {
        0.0
    }
}
