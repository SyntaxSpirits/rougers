use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyTuple, PyType};
use pyo3::{PyTraverseError, PyVisit};
use rayon::prelude::*;
use rougers::{Document, RougeScorer, RougeType, Scores, Tokenizer};
use std::sync::OnceLock;

#[pyclass(frozen, module = "rouge_score_rs._rougers")]
struct Scorer {
    inner: RougeScorer,
    /// The namedtuple class used for each score, `rouge_score_rs.scoring.Score`.
    score_class: Py<PyType>,
    /// Interned result keys, one per ROUGE type.
    names: Vec<Py<PyString>>,
    /// `tuple.__new__`, which builds a namedtuple without running Python code.
    tuple_new: Py<PyAny>,
}

impl Scorer {
    fn to_dict<'py>(&self, py: Python<'py>, scores: Scores) -> PyResult<Bound<'py, PyDict>> {
        let dict = PyDict::new(py);
        let tuple_new = self.tuple_new.bind(py);
        let score_class = self.score_class.bind(py);
        for (name, (_, score)) in self.names.iter().zip(scores) {
            let values = PyTuple::new(py, [score.precision, score.recall, score.fmeasure])?;
            dict.set_item(name.bind(py), tuple_new.call1((score_class, values))?)?;
        }
        Ok(dict)
    }
}

#[pymethods]
impl Scorer {
    #[new]
    #[pyo3(signature = (rouge_types, use_stemmer, profile = "rouge-score", score_class = None))]
    fn new(
        py: Python<'_>,
        rouge_types: Vec<String>,
        use_stemmer: bool,
        profile: &str,
        score_class: Option<Py<PyType>>,
    ) -> PyResult<Self> {
        // Scorers pickled by 0.2.0 and earlier were saved without a score class.
        let score_class = match score_class {
            Some(score_class) => score_class,
            None => py
                .import("rouge_score_rs.scoring")?
                .getattr("Score")?
                .extract()?,
        };
        let tokenizer = tokenizer(profile, use_stemmer)?;
        let inner = rouge_types
            .iter()
            .map(|name| name.parse::<RougeType>())
            .collect::<Result<Vec<_>, _>>()
            .and_then(|rouge_types| RougeScorer::with_tokenizer(rouge_types, tokenizer))
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let names = inner
            .rouge_types()
            .iter()
            .map(|rouge_type| PyString::intern(py, &rouge_type.to_string()).unbind())
            .collect();
        let tuple_new = py.get_type::<PyTuple>().getattr("__new__")?.unbind();
        Ok(Scorer {
            inner,
            score_class,
            names,
            tuple_new,
        })
    }

    fn __traverse__(&self, visit: PyVisit<'_>) -> Result<(), PyTraverseError> {
        visit.call(&self.score_class)?;
        visit.call(&self.tuple_new)?;
        for name in &self.names {
            visit.call(name)?;
        }
        Ok(())
    }

    #[allow(clippy::type_complexity)]
    fn __reduce__<'py>(
        slf: &Bound<'py, Self>,
    ) -> (
        Bound<'py, PyType>,
        (Vec<String>, bool, &'static str, Bound<'py, PyType>),
    ) {
        let scorer = slf.get();
        let rouge_types = scorer
            .inner
            .rouge_types()
            .iter()
            .map(ToString::to_string)
            .collect();
        let tokenizer = scorer.inner.tokenizer();
        (
            slf.get_type(),
            (
                rouge_types,
                tokenizer.use_stemmer(),
                tokenizer.profile(),
                scorer.score_class.bind(slf.py()).clone(),
            ),
        )
    }

    fn score<'py>(
        &self,
        py: Python<'py>,
        target: &str,
        prediction: &str,
    ) -> PyResult<Bound<'py, PyDict>> {
        let scores = py.detach(|| self.inner.score(target, prediction));
        self.to_dict(py, scores)
    }

    fn score_documents<'py>(
        &self,
        py: Python<'py>,
        target_tokens: Vec<String>,
        target_sentences: Vec<Vec<String>>,
        prediction_tokens: Vec<String>,
        prediction_sentences: Vec<Vec<String>>,
    ) -> PyResult<Bound<'py, PyDict>> {
        let target = Document {
            tokens: target_tokens,
            sentences: target_sentences,
        };
        let prediction = Document {
            tokens: prediction_tokens,
            sentences: prediction_sentences,
        };
        self.to_dict(py, self.inner.score_documents(&target, &prediction))
    }

    fn score_multi<'py>(
        &self,
        py: Python<'py>,
        targets: Vec<String>,
        prediction: &str,
    ) -> PyResult<Option<Bound<'py, PyDict>>> {
        py.detach(|| self.inner.score_multi(&targets, prediction))
            .map(|scores| self.to_dict(py, scores))
            .transpose()
    }

    fn score_batch<'py>(
        &self,
        py: Python<'py>,
        targets: Vec<String>,
        predictions: Vec<String>,
    ) -> PyResult<Vec<Bound<'py, PyDict>>> {
        if targets.len() != predictions.len() {
            return Err(PyValueError::new_err(format!(
                "targets and predictions differ in length: {} != {}",
                targets.len(),
                predictions.len()
            )));
        }
        let score = |(target, prediction): (&String, &String)| self.inner.score(target, prediction);
        let all: Vec<Scores> = py.detach(|| {
            if thread_pool_usable() {
                targets
                    .par_iter()
                    .zip(predictions.par_iter())
                    .map(score)
                    .collect()
            } else {
                targets.iter().zip(predictions.iter()).map(score).collect()
            }
        });
        all.into_iter()
            .map(|scores| self.to_dict(py, scores))
            .collect()
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
