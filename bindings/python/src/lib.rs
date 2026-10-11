use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyType;
use rayon::prelude::*;
use rougers::{Document, RougeScorer, RougeType, Scores, Tokenizer};
use std::sync::OnceLock;

/// Precision, recall and F1, in the order of the scorer's ROUGE types.
type ScoreRow = (f64, f64, f64);

fn rows(scores: Scores) -> Vec<ScoreRow> {
    scores
        .into_iter()
        .map(|(_, s)| (s.precision, s.recall, s.fmeasure))
        .collect()
}

#[pyclass(frozen, module = "rouge_score_rs._rougers")]
struct Scorer {
    inner: RougeScorer,
}

#[pymethods]
impl Scorer {
    #[new]
    #[pyo3(signature = (rouge_types, use_stemmer, profile = "rouge-score"))]
    fn new(rouge_types: Vec<String>, use_stemmer: bool, profile: &str) -> PyResult<Self> {
        let tokenizer = tokenizer(profile, use_stemmer)?;
        rouge_types
            .iter()
            .map(|name| name.parse::<RougeType>())
            .collect::<Result<Vec<_>, _>>()
            .and_then(|rouge_types| RougeScorer::with_tokenizer(rouge_types, tokenizer))
            .map(|inner| Scorer { inner })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }

    fn __reduce__<'py>(
        slf: &Bound<'py, Self>,
    ) -> (Bound<'py, PyType>, (Vec<String>, bool, &'static str)) {
        let inner = &slf.get().inner;
        let rouge_types = inner
            .rouge_types()
            .iter()
            .map(ToString::to_string)
            .collect();
        let tokenizer = inner.tokenizer();
        (
            slf.get_type(),
            (rouge_types, tokenizer.use_stemmer(), tokenizer.profile()),
        )
    }

    fn score(&self, py: Python<'_>, target: &str, prediction: &str) -> Vec<ScoreRow> {
        py.detach(|| rows(self.inner.score(target, prediction)))
    }

    fn score_documents(
        &self,
        target_tokens: Vec<String>,
        target_sentences: Vec<Vec<String>>,
        prediction_tokens: Vec<String>,
        prediction_sentences: Vec<Vec<String>>,
    ) -> Vec<ScoreRow> {
        let target = Document {
            tokens: target_tokens,
            sentences: target_sentences,
        };
        let prediction = Document {
            tokens: prediction_tokens,
            sentences: prediction_sentences,
        };
        rows(self.inner.score_documents(&target, &prediction))
    }

    fn score_multi(
        &self,
        py: Python<'_>,
        targets: Vec<String>,
        prediction: &str,
    ) -> Option<Vec<ScoreRow>> {
        py.detach(|| self.inner.score_multi(&targets, prediction).map(rows))
    }

    fn score_batch(
        &self,
        py: Python<'_>,
        targets: Vec<String>,
        predictions: Vec<String>,
    ) -> PyResult<Vec<Vec<ScoreRow>>> {
        if targets.len() != predictions.len() {
            return Err(PyValueError::new_err(format!(
                "targets and predictions differ in length: {} != {}",
                targets.len(),
                predictions.len()
            )));
        }
        let score =
            |(target, prediction): (&String, &String)| rows(self.inner.score(target, prediction));
        Ok(py.detach(|| {
            if thread_pool_usable() {
                targets
                    .par_iter()
                    .zip(predictions.par_iter())
                    .map(score)
                    .collect()
            } else {
                targets.iter().zip(predictions.iter()).map(score).collect()
            }
        }))
    }
}

/// The process that started Rayon's global thread pool. The pool's threads
/// do not survive `fork()`, so a forked child that used it would wait forever;
/// such children score on the calling thread instead.
static POOL_OWNER: OnceLock<u32> = OnceLock::new();

fn thread_pool_usable() -> bool {
    *POOL_OWNER.get_or_init(std::process::id) == std::process::id()
}

fn tokenizer(profile: &str, use_stemmer: bool) -> PyResult<Tokenizer> {
    match profile {
        "rouge-score" => Ok(Tokenizer::new(use_stemmer)),
        "unicode-v1" => Ok(Tokenizer::unicode()),
        "character-v1" => Ok(Tokenizer::character()),
        _ => Err(PyValueError::new_err(format!(
            "unknown tokenizer profile: {profile}"
        ))),
    }
}

#[pyfunction]
#[pyo3(signature = (text, use_stemmer, profile = "rouge-score"))]
fn tokenize(text: &str, use_stemmer: bool, profile: &str) -> PyResult<Vec<String>> {
    Ok(tokenizer(profile, use_stemmer)?.tokenize(text))
}

#[pymodule]
mod _rougers {
    #[pymodule_export]
    use super::{tokenize, Scorer};

    #[pymodule_init]
    fn init(m: &pyo3::Bound<'_, pyo3::types::PyModule>) -> pyo3::PyResult<()> {
        use pyo3::types::PyModuleMethods;
        m.add("__version__", env!("CARGO_PKG_VERSION"))
    }
}
