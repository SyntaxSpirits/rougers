"""Compare rouge-score-rs with rouge-score on summary-sized English texts.

pip install rouge-score nltk
python benchmarks/compare.py --pairs 5000
"""

import argparse
import os
import random
import time

import nltk
from rouge_score import rouge_scorer as reference

from rouge_score_rs import rouge_scorer

ROUGE_TYPES = ["rouge1", "rouge2", "rougeL", "rougeLsum"]


def load_pairs(count, seed):
    for resource in ("gutenberg", "punkt_tab"):
        nltk.download(resource, quiet=True, raise_on_error=True)
    sentences = [" ".join(words) for words in nltk.corpus.gutenberg.sents()]
    sentences = [s for s in sentences if 5 <= len(s.split()) <= 40]
    rng = random.Random(seed)
    pairs = []
    for _ in range(count):
        start = rng.randrange(len(sentences) - 10)
        target = "\n".join(sentences[start : start + 3])
        window = sentences[start : start + 6]
        prediction = "\n".join(rng.sample(window, 3))
        pairs.append((target, prediction))
    return pairs


def timed(fn):
    start = time.perf_counter()
    result = fn()
    return result, time.perf_counter() - start


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("--pairs", type=int, default=5000)
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--no-stemmer", action="store_true")
    args = parser.parse_args()

    use_stemmer = not args.no_stemmer
    pairs = load_pairs(args.pairs, args.seed)
    targets = [t for t, _ in pairs]
    predictions = [p for _, p in pairs]
    words = sum(len(t.split()) + len(p.split()) for t, p in pairs) / len(pairs) / 2

    theirs = reference.RougeScorer(ROUGE_TYPES, use_stemmer=use_stemmer)
    ours = rouge_scorer.RougeScorer(ROUGE_TYPES, use_stemmer=use_stemmer)

    expected, t_reference = timed(lambda: [theirs.score(t, p) for t, p in pairs])
    serial, t_serial = timed(lambda: [ours.score(t, p) for t, p in pairs])
    batch, t_batch = timed(lambda: ours.score_batch(targets, predictions))
    assert serial == expected and batch == expected, "scores differ from rouge-score"

    print(f"{len(pairs)} pairs, ~{words:.0f} words per text, stemmer={use_stemmer}")
    print(f"types: {', '.join(ROUGE_TYPES)}")
    print(f"{'implementation':<36}{'seconds':>10}{'pairs/s':>12}{'speedup':>10}")
    for name, seconds in [
        ("rouge-score", t_reference),
        ("rouge-score-rs score()", t_serial),
        (f"rouge-score-rs score_batch() x{os.cpu_count()}", t_batch),
    ]:
        print(
            f"{name:<36}{seconds:>10.3f}{len(pairs) / seconds:>12.0f}{t_reference / seconds:>9.1f}x"
        )


if __name__ == "__main__":
    main()
