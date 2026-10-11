# Changelog

## 0.2.0 - Unreleased

- `UnicodeTokenizer` (profile `unicode-v1`): Unicode word boundaries after lowercasing
  and NFKC normalization, for languages in any script.
- `CharacterTokenizer` (profile `character-v1`): one token per grapheme cluster, for
  character-level ROUGE such as in Chinese.
- Both run natively, including in parallel `score_batch`. The default tokenizer is
  unchanged.
- Rust: `Tokenizer::unicode`, `Tokenizer::character` and `Tokenizer::profile`.
  Minimum Rust version 1.85.

## 0.1.1 - 2026-10-11

Python package only; the Rust crate is unchanged.

- `score_batch` no longer hangs in a process forked after it ran in the parent; such
  processes score on a single thread.
- Pickling and `copy.deepcopy` keep the `__slots__` of subclasses.
- Changing `use_stemmer` on a scorer's tokenizer after construction takes effect.
- `score_multi` and `score_batch` respect a `score` method replaced on the instance.
- `tokenize.tokenize` accepts stemmers that return `bytes`, like `rouge-score`.

## 0.1.0 - 2026-10-11

First release.

- `rouge1`–`rouge9`, `rougeL` and `rougeLsum` with scores identical to `rouge-score` 0.1.2.
- Tokenizer and NLTK Porter stemmer matching `rouge-score`.
- Python package `rouge-score-rs` (import `rouge_score_rs`) with the `rouge_scorer`, `scoring`, `tokenize` and `tokenizers` modules
  of `rouge-score`, plus a parallel `score_batch`. Scorers can be pickled.
