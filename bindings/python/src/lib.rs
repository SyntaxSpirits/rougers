use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::prelude::*;
use rougers::{Document, RougeScorer, Scores, Tokenizer};

type ScoreRow = (String, f64, f64, f64);

fn rows(scores: Scores) -> Vec<ScoreRow> {
    scores
        .into_iter()
        .map(|(rouge_type, s)| (rouge_type.to_string(), s.precision, s.recall, s.fmeasure))
        .collect()
}

#[pyclass(frozen, module = "rougers._rougers")]
struct Scorer {
    inner: RougeScorer,
}

#[pymethods]
impl Scorer {
    #[new]
    fn new(rouge_types: Vec<String>, use_stemmer: bool) -> PyResult<Self> {
        RougeScorer::new(&rouge_types, use_stemmer)
            .map(|inner| Scorer { inner })
            .map_err(|e| PyValueError::new_err(e.to_string()))
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
        Ok(py.detach(|| {
            targets
                .par_iter()
                .zip(predictions.par_iter())
                .map(|(target, prediction)| rows(self.inner.score(target, prediction)))
                .collect()
        }))
    }
}

#[pyfunction]
fn tokenize(text: &str, use_stemmer: bool) -> Vec<String> {
    Tokenizer::new(use_stemmer).tokenize(text)
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
