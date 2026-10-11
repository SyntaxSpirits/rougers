"""Scores must be bit-for-bit identical to the reference ``rouge-score`` package."""

import random
import sys
from pathlib import Path

import pytest
from hypothesis import given, settings
from hypothesis import strategies as st

pytest.importorskip("rouge_score")

from nltk.stem import porter
from rouge_score import rouge_scorer as reference
from rouge_score import scoring as reference_scoring
from rouge_score import tokenize as reference_tokenize

from rouge_score_rs import rouge_scorer, scoring, tokenize, tokenizers

ALL_TYPES = ["rouge1", "rouge2", "rouge3", "rouge4", "rougeL", "rougeLsum"]
DICTIONARY = Path("/usr/share/dict/words")

WORDS = [
    "the",
    "a",
    "of",
    "to",
    "and",
    "in",
    "is",
    "was",
    "it",
    "for",
    "on",
    "with",
    "as",
    "by",
    "at",
    "from",
    "that",
    "this",
    "be",
    "are",
    "run",
    "runs",
    "running",
    "ran",
    "runner",
    "happy",
    "happily",
    "happiness",
    "general",
    "generalization",
    "relate",
    "relational",
    "relation",
    "connect",
    "connected",
    "connection",
    "connecting",
    "dying",
    "lying",
    "sky",
    "skies",
    "news",
    "inning",
    "innings",
    "proceed",
    "exceeded",
    "caresses",
    "ponies",
    "ties",
    "feed",
    "agreed",
    "model",
    "models",
    "modeling",
    "modelled",
    "summary",
    "summaries",
    "summarize",
    "summarization",
    "2024",
    "3.5",
    "U.S.",
    "e-mail",
    "don't",
    "COVID-19",
    "café",
    "naïve",
    "İstanbul",
    "Kelvin",
    "Ünïcödé",
    "東京",
]
SEPARATORS = [" ", " ", " ", ", ", ". ", "\n", "\n\n", " - ", "  ", "\t", "!\n"]


def random_text(rng, max_words=40):
    parts = []
    for _ in range(rng.randint(0, max_words)):
        parts.append(rng.choice(WORDS))
        parts.append(rng.choice(SEPARATORS))
    return "".join(parts)


def reference_tokens(text, use_stemmer):
    stemmer = porter.PorterStemmer() if use_stemmer else None
    return reference_tokenize.tokenize(text, stemmer)


def as_tuples(scores):
    return {name: tuple(score) for name, score in scores.items()}


@pytest.mark.skipif(not DICTIONARY.exists(), reason="no system word list")
def test_stemmer_matches_nltk_on_dictionary():
    words = DICTIONARY.read_text(encoding="utf-8", errors="ignore").split()
    text = " ".join(words)
    expected = reference_tokens(text, use_stemmer=True)
    actual = tokenizers.DefaultTokenizer(use_stemmer=True).tokenize(text)
    assert len(actual) == len(expected)
    mismatches = [(e, a) for e, a in zip(expected, actual) if e != a]
    assert not mismatches[:20]


def test_lowercasing_matches_python_for_every_code_point():
    tokenizer = tokenizers.DefaultTokenizer()
    chunk = []
    for code_point in range(sys.maxunicode + 1):
        if 0xD800 <= code_point <= 0xDFFF:
            continue
        chunk.append(f"a{chr(code_point)}b ")
        if len(chunk) == 4096 or code_point == sys.maxunicode:
            text = "".join(chunk)
            assert tokenizer.tokenize(text) == reference_tokens(text, use_stemmer=False), text[:20]
            chunk.clear()


@pytest.mark.parametrize("use_stemmer", [False, True])
def test_random_texts(use_stemmer):
    rng = random.Random(20261011)
    ours = rouge_scorer.RougeScorer(ALL_TYPES, use_stemmer=use_stemmer)
    theirs = reference.RougeScorer(ALL_TYPES, use_stemmer=use_stemmer)
    for _ in range(3000):
        target, prediction = random_text(rng), random_text(rng)
        assert as_tuples(ours.score(target, prediction)) == as_tuples(
            theirs.score(target, prediction)
        ), (target, prediction)


@settings(max_examples=500, deadline=None)
@given(st.text(max_size=200), st.text(max_size=200), st.booleans())
def test_arbitrary_unicode(target, prediction, use_stemmer):
    ours = rouge_scorer.RougeScorer(ALL_TYPES, use_stemmer=use_stemmer)
    theirs = reference.RougeScorer(ALL_TYPES, use_stemmer=use_stemmer)
    assert as_tuples(ours.score(target, prediction)) == as_tuples(theirs.score(target, prediction))


@pytest.mark.parametrize(
    "rouge_types", [["rouge1"], ["rougeLsum"], ["rouge2", "rougeL"], ["rouge9"]]
)
def test_type_subsets(rouge_types):
    rng = random.Random(7)
    ours = rouge_scorer.RougeScorer(rouge_types, use_stemmer=True)
    theirs = reference.RougeScorer(rouge_types, use_stemmer=True)
    for _ in range(300):
        target, prediction = random_text(rng), random_text(rng)
        assert as_tuples(ours.score(target, prediction)) == as_tuples(
            theirs.score(target, prediction)
        )


def test_score_multi():
    rng = random.Random(11)
    ours = rouge_scorer.RougeScorer(ALL_TYPES, use_stemmer=True)
    theirs = reference.RougeScorer(ALL_TYPES, use_stemmer=True)
    for _ in range(300):
        targets = [random_text(rng, 15) for _ in range(rng.randint(1, 4))]
        prediction = random_text(rng, 15)
        assert as_tuples(ours.score_multi(targets, prediction)) == as_tuples(
            theirs.score_multi(targets, prediction)
        )


def test_score_batch_matches_score():
    rng = random.Random(13)
    scorer = rouge_scorer.RougeScorer(ALL_TYPES, use_stemmer=True)
    targets = [random_text(rng) for _ in range(500)]
    predictions = [random_text(rng) for _ in range(500)]
    expected = [scorer.score(t, p) for t, p in zip(targets, predictions)]
    assert scorer.score_batch(targets, predictions) == expected


class WhitespaceTokenizer:
    def tokenize(self, text):
        return text.split()


def test_custom_tokenizer():
    rng = random.Random(17)
    ours = rouge_scorer.RougeScorer(ALL_TYPES, tokenizer=WhitespaceTokenizer())
    theirs = reference.RougeScorer(ALL_TYPES, tokenizer=WhitespaceTokenizer())
    for _ in range(300):
        target, prediction = random_text(rng), random_text(rng)
        assert as_tuples(ours.score(target, prediction)) == as_tuples(
            theirs.score(target, prediction)
        )


def test_split_summaries():
    nltk = pytest.importorskip("nltk")
    try:
        nltk.data.find("tokenizers/punkt_tab")
    except LookupError:
        pytest.skip("NLTK punkt_tab data is not installed")
    rng = random.Random(19)
    ours = rouge_scorer.RougeScorer(["rougeLsum"], use_stemmer=True, split_summaries=True)
    theirs = reference.RougeScorer(["rougeLsum"], use_stemmer=True, split_summaries=True)
    for _ in range(200):
        target, prediction = random_text(rng), random_text(rng)
        assert as_tuples(ours.score(target, prediction)) == as_tuples(
            theirs.score(target, prediction)
        )


def test_bootstrap_aggregator():
    np = pytest.importorskip("numpy")
    rng = random.Random(23)
    scorer = rouge_scorer.RougeScorer(["rouge1", "rougeL"])
    ours, theirs = scoring.BootstrapAggregator(), reference_scoring.BootstrapAggregator()
    for _ in range(50):
        scores = scorer.score(random_text(rng), random_text(rng))
        ours.add_scores(scores)
        theirs.add_scores(scores)
    np.random.seed(0)
    expected = theirs.aggregate()
    np.random.seed(0)
    actual = ours.aggregate()
    for rouge_type, interval in expected.items():
        assert [tuple(s) for s in actual[rouge_type]] == [tuple(s) for s in interval]


def both(rouge_types, **kwargs):
    return (
        rouge_scorer.RougeScorer(rouge_types, **kwargs),
        reference.RougeScorer(rouge_types, **kwargs),
    )


def test_custom_tokenizer_with_hashable_tokens():
    class IntTokenizer:
        def tokenize(self, text):
            return [len(word) for word in text.split()]

    ours, theirs = both(ALL_TYPES, tokenizer=IntTokenizer())
    rng = random.Random(29)
    for _ in range(100):
        target, prediction = random_text(rng), random_text(rng)
        assert as_tuples(ours.score(target, prediction)) == as_tuples(
            theirs.score(target, prediction)
        )


def test_custom_tokenizer_call_order():
    class CountingTokenizer:
        def __init__(self):
            self.calls = []

        def tokenize(self, text):
            self.calls.append(text)
            return text.split()

    ours_tokenizer, theirs_tokenizer = CountingTokenizer(), CountingTokenizer()
    rouge_scorer.RougeScorer(["rouge1", "rougeLsum"], tokenizer=ours_tokenizer).score(
        "a b\nc", "d\ne f"
    )
    reference.RougeScorer(["rouge1", "rougeLsum"], tokenizer=theirs_tokenizer).score(
        "a b\nc", "d\ne f"
    )
    assert ours_tokenizer.calls == theirs_tokenizer.calls


def test_rouge_types_can_change_after_construction():
    ours, theirs = both(["rouge1"])
    ours.rouge_types.append("rouge2")
    theirs.rouge_types.append("rouge2")
    assert as_tuples(ours.score("a b c", "a b")) == as_tuples(theirs.score("a b c", "a b"))
    ours.rouge_types = theirs.rouge_types = ["rougeLsum"]
    assert as_tuples(ours.score("a b\nc", "a c")) == as_tuples(theirs.score("a b\nc", "a c"))


@pytest.mark.parametrize("text", ["a\ud800b", "\udcff", "x \ud83d y\nz"])
def test_lone_surrogates(text):
    ours, theirs = both(ALL_TYPES, use_stemmer=True)
    assert as_tuples(ours.score(text, "a b x z")) == as_tuples(theirs.score(text, "a b x z"))
    assert as_tuples(ours.score_multi([text], "a b")) == as_tuples(
        theirs.score_multi([text], "a b")
    )
    assert ours.score_batch([text], ["a b"]) == [ours.score(text, "a b")]
    assert tokenize.tokenize(text, None) == reference_tokenize.tokenize(text, None)


def test_score_multi_without_types_or_targets():
    ours, theirs = both([])
    assert ours.score_multi([], "a") == theirs.score_multi([], "a") == {}


def test_overridden_score_is_used_by_score_multi():
    def zero(self, target, prediction):
        return {"rouge1": scoring.Score(0.0, 0.0, 0.0)}

    ours = type("Ours", (rouge_scorer.RougeScorer,), {"score": zero})(["rouge1"])
    theirs = type("Theirs", (reference.RougeScorer,), {"score": zero})(["rouge1"])
    assert ours.score_multi(["a"], "a") == theirs.score_multi(["a"], "a")


def test_tokenize_module():
    stemmer = porter.PorterStemmer()
    rng = random.Random(31)
    for _ in range(200):
        text = random_text(rng)
        assert tokenize.tokenize(text, None) == reference_tokenize.tokenize(text, None)
        assert tokenize.tokenize(text, stemmer) == reference_tokenize.tokenize(text, stemmer)


def test_lcs_ind():
    rng = random.Random(37)
    for _ in range(500):
        ref = [rng.choice("abc") for _ in range(rng.randint(0, 8))]
        can = [rng.choice("abc") for _ in range(rng.randint(0, 8))]
        assert rouge_scorer.lcs_ind(ref, can) == reference.lcs_ind(ref, can)
