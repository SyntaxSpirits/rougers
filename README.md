# rougers

[![CI](https://github.com/SyntaxSpirits/rougers/actions/workflows/ci.yml/badge.svg)](https://github.com/SyntaxSpirits/rougers/actions/workflows/ci.yml)
[![PyPI](https://img.shields.io/pypi/v/rouge-score-rs.svg)](https://pypi.org/project/rouge-score-rs/)
[![Crates.io](https://img.shields.io/crates/v/rougers.svg)](https://crates.io/crates/rougers)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

ROUGE metrics in Rust, with scores identical to Google's
[`rouge-score`](https://pypi.org/project/rouge-score/). On CNN/DailyMail, XSum and PubMed
test samples it scores 11–39 times faster on one core and 40–200 times faster with all
eight cores of an M1 Pro ([benchmarks](https://github.com/SyntaxSpirits/rougers/blob/main/BENCHMARKS.md)).

`rouge-score` is the ROUGE implementation behind Hugging Face `evaluate` and many
summarization and LLM evaluation pipelines. It is written in pure Python, so scoring large
sets of outputs, or scoring repeatedly, takes noticeable time. `rougers`
reproduces it exactly, down to the last bit of every float, including the tokenizer,
NLTK's Porter stemmer and the summary-level `rougeLsum`.

The Rust crate is `rougers`; the Python package built on it is `rouge-score-rs`. This is an
independent project, not affiliated with Google or the authors of `rouge-score`.

## Python

```sh
pip install rouge-score-rs
```

Change the import and keep the rest of your code:

```python
from rouge_score_rs import rouge_scorer  # was: from rouge_score import rouge_scorer

scorer = rouge_scorer.RougeScorer(["rouge1", "rouge2", "rougeL", "rougeLsum"], use_stemmer=True)
scores = scorer.score(
    "The quick brown fox jumps over the lazy dog",
    "The quick brown dog jumps on the log.",
)
scores["rouge1"]
# Score(precision=0.75, recall=0.6666666666666666, fmeasure=0.7058823529411765)
```

`score_multi`, custom tokenizers, `split_summaries`, `scoring.BootstrapAggregator` and
`tokenize.tokenize` work the same way as in `rouge-score`, and scorers can be pickled
for use in worker processes.

To score many pairs, use `score_batch`. With the default tokenizer it runs on all CPU
cores and releases the GIL:

```python
results = scorer.score_batch(references, predictions)  # list of dicts, one per pair
```

Set `RAYON_NUM_THREADS` to limit the number of threads. With a custom tokenizer or
`split_summaries=True`, and in a process forked after a parallel call, pairs are scored
one by one. The built-in tokenizers described below also run in parallel.

## Other languages

The default tokenizer is the one from `rouge-score`: it keeps only the ASCII letters `a`–`z`
and digits, so Russian, Arabic, Hindi or Chinese text scores zero. Two opt-in tokenizers
cover other languages and run natively, including in `score_batch`:

```python
from rouge_score_rs import rouge_scorer, tokenizers

scorer = rouge_scorer.RougeScorer(["rouge1", "rougeL"], tokenizer=tokenizers.UnicodeTokenizer())
scorer.score("Правительство объявило новые меры.", "Объявлены новые меры правительства.")

chinese = rouge_scorer.RougeScorer(["rouge1", "rougeL"], tokenizer=tokenizers.CharacterTokenizer())
chinese.score("今天天气很好", "今天天气不错")
```

| Tokenizer | Profile | Tokens |
|---|---|---|
| `DefaultTokenizer` | `rouge-score` | runs of `[a-z0-9]` after lowercasing; optional Porter stemming |
| `UnicodeTokenizer` | `unicode-v1` | words by the Unicode word boundary rules (UAX #29) that contain an alphabetic character or a number |
| `CharacterTokenizer` | `character-v1` | grapheme clusters that contain an alphabetic character or a number |

Both profiles first normalize the text: NFKC, full Unicode case folding, then NFKC again,
so `Straße`, `STRASSE` and `ｓｔｒａｓｓｅ` give the same token.

`UnicodeTokenizer` suits languages that separate words with spaces. It makes each Chinese
or Japanese ideograph a separate token and does not split Thai, Lao, Khmer or Burmese into
words; for those, pass your own tokenizer. `CharacterTokenizer` gives character-level
ROUGE, a common choice for Chinese. Neither stems. Scores from these profiles are not
comparable with `rouge-score` defaults or with other tokenization schemes, so report the
profile name with your results. The profiles use Unicode 17.0 data from pinned
dependencies, so their output does not depend on the platform or the Rust version, and a
profile's behaviour will not change under the same name.

## Hugging Face Evaluate

The [`SyntaxSpirits/rouge`](https://huggingface.co/spaces/SyntaxSpirits/rouge) metric has
the same inputs, options and scores as Evaluate's built-in `rouge` metric and computes them
with this package:

```python
import evaluate

rouge = evaluate.load("SyntaxSpirits/rouge")  # was: evaluate.load("rouge")
results = rouge.compute(predictions=predictions, references=references)
```

It needs `pip install evaluate 'rouge-score-rs[aggregate]>=0.1.1'`.

## Rust

```toml
[dependencies]
rougers = "0.2"
```

```rust
use rougers::{RougeScorer, RougeType};

let scorer = RougeScorer::new(["rouge1", "rougeL"], true)?;
let scores = scorer.score(
    "The quick brown fox jumps over the lazy dog",
    "The quick brown dog jumps on the log.",
);
let rouge_l = scores.get(RougeType::L).unwrap();
println!("{:.4}", rouge_l.fmeasure);
```

`RougeScorer` is `Send + Sync`, so it can be shared across threads, for example with
Rayon.

## Performance

The benchmark uses fixed samples from the CNN/DailyMail, XSum and PubMed test sets
with lead-sentence predictions. It checks score parity, measures serial and parallel
scoring with and without stemming, and includes Hugging Face Evaluate's default
bootstrap aggregation.

See [BENCHMARKS.md](https://github.com/SyntaxSpirits/rougers/blob/main/BENCHMARKS.md)
for measured results, pinned data sources, package comparisons and reproduction
commands. Speedups depend on summary length, requested metrics, stemming and thread
count; timings for packages computing different scores are reported separately.

## Compatibility

The test suite compares `rouge-score-rs` with `rouge-score` and fails on any difference
in any float:

- the stemmer against NLTK on all 235,976 words of a system dictionary (NLTK 3.8–3.10
  stem them identically);
- tokenization against Python's `str.lower` for every Unicode code point;
- thousands of random texts with repeated words, punctuation, digits, newlines and
  non-Latin scripts, plus property-based tests on arbitrary Unicode;
- `score_multi`, custom tokenizers, `split_summaries` and bootstrap aggregation with
  a fixed NumPy seed.

Known differences:

- An invalid ROUGE type raises `ValueError` when the scorer is created, not on the
  first call to `score`. Type names must be exactly `rouge1`…`rouge9`, `rougeL` or
  `rougeLsum`.
- Texts must be `str`; `rouge-score` also accepts UTF-8 `bytes`.
- `BootstrapAggregator` needs NumPy (`pip install 'rouge-score-rs[aggregate]'`) and
  `split_summaries=True` needs NLTK (`pip install 'rouge-score-rs[sentences]'`).
  Without them, `rouge-score-rs` has no Python dependencies.

## Development

```sh
cargo test -p rougers
cd bindings/python
uv sync --group dev
uv run pytest
```

## License

Apache-2.0, the same licence as `rouge-score` and NLTK, whose scoring, tokenization and
stemming logic this project ports. See [NOTICE](NOTICE).
