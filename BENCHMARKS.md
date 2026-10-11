# Real-data ROUGE benchmarks

These measurements replace the Gutenberg synthetic benchmark. They measure summary
scoring, not generation or scoring the entire source document. Results apply to this
sample, these package versions and this machine.

## Method

Select 2,000 rows uniformly without replacement from each full test split with
`random.Random(1729).sample`, then sort the row indices. Predictions are the first
3 sentences of CNN/DailyMail articles, 1 of XSum documents, and 5 of PubMed articles.
Split on existing newlines or whitespace following `.`, `!` or `?`; remove PubMed
`<S>`/`</S>` markers. Join reference and prediction sentences with `\n`. This simple
rule needs no downloaded sentence tokenizer and can split abbreviations; it is fixed
for every implementation. No article or reference is truncated beyond the lead
prediction. Dataset preparation and downloads are outside scoring timings.

| Test source | Pinned revision | Parquet file | Download MB |
|---|---|---|---:|
| [CNN/DailyMail 3.0.0][data-cnn] | `96df5e686bee6baa90b8bee7c28b81fa3fa6223d` | `3.0.0/test-00000-of-00001.parquet` | 30.00 |
| [XSum][data-xsum] | `7d4d486c2f8ef850b1a11aead99b894ff3dd7da9` | `data/test-00000-of-00001.parquet` | 16.73 |
| [PubMed, document configuration][data-pubmed] | `6b30a2cae59b11ed77cb19959bffccbbd18e1106` | `document/test-00000-of-00001.parquet` | 58.46 |

Only these files are downloaded: 105,196,681 bytes total (105.20 MB). Each is verified
against its pinned SHA-256 before use. The default cache is
`~/.cache/rouge-score-rs-bench`; dataset files are never stored in the repository.
Exact URLs, hashes, selected row indices and observed text lengths are in the JSON.

Each configuration runs in a fresh Python process with `RAYON_NUM_THREADS` set before
imports. Timings are the median of three complete calls following one complete
warmup. Raw samples are retained. Input construction, imports, scorer construction,
normalization for comparison, and parity checks are outside direct scoring timings;
Python result construction is included. Thread counts are 1, 2, 4 and all logical
CPUs (8 here), without CPU affinity. Sub-millisecond subset timings are especially
sensitive to noise; this is one machine/run, not a confidence interval.

The full comparison requests `rouge1`, `rouge2`, `rougeL`, `rougeLsum`, with stemming
both disabled and enabled. Every P/R/F value from every Rust result must exactly
match `rouge-score` 0.1.2, without a tolerance. Repetitions must also agree. The
`rouge-rust` comparison separately computes only `rouge1/2/L`, without stemming,
including matching Google and local-package baselines. Its mismatched pair/field
counts and up to three examples are recorded; a speedup ratio is omitted if scores
differ. `rouge` (pltrdy) runs its default metrics unchanged and is only a timing
reference for a different computation.[^google][^rust][^pltrdy]

Evaluate uses the actual `evaluate.load("rouge")` and
`evaluate.load("SyntaxSpirits/rouge")` APIs, pinned to metric revisions
`1bc20e7e9df922e5dfb597c31a4ea8467a89caff` and
`3950ae735ae7a66bbae41f73597ea84549b6d010`. Both receive `seed=1729` at load and retain
the default aggregation (1,000 bootstrap samples). Aggregate results must match
exactly. Reported compute time includes input handling, scoring and aggregation;
load time is measured separately and depends on cache/network state. Load + compute
adds that single load to median warm compute, rather than claiming a cold-start
measurement. Scoring time comes from a separate instrumented compute pass and
includes the scorer calls only; instrumentation is absent from reported compute
timings. Python imports are excluded from both.[^evaluate]

Memory is `resource.getrusage(RUSAGE_SELF).ru_maxrss`, converted to MiB using the
platform's units. It is each worker's whole-process peak, including imports, warmup,
results and the extra Evaluate instrumentation pass. It captures native memory but
does not isolate scorer allocations; do not interpret differences as per-pair
memory costs.[^rss]

## Environment and results

Run on 2026-10-11: Apple M1 Pro, 8 logical CPUs, macOS 14.5 arm64, Python 3.12.11.
Local `rouge-score-rs` was built from Git revision `8e7aa7d` with maturin 1.15.0 in release mode; its library and
binding code is identical to the 0.2.1 release (the package list in the JSON shows a stale 0.1.0 from the
environment's install metadata).
Comparison versions: `rouge-score` 0.1.2, `rouge-rust` 0.1.12, `rouge` 1.0.1, Evaluate
0.4.6, datasets 5.1.0, NumPy 2.5.4, NLTK 3.10.3 and PyArrow 26.0.0. All installed versions
and the library Git revision are in
[the machine-readable results](bindings/python/benchmarks/results/m1-pro.json).

The complete run took 646 seconds, including downloads and worker startup.
All expected parity checks passed. `rouge-rust` had **0/6,000 mismatched pairs** and
**0/54,000 mismatched P/R/F fields** for each tested API/thread setting; there are no
mismatch examples to display. That is evidence for this sample, not a general proof.

| Workload | Mean article words | Mean reference words | Mean prediction words |
|---|---:|---:|---:|
| cnn_dailymail | 687.5 | 54.7 | 75.8 |
| xsum | 381.8 | 20.9 | 19.9 |
| pubmed | 3038.3 | 205.0 | 150.4 |

Full four-metric scoring, seconds for 2,000 pairs. Batch columns are `rouge-score-rs`.

| Workload | Stem | Google loop | RS loop | Batch 1 | Batch 2 | Batch 4 | Batch 8 | Batch 8 speedup |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| cnn_dailymail | no | 2.5209 | 0.0428 | 0.0442 | 0.0257 | 0.0245 | 0.0105 | 239.0× |
| cnn_dailymail | yes | 5.2720 | 0.0929 | 0.0851 | 0.0462 | 0.0251 | 0.0175 | 300.4× |
| xsum | no | 0.3758 | 0.0126 | 0.0103 | 0.0066 | 0.0045 | 0.0045 | 82.6× |
| xsum | yes | 1.2187 | 0.0247 | 0.0237 | 0.0132 | 0.0079 | 0.0062 | 196.2× |
| pubmed | no | 13.6516 | 0.1693 | 0.1682 | 0.0867 | 0.0464 | 0.0438 | 311.4× |
| pubmed | yes | 19.6462 | 0.2804 | 0.3042 | 0.1479 | 0.0993 | 0.0557 | 352.5× |

Across these six workloads/settings, the RS loop is **29.7–80.6×** faster than
Google; batch with 8 threads is **82.6–352.5×** faster. These are scoring-only ratios.

Like-for-like subset (`rouge1/2/L`, no stemming), seconds for 2,000 pairs:

| Workload | Google loop | RS loop | RS batch 1 | RS batch 8 | rouge-rust loop | rouge-rust batch 1 | rouge-rust batch 8 |
|---|---:|---:|---:|---:|---:|---:|---:|
| cnn_dailymail | 1.3252 | 0.0172 | 0.0160 | 0.0047 | 0.0384 | 0.0389 | 0.0074 |
| xsum | 0.2082 | 0.0075 | 0.0066 | 0.0029 | 0.0098 | 0.0085 | 0.0042 |
| pubmed | 6.9406 | 0.0355 | 0.0346 | 0.0081 | 0.1619 | 0.1618 | 0.0269 |

Relative to `rouge-rust` on this subset, `rouge-score-rs` is cnn_dailymail: 2.24× (loop), 1.57× (8 threads); xsum: 1.30× (loop), 1.43× (8 threads); pubmed: 4.56× (loop), 3.33× (8 threads) (values above 1 mean `rouge-score-rs` is faster). `rouge-rust` does not provide stemming or `rougeLsum`; the full four-metric timings above are not a like-for-like comparison with it.[^rust]

Evaluate with default bootstrap aggregation, seconds for 2,000 pairs. Each pair of
columns is Google / SyntaxSpirits. Scoring is a separate instrumented pass, so it
need not be smaller than median compute time.

| Workload | Stem | Scoring s | Compute s | Load s | Load + compute s | Compute speedup |
|---|---|---:|---:|---:|---:|---:|
| cnn_dailymail | no | 2.8886 / 0.0098 | 2.6896 / 0.2335 | 0.9023 / 0.6752 | 3.5918 / 0.9088 | 11.52× |
| cnn_dailymail | yes | 5.1605 / 0.0489 | 5.3930 / 0.3006 | 0.8012 / 0.3579 | 6.1941 / 0.6585 | 17.94× |
| xsum | no | 0.3841 / 0.0039 | 0.7498 / 0.2258 | 1.0971 / 0.2950 | 1.8468 / 0.5208 | 3.32× |
| xsum | yes | 1.2187 / 0.0065 | 1.4617 / 0.2233 | 0.6947 / 0.3025 | 2.1564 / 0.5258 | 6.55× |
| pubmed | no | 13.2295 / 0.0437 | 21.5716 / 0.3191 | 0.9735 / 0.4257 | 22.5452 / 0.7448 | 67.61× |
| pubmed | yes | 20.0144 / 0.0579 | 20.6311 / 0.2868 | 0.8930 / 0.3666 | 21.5241 / 0.6534 | 71.94× |

Evaluate compute speedups are **3.32–71.94×**, much smaller than scoring-only
speedups because aggregation and input handling remain. Load timings are reported
for completeness and must not be treated as a stable package speed comparison.

Peak RSS ranges over all workloads and settings; pltrdy times are a **different
computation**, with no speedup ratio:

| Implementation | Whole-worker peak RSS MiB | pltrdy seconds, CNN/DM / XSum / PubMed |
|---|---:|---:|
| rouge-score direct | 74.0–96.1 | — |
| rouge-score-rs direct | 38.5–69.1 | — |
| rouge-rust | 41.7–60.7 | — |
| Evaluate rouge | 180.1–199.1 | — |
| Evaluate SyntaxSpirits/rouge | 164.6–207.6 | — |
| rouge/pltrdy | 46.6–104.0 | 2.8526 / 0.3411 / 25.4146 |

The [complete Markdown output](bindings/python/benchmarks/results/m1-pro.md)
contains all 72 measurements, throughput, per-configuration RSS and mismatch counts.
The JSON also includes raw timing repetitions and exact-score digests.

## Feature comparison

“Multiple references” means a native per-prediction multi-reference API, not a loop
written by the caller. “Bootstrap” means built-in ROUGE aggregation. Every cell in
a package row is supported by that row's source note; empirical parity is limited
to this benchmark. TorchMetrics is inspected at 1.8.2 and is not timed or installed.

| Package | Stemming | rougeLsum | Custom tokenizer | Multiple references | Bootstrap aggregation | Parity with rouge-score | Parallelism | Licence |
|---|---|---|---|---|---|---|---|---|---|
| rouge-score 0.1.2 [^google] | Porter | Yes; newlines by default | Object with `tokenize` | `score_multi`, best F1 per metric | `BootstrapAggregator` | Reference | Serial scoring; caller can distribute pairs | Apache-2.0 |
| rouge-score-rs 0.2.1 [^rs] | Compatible Porter | Yes; newlines by default | Same API, plus native `unicode-v1` and `character-v1` profiles | Same API | Compatible `BootstrapAggregator` | Exact on all tested full/subset configurations and Evaluate aggregates | Rayon batch; GIL released; custom tokenizer/sentence-splitting fallback is serial | Apache-2.0 |
| rouge-rust 0.1.12 [^rust] | No | No | No | No | No | Exact on the tested 1/2/L subset without stemming; not full API parity | Rayon batch; GIL released | MIT |
| rouge 1.0.1 (pltrdy) [^pltrdy] | No | No compatible `rougeLsum`; its `rouge-l` is summary-level | No | No | No; optional arithmetic mean | Different: tokenization, counting and F1 smoothing | Serial Python | Apache-2.0 |
| TorchMetrics ROUGEScore 1.8.2 [^torch] | Porter | Yes; NLTK sentence splitting | Callable tokenizer and normalizer | Best F1 or average | No in ROUGEScore; corpus arithmetic mean | Not bit-identical by default: tensor dtype, sentence splitting and aggregation differ | Python scoring loops; Metric supports distributed state synchronization | Apache-2.0 |

TorchMetrics' general `BootStrapper` wrapper resamples tensor inputs; it is not the
Google ROUGE bootstrap API for lists of text. Its existence should not be confused
with built-in ROUGEScore aggregation.[^torch]

## Reproduce

From the repository root, with Python 3.12, Rust and `uv` available:

```sh
uv venv .venv --python 3.12
uv pip install --python .venv/bin/python -r bindings/python/benchmarks/requirements.txt
source .venv/bin/activate
maturin develop --release -m bindings/python/Cargo.toml
python bindings/python/benchmarks/compare.py \
  --pairs 2000 --seed 1729 --repeats 3 --output benchmark-results
uvx ruff check --config bindings/python/pyproject.toml bindings/python/benchmarks
uvx ruff format --check --config bindings/python/pyproject.toml bindings/python/benchmarks
```

Dependencies are pinned in `requirements.txt`; the local package must be built from this checkout.
The script prints Markdown and writes `benchmark-results.json` and
`benchmark-results.md`. It exits with an error on an expected parity failure,
corrupt download or failed worker. JSON is checkpointed after each measurement;
only a finished run has `complete: true`. Use `--pairs 20 --repeats 1` for a quick
check, or `--workloads xsum` to select one workload.

[The Linux workflow](.github/workflows/benchmark.yml) is manual (`workflow_dispatch`)
only. It builds the local release package on `ubuntu-latest`, runs this same sample,
and uploads JSON and Markdown. It does not run on pushes or pull requests. Linux
numbers have not yet been collected.

## Sources

[data-cnn]: https://huggingface.co/datasets/abisee/cnn_dailymail/tree/96df5e686bee6baa90b8bee7c28b81fa3fa6223d/3.0.0
[data-xsum]: https://huggingface.co/datasets/EdinburghNLP/xsum/tree/7d4d486c2f8ef850b1a11aead99b894ff3dd7da9/data
[data-pubmed]: https://huggingface.co/datasets/ccdv/pubmed-summarization/tree/6b30a2cae59b11ed77cb19959bffccbbd18e1106/document

[^google]: [rouge-score 0.1.2 source distribution](https://files.pythonhosted.org/packages/e2/c5/9136736c37022a6ad27fea38f3111eb8f02fe75d067f9a985cc358653102/rouge_score-0.1.2.tar.gz): `rouge_scorer.py` (constructor, `score`, `score_multi`), `tokenizers.py`, `scoring.py` and `LICENSE`; also [versioned package documentation](https://pypi.org/project/rouge-score/0.1.2/).
[^rs]: Local [scorer](bindings/python/python/rouge_score_rs/rouge_scorer.py), [aggregation](bindings/python/python/rouge_score_rs/scoring.py), [native bindings](bindings/python/src/lib.rs), [licence](LICENSE); [repository](https://github.com/SyntaxSpirits/rougers). Measurements refer to the Git revision recorded in the JSON.
[^rust]: [0.1.12 Python API and Rayon implementation](https://github.com/kyle-mirich/rouge-rust/blob/af18a4cc7a93621ed3c3968cb8a71557092cbe95/src/python.rs), [scoring and tokenization](https://github.com/kyle-mirich/rouge-rust/blob/af18a4cc7a93621ed3c3968cb8a71557092cbe95/src/scorer.rs), [MIT licence](https://github.com/kyle-mirich/rouge-rust/blob/af18a4cc7a93621ed3c3968cb8a71557092cbe95/LICENSE).
[^pltrdy]: [1.0.1 public API](https://github.com/pltrdy/rouge/blob/657d4d2f61892fb6c5aaf8796e93ebd3ed88e857/rouge/rouge.py), [scoring implementation](https://github.com/pltrdy/rouge/blob/657d4d2f61892fb6c5aaf8796e93ebd3ed88e857/rouge/rouge_score.py), [Apache licence](https://github.com/pltrdy/rouge/blob/657d4d2f61892fb6c5aaf8796e93ebd3ed88e857/LICENSE).
[^torch]: [1.8.2 ROUGEScore API and state](https://github.com/Lightning-AI/torchmetrics/blob/v1.8.2/src/torchmetrics/text/rouge.py), [functional scoring, sentence splitting, tensors and mean](https://github.com/Lightning-AI/torchmetrics/blob/v1.8.2/src/torchmetrics/functional/text/rouge.py), [Metric synchronization](https://github.com/Lightning-AI/torchmetrics/blob/v1.8.2/src/torchmetrics/metric.py), [general bootstrap wrapper](https://github.com/Lightning-AI/torchmetrics/blob/v1.8.2/src/torchmetrics/wrappers/bootstrapping.py), [Apache licence](https://github.com/Lightning-AI/torchmetrics/blob/v1.8.2/LICENSE).
[^evaluate]: Pinned metric source: [Google wrapper](https://huggingface.co/spaces/evaluate-metric/rouge/blob/1bc20e7e9df922e5dfb597c31a4ea8467a89caff/rouge.py), [SyntaxSpirits wrapper](https://huggingface.co/spaces/SyntaxSpirits/rouge/blob/3950ae735ae7a66bbae41f73597ea84549b6d010/rouge.py); bootstrap implementation in [^google] and [^rs].
[^rss]: [Python resource API](https://docs.python.org/3.12/library/resource.html#resource.getrusage), [Linux kernel conversion to KiB](https://github.com/torvalds/linux/blob/v6.8/kernel/sys.c), [Darwin kernel resident-size reporting](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.121.3/bsd/kern/kern_resource.c).
