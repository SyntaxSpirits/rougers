# Benchmark results

2026-10-11T02:59:46.035311+00:00; Apple M1 Pro; 8 logical CPUs; macOS-14.5-arm64-arm-64bit; Python 3.12.11.
2000 pairs per workload; seed 1729; median of 3 timed runs after one warmup.

| Workload | Stem | Implementation | Threads | Seconds | Pairs/s | Like-for-like speedup | Peak RSS MiB |
|---|---|---|---:|---:|---:|---:|---:|
| cnn_dailymail | False | rouge-score loop | 1 | 2.6061 | 767 | 1.00× | 85.8 |
| cnn_dailymail | False | rouge-score-rs loop | 1 | 0.1197 | 16715 | 21.78× | 47.2 |
| cnn_dailymail | False | rouge-score-rs batch | 1 | 0.1155 | 17315 | 22.56× | 51.3 |
| cnn_dailymail | False | rouge-score-rs batch | 2 | 0.0782 | 25578 | 33.33× | 52.6 |
| cnn_dailymail | False | rouge-score-rs batch | 4 | 0.0393 | 50849 | 66.26× | 49.5 |
| cnn_dailymail | False | rouge-score-rs batch | 8 | 0.0285 | 70168 | 91.43× | 51.8 |
| cnn_dailymail | False | Evaluate rouge | 1 | 2.6018 | 769 | 1.00× | 192.0 |
| cnn_dailymail | False | Evaluate SyntaxSpirits/rouge | 8 | 0.2563 | 7805 | 10.15× | 187.9 |
| cnn_dailymail | False | rouge-score loop (1/2/L) | 1 | 1.2119 | 1650 | 1.00× | 88.8 |
| cnn_dailymail | False | rouge-score-rs loop (1/2/L) | 1 | 0.0640 | 31229 | 18.92× | 43.4 |
| cnn_dailymail | False | rouge-score-rs batch (1/2/L) | 1 | 0.0636 | 31460 | 19.06× | 51.1 |
| cnn_dailymail | False | rouge-score-rs batch (1/2/L) | 8 | 0.0133 | 150690 | 91.31× | 47.3 |
| cnn_dailymail | False | rouge-rust loop (1/2/L) | 1 | 0.0384 | 52146 | 31.60× | 43.3 |
| cnn_dailymail | False | rouge-rust batch (1/2/L) | 1 | 0.0373 | 53557 | 32.45× | 46.6 |
| cnn_dailymail | False | rouge-rust batch (1/2/L) | 8 | 0.0072 | 279624 | 169.43× | 46.9 |
| cnn_dailymail | False | rouge/pltrdy (different computation) | 1 | 2.8363 | 705 | — | 59.9 |
| cnn_dailymail | True | rouge-score loop | 1 | 5.1030 | 392 | 1.00× | 91.6 |
| cnn_dailymail | True | rouge-score-rs loop | 1 | 0.1543 | 12963 | 33.08× | 46.0 |
| cnn_dailymail | True | rouge-score-rs batch | 1 | 0.1520 | 13161 | 33.58× | 50.6 |
| cnn_dailymail | True | rouge-score-rs batch | 2 | 0.0821 | 24370 | 62.18× | 51.1 |
| cnn_dailymail | True | rouge-score-rs batch | 4 | 0.0432 | 46342 | 118.24× | 49.9 |
| cnn_dailymail | True | rouge-score-rs batch | 8 | 0.0283 | 70689 | 180.36× | 52.7 |
| cnn_dailymail | True | Evaluate rouge | 1 | 5.2883 | 378 | 1.00× | 202.5 |
| cnn_dailymail | True | Evaluate SyntaxSpirits/rouge | 8 | 0.2513 | 7960 | 21.05× | 186.2 |
| xsum | False | rouge-score loop | 1 | 0.3660 | 5464 | 1.00× | 87.3 |
| xsum | False | rouge-score-rs loop | 1 | 0.0344 | 58162 | 10.64× | 43.4 |
| xsum | False | rouge-score-rs batch | 1 | 0.0329 | 60829 | 11.13× | 44.9 |
| xsum | False | rouge-score-rs batch | 2 | 0.0186 | 107254 | 19.63× | 44.7 |
| xsum | False | rouge-score-rs batch | 4 | 0.0118 | 169358 | 31.00× | 47.6 |
| xsum | False | rouge-score-rs batch | 8 | 0.0092 | 217110 | 39.74× | 44.7 |
| xsum | False | Evaluate rouge | 1 | 0.5784 | 3458 | 1.00× | 191.1 |
| xsum | False | Evaluate SyntaxSpirits/rouge | 8 | 0.2467 | 8107 | 2.34× | 174.6 |
| xsum | False | rouge-score loop (1/2/L) | 1 | 0.2034 | 9833 | 1.00× | 81.4 |
| xsum | False | rouge-score-rs loop (1/2/L) | 1 | 0.0212 | 94138 | 9.57× | 39.7 |
| xsum | False | rouge-score-rs batch (1/2/L) | 1 | 0.0222 | 90189 | 9.17× | 42.7 |
| xsum | False | rouge-score-rs batch (1/2/L) | 8 | 0.0083 | 240769 | 24.49× | 43.1 |
| xsum | False | rouge-rust loop (1/2/L) | 1 | 0.0082 | 242489 | 24.66× | 41.0 |
| xsum | False | rouge-rust batch (1/2/L) | 1 | 0.0083 | 241665 | 24.58× | 41.8 |
| xsum | False | rouge-rust batch (1/2/L) | 8 | 0.0040 | 497667 | 50.61× | 41.9 |
| xsum | False | rouge/pltrdy (different computation) | 1 | 0.3450 | 5797 | — | 45.7 |
| xsum | True | rouge-score loop | 1 | 1.1921 | 1678 | 1.00× | 87.1 |
| xsum | True | rouge-score-rs loop | 1 | 0.0498 | 40121 | 23.91× | 43.5 |
| xsum | True | rouge-score-rs batch | 1 | 0.0464 | 43089 | 25.68× | 46.8 |
| xsum | True | rouge-score-rs batch | 2 | 0.0264 | 75758 | 45.15× | 46.3 |
| xsum | True | rouge-score-rs batch | 4 | 0.0188 | 106184 | 63.29× | 45.4 |
| xsum | True | rouge-score-rs batch | 8 | 0.0121 | 165864 | 98.86× | 46.7 |
| xsum | True | Evaluate rouge | 1 | 1.4742 | 1357 | 1.00× | 177.0 |
| xsum | True | Evaluate SyntaxSpirits/rouge | 8 | 0.2332 | 8578 | 6.32× | 166.8 |
| pubmed | False | rouge-score loop | 1 | 13.5276 | 148 | 1.00× | 95.1 |
| pubmed | False | rouge-score-rs loop | 1 | 0.4040 | 4951 | 33.49× | 55.4 |
| pubmed | False | rouge-score-rs batch | 1 | 0.4033 | 4959 | 33.54× | 68.9 |
| pubmed | False | rouge-score-rs batch | 2 | 0.2069 | 9666 | 65.38× | 64.4 |
| pubmed | False | rouge-score-rs batch | 4 | 0.1087 | 18401 | 124.46× | 63.5 |
| pubmed | False | rouge-score-rs batch | 8 | 0.0786 | 25452 | 172.15× | 67.7 |
| pubmed | False | Evaluate rouge | 1 | 13.1302 | 152 | 1.00× | 218.5 |
| pubmed | False | Evaluate SyntaxSpirits/rouge | 8 | 0.3299 | 6063 | 39.80× | 197.2 |
| pubmed | False | rouge-score loop (1/2/L) | 1 | 6.2669 | 319 | 1.00× | 89.2 |
| pubmed | False | rouge-score-rs loop (1/2/L) | 1 | 0.2077 | 9629 | 30.17× | 55.3 |
| pubmed | False | rouge-score-rs batch (1/2/L) | 1 | 0.2061 | 9706 | 30.41× | 60.3 |
| pubmed | False | rouge-score-rs batch (1/2/L) | 8 | 0.0364 | 54987 | 172.30× | 67.4 |
| pubmed | False | rouge-rust loop (1/2/L) | 1 | 0.1594 | 12546 | 39.31× | 50.2 |
| pubmed | False | rouge-rust batch (1/2/L) | 1 | 0.1604 | 12466 | 39.06× | 61.8 |
| pubmed | False | rouge-rust batch (1/2/L) | 8 | 0.0267 | 74950 | 234.85× | 61.5 |
| pubmed | False | rouge/pltrdy (different computation) | 1 | 21.1645 | 94 | — | 104.9 |
| pubmed | True | rouge-score loop | 1 | 19.9971 | 100 | 1.00× | 89.8 |
| pubmed | True | rouge-score-rs loop | 1 | 0.5081 | 3936 | 39.35× | 53.1 |
| pubmed | True | rouge-score-rs batch | 1 | 0.5328 | 3754 | 37.53× | 65.8 |
| pubmed | True | rouge-score-rs batch | 2 | 0.2699 | 7411 | 74.10× | 67.1 |
| pubmed | True | rouge-score-rs batch | 4 | 0.1359 | 14721 | 147.19× | 66.0 |
| pubmed | True | rouge-score-rs batch | 8 | 0.0994 | 20122 | 201.19× | 64.4 |
| pubmed | True | Evaluate rouge | 1 | 20.3083 | 98 | 1.00× | 218.8 |
| pubmed | True | Evaluate SyntaxSpirits/rouge | 8 | 0.3198 | 6254 | 63.51× | 216.0 |

Full = rouge1/2/L/Lsum. Subset = rouge1/2/L, no stemming. Speedup uses the
rouge-score baseline in the same group (Evaluate uses compute time).
pltrdy computes different numbers and has no speedup ratio. RSS is the whole fresh
worker's high-water mark, including imports, warmup and retained results, not allocation
attributable to the scorer. Thread counts are RAYON_NUM_THREADS, not CPU affinity.

| Workload | Stem | Evaluate metric | Load s | Compute s | Scoring s | Load + compute s |
|---|---|---|---:|---:|---:|---:|
| cnn_dailymail | False | rouge | 1.0228 | 2.6018 | 2.4042 | 3.6246 |
| cnn_dailymail | False | SyntaxSpirits/rouge | 1.2511 | 0.2563 | 0.0321 | 1.5074 |
| cnn_dailymail | True | rouge | 0.7944 | 5.2883 | 6.2836 | 6.0827 |
| cnn_dailymail | True | SyntaxSpirits/rouge | 0.3300 | 0.2513 | 0.0316 | 0.5813 |
| xsum | False | rouge | 1.1881 | 0.5784 | 0.3774 | 1.7665 |
| xsum | False | SyntaxSpirits/rouge | 0.3558 | 0.2467 | 0.0099 | 0.6025 |
| xsum | True | rouge | 0.8325 | 1.4742 | 1.2019 | 2.3067 |
| xsum | True | SyntaxSpirits/rouge | 0.7614 | 0.2332 | 0.0116 | 0.9946 |
| pubmed | False | rouge | 0.7723 | 13.1302 | 12.8830 | 13.9025 |
| pubmed | False | SyntaxSpirits/rouge | 0.5232 | 0.3299 | 0.2195 | 0.8531 |
| pubmed | True | rouge | 0.8959 | 20.3083 | 19.4537 | 21.2043 |
| pubmed | True | SyntaxSpirits/rouge | 0.3557 | 0.3198 | 0.0870 | 0.6754 |

Load is one evaluate.load call (cache state can vary); compute includes input handling
and default bootstrap aggregation. Scoring is one separate instrumented compute pass;
load + compute is load plus median warm compute; imports and data downloads are excluded.

| Workload | rouge-rust API / threads | Mismatched pairs | Mismatched P/R/F fields |
|---|---|---:|---:|
| cnn_dailymail | rust-loop / 1 | 0 | 0 |
| cnn_dailymail | rust-batch / 1 | 0 | 0 |
| cnn_dailymail | rust-batch / 8 | 0 | 0 |
| xsum | rust-loop / 1 | 0 | 0 |
| xsum | rust-batch / 1 | 0 | 0 |
| xsum | rust-batch / 8 | 0 | 0 |
| pubmed | rust-loop / 1 | 0 | 0 |
| pubmed | rust-batch / 1 | 0 | 0 |
| pubmed | rust-batch / 8 | 0 | 0 |
