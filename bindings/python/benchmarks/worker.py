"""Measurements in fresh processes, with Rayon configured before imports."""

import hashlib
import importlib
import json
import resource
import statistics
import sys
import time
from unittest.mock import patch

TYPES = ["rouge1", "rouge2", "rougeL", "rougeLsum"]
METRICS = {
    "evaluate-google": ("rouge", "1bc20e7e9df922e5dfb597c31a4ea8467a89caff"),
    "evaluate-rs": ("SyntaxSpirits/rouge", "3950ae735ae7a66bbae41f73597ea84549b6d010"),
}


def digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True).encode()).hexdigest()


def normalize(scores):
    return [
        {key: [value.precision, value.recall, value.fmeasure] for key, value in row.items()}
        for row in scores
    ]


def measure(fn, repeats):
    fn()
    samples = []
    hashes = []
    for _ in range(repeats):
        start = time.perf_counter()
        result = fn()
        samples.append(time.perf_counter() - start)
        hashes.append(digest(result))
    if len(set(hashes)) != 1:
        raise AssertionError("Scores changed across repetitions")
    return result, {"seconds": statistics.median(samples), "samples_seconds": samples}


def evaluate_job(job, references, predictions):
    import evaluate

    implementation = job["implementation"]
    name, revision = METRICS[implementation]
    start = time.perf_counter()
    metric = evaluate.load(name, revision=revision, seed=job["seed"])
    load_seconds = time.perf_counter() - start

    def compute():
        return metric.compute(
            predictions=predictions,
            references=references,
            rouge_types=TYPES,
            use_stemmer=job["stemming"],
        )

    result, timing = measure(compute, job["repeats"])
    module = importlib.import_module(type(metric).__module__)
    scorer_class = module.rouge_scorer.RougeScorer
    method = "score" if implementation == "evaluate-google" else "score_batch"
    original = getattr(scorer_class, method)
    durations = []

    def instrumented(self, *args, **kwargs):
        start = time.perf_counter()
        scores = original(self, *args, **kwargs)
        durations.append(time.perf_counter() - start)
        return scores

    with patch.object(scorer_class, method, instrumented):
        instrumented_result = compute()
    if instrumented_result != result:
        raise AssertionError("Instrumentation changed Evaluate scores")
    timing.update(
        load_seconds=load_seconds,
        load_plus_compute_seconds=load_seconds + timing["seconds"],
        scoring_seconds=sum(durations),
        scoring_calls=len(durations),
        metric=name,
        revision=revision,
    )
    return result, timing


def run(job):
    references, predictions = map(list, zip(*job["pairs"]))
    implementation = job["implementation"]
    if implementation in METRICS:
        result, timing = evaluate_job(job, references, predictions)
    else:
        if implementation.startswith("google") or implementation.startswith("rs"):
            package = "rouge_score" if implementation.startswith("google") else "rouge_score_rs"
            scorer = importlib.import_module(f"{package}.rouge_scorer").RougeScorer(
                job["types"], use_stemmer=job["stemming"]
            )
            if "batch" in implementation:

                def compute():
                    return scorer.score_batch(references, predictions)
            else:

                def compute():
                    return [scorer.score(ref, pred) for ref, pred in job["pairs"]]
        elif implementation.startswith("rust"):
            import fast_rouge

            if "batch" in implementation:

                def compute():
                    return fast_rouge.score_batch(references, predictions)
            else:

                def compute():
                    return [fast_rouge.score(ref, pred) for ref, pred in job["pairs"]]
        elif implementation == "pltrdy":
            from rouge import Rouge

            scorer = Rouge()

            def compute():
                return scorer.get_scores(predictions, references)
        else:
            raise ValueError(implementation)
        compute()
        samples = []
        hashes = []
        for _ in range(job["repeats"]):
            start = time.perf_counter()
            raw = compute()
            samples.append(time.perf_counter() - start)
            result = raw if implementation == "pltrdy" else normalize(raw)
            hashes.append(digest(result))
        if len(set(hashes)) != 1:
            raise AssertionError("Scores changed across repetitions")
        timing = {"seconds": statistics.median(samples), "samples_seconds": samples}
    peak = resource.getrusage(resource.RUSAGE_SELF).ru_maxrss
    return {
        **timing,
        "peak_rss_mib": peak / (1024**2 if sys.platform == "darwin" else 1024),
        "score_sha256": digest(result),
        "scores": result,
    }
