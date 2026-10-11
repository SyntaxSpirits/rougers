"""Benchmark pinned summarization test sets and check ROUGE score parity."""

import argparse
import importlib.metadata
import json
import os
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

from data import SOURCES, load_workload, validate_cache
from worker import TYPES, digest, run

NAMES = {
    "google": "rouge-score loop",
    "rs-loop": "rouge-score-rs loop",
    "rs-batch": "rouge-score-rs batch",
    "google-subset": "rouge-score loop (1/2/L)",
    "rs-subset-loop": "rouge-score-rs loop (1/2/L)",
    "rs-subset-batch": "rouge-score-rs batch (1/2/L)",
    "rust-loop": "rouge-rust loop (1/2/L)",
    "rust-batch": "rouge-rust batch (1/2/L)",
    "pltrdy": "rouge/pltrdy (different computation)",
    "evaluate-google": "Evaluate rouge",
    "evaluate-rs": "Evaluate SyntaxSpirits/rouge",
}


def child(job, runtime_cache):
    env = dict(os.environ)
    env.update(
        RAYON_NUM_THREADS=str(job["threads"]),
        HF_HOME=str(runtime_cache / "huggingface"),
        HF_MODULES_CACHE=str(runtime_cache / "modules"),
        HF_EVALUATE_CACHE=str(runtime_cache / "evaluate"),
        HF_DATASETS_CACHE=str(runtime_cache / "datasets"),
        HF_HUB_DISABLE_TELEMETRY="1",
        PYTHONHASHSEED=str(job["seed"]),
    )
    completed = subprocess.run(
        [sys.executable, str(Path(__file__).resolve()), "--worker"],
        input=json.dumps(job),
        text=True,
        capture_output=True,
        env=env,
        timeout=180,
    )
    if completed.returncode:
        raise RuntimeError(f"{job['implementation']} failed:\n{completed.stderr}")
    return json.loads(completed.stdout)


def mismatch_report(expected, actual, pairs, indices):
    mismatched_pairs = 0
    fields = 0
    examples = []
    for i, (left, right) in enumerate(zip(expected, actual)):
        differences = {
            key: {"rouge-score": values, "rouge-rust": right[key]}
            for key, values in left.items()
            if values != right[key]
        }
        if differences:
            mismatched_pairs += 1
            fields += sum(a != b for key in differences for a, b in zip(left[key], right[key]))
            if len(examples) < 3:
                examples.append(
                    {
                        "test_row": indices[i],
                        "reference_excerpt": pairs[i][0][:200],
                        "prediction_excerpt": pairs[i][1][:200],
                        "differences": differences,
                    }
                )
    if len(actual) != len(expected):
        raise AssertionError("Output length mismatch")
    return {"pairs": mismatched_pairs, "fields": fields, "examples": examples}


def jobs(stemming, cores):
    yield "google", 1, TYPES, "full"
    yield "rs-loop", 1, TYPES, "full"
    for threads in dict.fromkeys([1, 2, 4, cores]):
        yield "rs-batch", threads, TYPES, "full"
    yield "evaluate-google", 1, TYPES, "evaluate"
    yield "evaluate-rs", cores, TYPES, "evaluate"
    if not stemming:
        for name, threads in [
            ("google-subset", 1),
            ("rs-subset-loop", 1),
            ("rs-subset-batch", 1),
            ("rs-subset-batch", cores),
            ("rust-loop", 1),
            ("rust-batch", 1),
            ("rust-batch", cores),
        ]:
            yield name, threads, TYPES[:3], "subset"
        yield "pltrdy", 1, ["rouge-1", "rouge-2", "rouge-l"], "different"


def environment():
    packages = {dist.metadata["Name"]: dist.version for dist in importlib.metadata.distributions()}
    cpu = platform.processor()
    if sys.platform == "darwin":
        result = subprocess.run(
            ["sysctl", "-n", "machdep.cpu.brand_string"], capture_output=True, text=True
        )
        if result.returncode == 0:
            cpu = result.stdout.strip()
    elif Path("/proc/cpuinfo").exists():
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                cpu = line.split(":", 1)[1].strip()
                break
    return {
        "cpu": cpu,
        "logical_cpus": os.cpu_count(),
        "platform": platform.platform(),
        "python": sys.version,
        "packages": dict(sorted(packages.items())),
        "git_revision": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    }


def markdown(report):
    env = report["environment"]
    lines = [
        "# Benchmark results",
        "",
        f"{report['created_at']}; {env['cpu']}; {env['logical_cpus']} logical CPUs; "
        f"{env['platform']}; Python {platform.python_version()}.",
        f"{report['pairs']} pairs per workload; seed {report['seed']}; "
        f"median of {report['repeats']} timed runs after one warmup.",
        "",
        "| Workload | Stem | Implementation | Threads | Seconds | Pairs/s | "
        "Like-for-like speedup | Peak RSS MiB |",
        "|---|---|---|---:|---:|---:|---:|---:|",
    ]
    for row in report["results"]:
        speedup = f"{row['speedup']:.2f}×" if row.get("speedup") else "—"
        lines.append(
            f"| {row['workload']} | {row['stemming']} | {NAMES[row['implementation']]} | "
            f"{row['threads']} | {row['seconds']:.4f} | {report['pairs'] / row['seconds']:.0f} | "
            f"{speedup} | {row['peak_rss_mib']:.1f} |"
        )
    lines += [
        "",
        "Full = rouge1/2/L/Lsum. Subset = rouge1/2/L, no stemming. Speedup uses the",
        "rouge-score baseline in the same group (Evaluate uses compute time).",
        "pltrdy computes different numbers and has no speedup ratio. RSS is the whole fresh",
        "worker's high-water mark, including imports, warmup and retained results, not allocation",
        "attributable to the scorer. Thread counts are RAYON_NUM_THREADS, not CPU affinity.",
        "",
        "| Workload | Stem | Evaluate metric | Load s | Compute s | Scoring s | Load + compute s |",
        "|---|---|---|---:|---:|---:|---:|",
    ]
    for row in report["results"]:
        if row["group"] == "evaluate":
            lines.append(
                f"| {row['workload']} | {row['stemming']} | {row['metric']} | "
                f"{row['load_seconds']:.4f} | {row['seconds']:.4f} | "
                f"{row['scoring_seconds']:.4f} | {row['load_plus_compute_seconds']:.4f} |"
            )
    lines += [
        "",
        "Load is one evaluate.load call (cache state can vary); compute includes input handling",
        "and default bootstrap aggregation. Scoring is one separate instrumented compute pass;",
        "load + compute is load plus median warm compute; imports and data downloads are excluded.",
        "",
        "| Workload | rouge-rust API / threads | Mismatched pairs | Mismatched P/R/F fields |",
        "|---|---|---:|---:|",
    ]
    for row in report["results"]:
        if "mismatches" in row:
            m = row["mismatches"]
            lines.append(
                f"| {row['workload']} | {row['implementation']} / {row['threads']} | "
                f"{m['pairs']} | {m['fields']} |"
            )
            for example in m["examples"]:
                lines += ["", f"Mismatch example: `{json.dumps(example, ensure_ascii=True)}`", ""]
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pairs", type=int, default=2000)
    parser.add_argument("--seed", type=int, default=1729)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--workloads", nargs="+", choices=list(SOURCES), default=list(SOURCES))
    parser.add_argument("--cache", type=Path, default=Path.home() / ".cache/rouge-score-rs-bench")
    parser.add_argument("--runtime-cache", type=Path)
    parser.add_argument("--memory-data", action="store_true", help="Do not write dataset downloads")
    parser.add_argument("--output", type=Path, default=Path("benchmark-results"))
    parser.add_argument("--worker", action="store_true", help=argparse.SUPPRESS)
    args = parser.parse_args()
    if args.worker:
        print(json.dumps(run(json.load(sys.stdin))))
        return
    if args.pairs < 1 or args.repeats < 1:
        parser.error("--pairs and --repeats must be positive")
    if importlib.metadata.version("rouge-score") != "0.1.2":
        parser.error("rouge-score==0.1.2 is required")
    cache = args.cache.expanduser().resolve()
    validate_cache(cache)
    runtime_cache = (args.runtime_cache or cache).expanduser().resolve()
    started = time.perf_counter()
    report = {
        "schema_version": 1,
        "created_at": datetime.now(timezone.utc).isoformat(),
        "environment": environment(),
        "pairs": args.pairs,
        "seed": args.seed,
        "repeats": args.repeats,
        "dataset_cache": None if args.memory_data else str(cache),
        "sources": {},
        "results": [],
    }
    for name in args.workloads:
        print(f"Loading {name}", file=sys.stderr, flush=True)
        pairs, metadata = load_workload(
            name, args.pairs, args.seed, None if args.memory_data else cache
        )
        report["sources"][name] = metadata
        for stemming in (False, True):
            baselines = {}
            for implementation, threads, types, group in jobs(stemming, os.cpu_count() or 1):
                print(
                    f"{name}: stem={stemming} {implementation} threads={threads}", file=sys.stderr
                )
                job = {
                    "pairs": pairs,
                    "implementation": implementation,
                    "threads": threads,
                    "types": types,
                    "stemming": stemming,
                    "seed": args.seed,
                    "repeats": args.repeats,
                }
                row = child(job, runtime_cache)
                scores = row.pop("scores")
                row.update(
                    workload=name,
                    implementation=implementation,
                    threads=threads,
                    types=types,
                    stemming=stemming,
                    group=group,
                )
                if group not in baselines:
                    baselines[group] = (scores, row["seconds"])
                expected, baseline_seconds = baselines[group]
                if implementation.startswith("rust"):
                    row["mismatches"] = mismatch_report(
                        expected, scores, pairs, metadata["row_indices"]
                    )
                    row["parity"] = row["mismatches"]["pairs"] == 0
                elif group == "different":
                    row["parity"] = False
                else:
                    if digest(scores) != digest(expected):
                        raise AssertionError(f"Score parity failed: {name} {implementation}")
                    row["parity"] = True
                row["speedup"] = baseline_seconds / row["seconds"] if row["parity"] else None
                report["results"].append(row)
                args.output.parent.mkdir(parents=True, exist_ok=True)
                args.output.with_suffix(".json").write_text(json.dumps(report, indent=2) + "\n")
    report["elapsed_seconds"] = time.perf_counter() - started
    report["complete"] = True
    args.output.with_suffix(".json").write_text(json.dumps(report, indent=2) + "\n")
    rendered = markdown(report)
    args.output.with_suffix(".md").write_text(rendered)
    print(rendered)


if __name__ == "__main__":
    main()
