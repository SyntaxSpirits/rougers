# Changelog

## 0.1.0 - 2026-10-11

First release.

- `rouge1`–`rouge9`, `rougeL` and `rougeLsum` with scores identical to `rouge-score` 0.1.2.
- Tokenizer and NLTK Porter stemmer matching `rouge-score`.
- Python package `rouge-score-rs` (import `rouge_score_rs`) with the `rouge_scorer`, `scoring`, `tokenize` and `tokenizers` modules
  of `rouge-score`, plus a parallel `score_batch`. Scorers can be pickled.
