"""Pinned test splits and deterministic extractive predictions."""

import hashlib
import io
import random
import re
import urllib.request
from pathlib import Path

SOURCES = {
    "cnn_dailymail": {
        "repo": "abisee/cnn_dailymail",
        "revision": "96df5e686bee6baa90b8bee7c28b81fa3fa6223d",
        "file": "3.0.0/test-00000-of-00001.parquet",
        "sha256": "04e322d2634a96dba76bf9a6294fbbe48e0b36abeae43f13d86ba2c3bebffe4e",
        "bytes": 30000471,
        "document": "article",
        "reference": "highlights",
        "lead": 3,
    },
    "xsum": {
        "repo": "EdinburghNLP/xsum",
        "revision": "7d4d486c2f8ef850b1a11aead99b894ff3dd7da9",
        "file": "data/test-00000-of-00001.parquet",
        "sha256": "224e9dbc6fed987759c1954603b43cb280b8d475d78893779130aa707d967ed7",
        "bytes": 16731226,
        "document": "document",
        "reference": "summary",
        "lead": 1,
    },
    "pubmed": {
        "repo": "ccdv/pubmed-summarization",
        "revision": "6b30a2cae59b11ed77cb19959bffccbbd18e1106",
        "file": "document/test-00000-of-00001.parquet",
        "sha256": "d315d092a3b2d9eeec9cc5869f5526ade3c5b4495693f0f8c7f1de1feeb8e6c3",
        "bytes": 58464984,
        "document": "article",
        "reference": "abstract",
        "lead": 5,
    },
}


def sentences(text):
    text = text.replace("<S>", "").replace("</S>", "\n")
    return [s.strip() for s in re.split(r"\n+|(?<=[.!?])\s+", text) if s.strip()]


def download(source, cache):
    url = (
        f"https://huggingface.co/datasets/{source['repo']}/resolve/"
        f"{source['revision']}/{source['file']}"
    )
    path = cache / f"{source['sha256']}.parquet" if cache else None
    if path and path.exists():
        content = path.read_bytes()
    else:
        with urllib.request.urlopen(url, timeout=120) as response:
            content = response.read(source["bytes"] + 1)
    if len(content) != source["bytes"] or hashlib.sha256(content).hexdigest() != source["sha256"]:
        raise ValueError(f"Dataset size or SHA-256 mismatch: {url}")
    if path and not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        partial = path.with_suffix(".partial")
        partial.write_bytes(content)
        partial.replace(path)
    return content, url


def load_workload(name, count, seed, cache):
    import pyarrow.parquet as pq

    source = SOURCES[name]
    content, url = download(source, cache)
    table = pq.read_table(io.BytesIO(content), columns=[source["document"], source["reference"]])
    if count > len(table):
        raise ValueError(f"Requested {count} pairs from {name}, which has {len(table)} rows")
    indices = sorted(random.Random(seed).sample(range(len(table)), count))
    rows = table.take(indices).to_pylist()
    pairs = [
        (
            "\n".join(sentences(row[source["reference"]])),
            "\n".join(sentences(row[source["document"]])[: source["lead"]]),
        )
        for row in rows
    ]
    if any(not ref or not pred for ref, pred in pairs):
        raise ValueError(f"Empty reference or prediction in selected {name} rows")
    metadata = {
        **source,
        "url": url,
        "test_rows": len(table),
        "row_indices": indices,
        "pairs": count,
        "mean_document_words": sum(len(row[source["document"]].split()) for row in rows) / count,
        "mean_reference_words": sum(len(ref.split()) for ref, _ in pairs) / count,
        "mean_prediction_words": sum(len(pred.split()) for _, pred in pairs) / count,
    }
    return pairs, metadata


def validate_cache(cache):
    root = Path(__file__).resolve().parents[3]
    if cache == root or root in cache.parents:
        raise ValueError(
            "Dataset cache must be outside the repository; use --memory-data if needed"
        )
