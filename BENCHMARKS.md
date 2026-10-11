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
Local `rouge-score-rs` 0.1.1 was built with maturin 1.15.0 in release mode.
Comparison versions: `rouge-score` 0.1.2, `rouge-rust` 0.1.12, `rouge` 1.0.1, Evaluate
0.4.6, datasets 5.1.0, NumPy 2.5.4, NLTK 3.10.3 and PyArrow 26.0.0. All installed versions
and the library Git revision are in
[the machine-readable results](bindings/python/benchmarks/results/m1-pro.json).

The complete run took 615 seconds, including downloads and worker startup.
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
| cnn_dailymail | no | 2.6061 | 0.1197 | 0.1155 | 0.0782 | 0.0393 | 0.0285 | 91.4× |
| cnn_dailymail | yes | 5.1030 | 0.1543 | 0.1520 | 0.0821 | 0.0432 | 0.0283 | 180.4× |
| xsum | no | 0.3660 | 0.0344 | 0.0329 | 0.0186 | 0.0118 | 0.0092 | 39.7× |
| xsum | yes | 1.1921 | 0.0498 | 0.0464 | 0.0264 | 0.0188 | 0.0121 | 98.9× |
| pubmed | no | 13.5276 | 0.4040 | 0.4033 | 0.2069 | 0.1087 | 0.0786 | 172.2× |
| pubmed | yes | 19.9971 | 0.5081 | 0.5328 | 0.2699 | 0.1359 | 0.0994 | 201.2× |

Across these six workloads/settings, the RS loop is **10.6–39.4×** faster than
Google; batch with 8 threads is **39.7–201.2×** faster. These are scoring-only ratios.

Like-for-like subset (`rouge1/2/L`, no stemming), seconds for 2,000 pairs:

| Workload | Google loop | RS loop | RS batch 1 | RS batch 8 | rouge-rust loop | rouge-rust batch 1 | rouge-rust batch 8 |
|---|---:|---:|---:|---:|---:|---:|---:|
| cnn_dailymail | 1.2119 | 0.0640 | 0.0636 | 0.0133 | 0.0384 | 0.0373 | 0.0072 |
| xsum | 0.2034 | 0.0212 | 0.0222 | 0.0083 | 0.0082 | 0.0083 | 0.0040 |
| pubmed | 6.2669 | 0.2077 | 0.2061 | 0.0364 | 0.1594 | 0.1604 | 0.0267 |

`rouge-rust` is faster than `rouge-score-rs` on this supported subset in this run.
It does not provide stemming or `rougeLsum`; the full four-metric timings above
are not a like-for-like comparison with it.[^rust]

Evaluate with default bootstrap aggregation, seconds for 2,000 pairs. Each pair of
columns is Google / SyntaxSpirits. Scoring is a separate instrumented pass, so it
need not be smaller than median compute time.

| Workload | Stem | Scoring s | Compute s | Load s | Load + compute s | Compute speedup |
|---|---|---:|---:|---:|---:|---:|
| cnn_dailymail | no | 2.4042 / 0.0321 | 2.6018 / 0.2563 | 1.0228 / 1.2511 | 3.6246 / 1.5074 | 10.15× |
| cnn_dailymail | yes | 6.2836 / 0.0316 | 5.2883 / 0.2513 | 0.7944 / 0.3300 | 6.0827 / 0.5813 | 21.05× |
| xsum | no | 0.3774 / 0.0099 | 0.5784 / 0.2467 | 1.1881 / 0.3558 | 1.7665 / 0.6025 | 2.34× |
| xsum | yes | 1.2019 / 0.0116 | 1.4742 / 0.2332 | 0.8325 / 0.7614 | 2.3067 / 0.9946 | 6.32× |
| pubmed | no | 12.8830 / 0.2195 | 13.1302 / 0.3299 | 0.7723 / 0.5232 | 13.9025 / 0.8531 | 39.80× |
| pubmed | yes | 19.4537 / 0.0870 | 20.3083 / 0.3198 | 0.8959 / 0.3557 | 21.2043 / 0.6754 | 63.51× |

Evaluate compute speedups are **2.34–63.51×**, much smaller than scoring-only
speedups because aggregation and input handling remain. Load timings are reported
for completeness and must not be treated as a stable package speed comparison.

Peak RSS ranges over all workloads and settings; pltrdy times are a **different
computation**, with no speedup ratio:

| Implementation | Whole-worker peak RSS MiB | pltrdy seconds, CNN/DM / XSum / PubMed |
|---|---:|---:|
| rouge-score direct | 81.4–95.1 | — |
| rouge-score-rs direct | 39.7–68.9 | — |
| rouge-rust | 41.0–61.8 | — |
| Evaluate rouge | 177.0–218.8 | — |
| Evaluate SyntaxSpirits/rouge | 166.8–216.0 | — |
| rouge/pltrdy | 45.7–104.9 | 2.8363 / 0.3450 / 21.1645 |

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
| rouge-score-rs 0.1.1 [^rs] | Compatible Porter | Yes; newlines by default | Same API | Same API | Compatible `BootstrapAggregator` | Exact on all tested full/subset configurations and Evaluate aggregates | Rayon batch; GIL released; custom tokenizer/sentence-splitting fallback is serial | Apache-2.0 |
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
