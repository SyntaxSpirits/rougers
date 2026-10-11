# Benchmark results

2026-10-11T04:04:40.115976+00:00; Apple M1 Pro; 8 logical CPUs; macOS-14.5-arm64-arm-64bit; Python 3.12.11.
2000 pairs per workload; seed 1729; median of 3 timed runs after one warmup.

| Workload | Stem | Implementation | Threads | Seconds | Pairs/s | Like-for-like speedup | Peak RSS MiB |
|---|---|---|---:|---:|---:|---:|---:|
| cnn_dailymail | False | rouge-score loop | 1 | 2.5209 | 793 | 1.00× | 83.2 |
| cnn_dailymail | False | rouge-score-rs loop | 1 | 0.0428 | 46726 | 58.89× | 46.8 |
| cnn_dailymail | False | rouge-score-rs batch | 1 | 0.0442 | 45224 | 57.00× | 51.8 |
| cnn_dailymail | False | rouge-score-rs batch | 2 | 0.0257 | 77873 | 98.15× | 48.7 |
| cnn_dailymail | False | rouge-score-rs batch | 4 | 0.0245 | 81533 | 102.77× | 50.7 |
| cnn_dailymail | False | rouge-score-rs batch | 8 | 0.0105 | 189595 | 238.97× | 49.3 |
| cnn_dailymail | False | Evaluate rouge | 1 | 2.6896 | 744 | 1.00× | 180.9 |
| cnn_dailymail | False | Evaluate SyntaxSpirits/rouge | 8 | 0.2335 | 8564 | 11.52× | 164.6 |
| cnn_dailymail | False | rouge-score loop (1/2/L) | 1 | 1.3252 | 1509 | 1.00× | 86.2 |
| cnn_dailymail | False | rouge-score-rs loop (1/2/L) | 1 | 0.0172 | 116453 | 77.16× | 42.7 |
| cnn_dailymail | False | rouge-score-rs batch (1/2/L) | 1 | 0.0160 | 125323 | 83.04× | 45.5 |
| cnn_dailymail | False | rouge-score-rs batch (1/2/L) | 8 | 0.0047 | 422145 | 279.71× | 45.2 |
| cnn_dailymail | False | rouge-rust loop (1/2/L) | 1 | 0.0384 | 52068 | 34.50× | 43.0 |
| cnn_dailymail | False | rouge-rust batch (1/2/L) | 1 | 0.0389 | 51389 | 34.05× | 45.8 |
| cnn_dailymail | False | rouge-rust batch (1/2/L) | 8 | 0.0074 | 268719 | 178.05× | 46.4 |
| cnn_dailymail | False | rouge/pltrdy (different computation) | 1 | 2.8526 | 701 | — | 53.8 |
| cnn_dailymail | True | rouge-score loop | 1 | 5.2720 | 379 | 1.00× | 74.0 |
| cnn_dailymail | True | rouge-score-rs loop | 1 | 0.0929 | 21523 | 56.73× | 45.3 |
| cnn_dailymail | True | rouge-score-rs batch | 1 | 0.0851 | 23507 | 61.97× | 49.1 |
| cnn_dailymail | True | rouge-score-rs batch | 2 | 0.0462 | 43308 | 114.16× | 48.5 |
| cnn_dailymail | True | rouge-score-rs batch | 4 | 0.0251 | 79639 | 209.93× | 48.3 |
| cnn_dailymail | True | rouge-score-rs batch | 8 | 0.0175 | 113977 | 300.44× | 48.8 |
| cnn_dailymail | True | Evaluate rouge | 1 | 5.3930 | 371 | 1.00× | 197.8 |
| cnn_dailymail | True | Evaluate SyntaxSpirits/rouge | 8 | 0.3006 | 6654 | 17.94× | 179.7 |
| xsum | False | rouge-score loop | 1 | 0.3758 | 5322 | 1.00× | 82.3 |
| xsum | False | rouge-score-rs loop | 1 | 0.0126 | 158205 | 29.73× | 41.1 |
| xsum | False | rouge-score-rs batch | 1 | 0.0103 | 193793 | 36.42× | 43.5 |
| xsum | False | rouge-score-rs batch | 2 | 0.0066 | 305039 | 57.32× | 44.6 |
| xsum | False | rouge-score-rs batch | 4 | 0.0045 | 448766 | 84.33× | 44.1 |
| xsum | False | rouge-score-rs batch | 8 | 0.0045 | 439786 | 82.64× | 44.4 |
| xsum | False | Evaluate rouge | 1 | 0.7498 | 2668 | 1.00× | 184.3 |
| xsum | False | Evaluate SyntaxSpirits/rouge | 8 | 0.2258 | 8856 | 3.32× | 171.1 |
| xsum | False | rouge-score loop (1/2/L) | 1 | 0.2082 | 9607 | 1.00× | 86.8 |
| xsum | False | rouge-score-rs loop (1/2/L) | 1 | 0.0075 | 266096 | 27.70× | 41.7 |
| xsum | False | rouge-score-rs batch (1/2/L) | 1 | 0.0066 | 304082 | 31.65× | 41.7 |
| xsum | False | rouge-score-rs batch (1/2/L) | 8 | 0.0029 | 678100 | 70.58× | 42.8 |
| xsum | False | rouge-rust loop (1/2/L) | 1 | 0.0098 | 204351 | 21.27× | 41.7 |
| xsum | False | rouge-rust batch (1/2/L) | 1 | 0.0085 | 234922 | 24.45× | 42.6 |
| xsum | False | rouge-rust batch (1/2/L) | 8 | 0.0042 | 475125 | 49.46× | 42.9 |
| xsum | False | rouge/pltrdy (different computation) | 1 | 0.3411 | 5864 | — | 46.6 |
| xsum | True | rouge-score loop | 1 | 1.2187 | 1641 | 1.00× | 83.4 |
| xsum | True | rouge-score-rs loop | 1 | 0.0247 | 81036 | 49.38× | 38.5 |
| xsum | True | rouge-score-rs batch | 1 | 0.0237 | 84239 | 51.33× | 40.8 |
| xsum | True | rouge-score-rs batch | 2 | 0.0132 | 151846 | 92.53× | 42.6 |
| xsum | True | rouge-score-rs batch | 4 | 0.0079 | 251628 | 153.33× | 43.6 |
| xsum | True | rouge-score-rs batch | 8 | 0.0062 | 321914 | 196.16× | 45.2 |
| xsum | True | Evaluate rouge | 1 | 1.4617 | 1368 | 1.00× | 189.3 |
| xsum | True | Evaluate SyntaxSpirits/rouge | 8 | 0.2233 | 8956 | 6.55× | 173.6 |
| pubmed | False | rouge-score loop | 1 | 13.6516 | 147 | 1.00× | 89.7 |
| pubmed | False | rouge-score-rs loop | 1 | 0.1693 | 11810 | 80.62× | 55.1 |
| pubmed | False | rouge-score-rs batch | 1 | 0.1682 | 11893 | 81.18× | 60.3 |
| pubmed | False | rouge-score-rs batch | 2 | 0.0867 | 23065 | 157.44× | 67.6 |
| pubmed | False | rouge-score-rs batch | 4 | 0.0464 | 43120 | 294.33× | 66.8 |
| pubmed | False | rouge-score-rs batch | 8 | 0.0438 | 45620 | 311.39× | 57.2 |
| pubmed | False | Evaluate rouge | 1 | 21.5716 | 93 | 1.00× | 180.1 |
| pubmed | False | Evaluate SyntaxSpirits/rouge | 8 | 0.3191 | 6269 | 67.61× | 179.5 |
| pubmed | False | rouge-score loop (1/2/L) | 1 | 6.9406 | 288 | 1.00× | 96.1 |
| pubmed | False | rouge-score-rs loop (1/2/L) | 1 | 0.0355 | 56335 | 195.50× | 50.2 |
| pubmed | False | rouge-score-rs batch (1/2/L) | 1 | 0.0346 | 57826 | 200.67× | 59.1 |
| pubmed | False | rouge-score-rs batch (1/2/L) | 8 | 0.0081 | 247374 | 858.46× | 58.0 |
| pubmed | False | rouge-rust loop (1/2/L) | 1 | 0.1619 | 12354 | 42.87× | 48.6 |
| pubmed | False | rouge-rust batch (1/2/L) | 1 | 0.1618 | 12362 | 42.90× | 60.7 |
| pubmed | False | rouge-rust batch (1/2/L) | 8 | 0.0269 | 74219 | 257.56× | 54.7 |
| pubmed | False | rouge/pltrdy (different computation) | 1 | 25.4146 | 79 | — | 104.0 |
| pubmed | True | rouge-score loop | 1 | 19.6462 | 102 | 1.00× | 93.9 |
| pubmed | True | rouge-score-rs loop | 1 | 0.2804 | 7134 | 70.08× | 54.2 |
| pubmed | True | rouge-score-rs batch | 1 | 0.3042 | 6575 | 64.58× | 69.1 |
| pubmed | True | rouge-score-rs batch | 2 | 0.1479 | 13523 | 132.84× | 65.7 |
| pubmed | True | rouge-score-rs batch | 4 | 0.0993 | 20132 | 197.76× | 60.4 |
| pubmed | True | rouge-score-rs batch | 8 | 0.0557 | 35888 | 352.53× | 67.9 |
| pubmed | True | Evaluate rouge | 1 | 20.6311 | 97 | 1.00× | 199.1 |
| pubmed | True | Evaluate SyntaxSpirits/rouge | 8 | 0.2868 | 6974 | 71.94× | 207.6 |

Full = rouge1/2/L/Lsum. Subset = rouge1/2/L, no stemming. Speedup uses the
rouge-score baseline in the same group (Evaluate uses compute time).
pltrdy computes different numbers and has no speedup ratio. RSS is the whole fresh
worker's high-water mark, including imports, warmup and retained results, not allocation
attributable to the scorer. Thread counts are RAYON_NUM_THREADS, not CPU affinity.

| Workload | Stem | Evaluate metric | Load s | Compute s | Scoring s | Load + compute s |
|---|---|---|---:|---:|---:|---:|
| cnn_dailymail | False | rouge | 0.9023 | 2.6896 | 2.8886 | 3.5918 |
| cnn_dailymail | False | SyntaxSpirits/rouge | 0.6752 | 0.2335 | 0.0098 | 0.9088 |
| cnn_dailymail | True | rouge | 0.8012 | 5.3930 | 5.1605 | 6.1941 |
| cnn_dailymail | True | SyntaxSpirits/rouge | 0.3579 | 0.3006 | 0.0489 | 0.6585 |
| xsum | False | rouge | 1.0971 | 0.7498 | 0.3841 | 1.8468 |
| xsum | False | SyntaxSpirits/rouge | 0.2950 | 0.2258 | 0.0039 | 0.5208 |
| xsum | True | rouge | 0.6947 | 1.4617 | 1.2187 | 2.1564 |
| xsum | True | SyntaxSpirits/rouge | 0.3025 | 0.2233 | 0.0065 | 0.5258 |
| pubmed | False | rouge | 0.9735 | 21.5716 | 13.2295 | 22.5452 |
| pubmed | False | SyntaxSpirits/rouge | 0.4257 | 0.3191 | 0.0437 | 0.7448 |
| pubmed | True | rouge | 0.8930 | 20.6311 | 20.0144 | 21.5241 |
| pubmed | True | SyntaxSpirits/rouge | 0.3666 | 0.2868 | 0.0579 | 0.6534 |

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
