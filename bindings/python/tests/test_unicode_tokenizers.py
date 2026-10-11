"""Unicode tokenizer profiles: their tokens, and native scoring that matches rouge-score."""

import copy
import pickle
import random
import unicodedata

import pytest
from hypothesis import given, settings
from hypothesis import strategies as st

from rouge_score_rs import rouge_scorer, tokenizers

pytest.importorskip("rouge_score")

from rouge_score import rouge_scorer as reference  # noqa: E402

ALL_TYPES = ["rouge1", "rouge2", "rougeL", "rougeLsum"]
PROFILES = [tokenizers.UnicodeTokenizer, tokenizers.CharacterTokenizer]
WORDS = [
    "привет",
    "мир",
    "Мир",
    "миру",
    "новости",
    "Україна",
    "київ",
    "مرحبا",
    "بالعالم",
    "العالم",
    "नमस्ते",
    "दुनिया",
    "我",
    "爱",
    "北京",
    "天安门",
    "東京",
    "カタカナ",
    "ひらがな",
    "서울",
    "Ελληνικά",
    "ΟΔΟΣ",
    "café",
    "Café",
    "ＧＰＵ",
    "2024",
    "２０２４",
    "don't",
    "U.S.",
    "e-mail",
    "3.50",
    "the",
    "cat",
]
SEPARATORS = [" ", " ", ", ", "。", "، ", "\n", " — ", "!\n"]


def random_text(rng, max_words=30):
    return "".join(
        rng.choice(WORDS) + rng.choice(SEPARATORS) for _ in range(rng.randint(0, max_words))
    )


class PythonSideTokenizer:
    """Hands rouge-score the tokens of a native profile through its custom-tokenizer path."""

    def __init__(self, native):
        self.native = native

    def tokenize(self, text):
        return self.native.tokenize(text)


def as_tuples(scores):
    return {name: tuple(score) for name, score in scores.items()}


@pytest.mark.parametrize(
    ("text", "expected"),
    [
        ("Привет, мир!", ["привет", "мир"]),
        ("Straße STRASSE ｓｔｒａｓｓｅ", ["strasse", "strasse", "strasse"]),
        ("𝐀 ㎒", ["a", "mhz"]),
        ("مرحبا بالعالم", ["مرحبا", "بالعالم"]),
        ("नमस्ते दुनिया", ["नमस्ते", "दुनिया"]),
        ("Don't pay $3.50 for e-mail.", ["don't", "pay", "3.50", "for", "e", "mail"]),
        ("ＧＰＵ ２０２４", ["gpu", "2024"]),
        ("我爱北京", ["我", "爱", "北", "京"]),
    ],
)
def test_unicode_tokens(text, expected):
    assert tokenizers.UnicodeTokenizer().tokenize(text) == expected


@pytest.mark.parametrize(
    ("text", "expected"),
    [
        ("\u0345\u093e\u05b0", ["\u03b9\u093e\u05b0"]),
        ("\u093e", ["\u093e"]),
        ("我爱北京。", ["我", "爱", "北", "京"]),
        ("A1 b!", ["a", "1", "b"]),
        ("Café", ["c", "a", "f", "é"]),
    ],
)
def test_character_tokens(text, expected):
    assert tokenizers.CharacterTokenizer().tokenize(text) == expected


def test_default_tokenizer_still_drops_other_scripts():
    scorer = rouge_scorer.RougeScorer(["rouge1"])
    assert scorer.score("привет мир", "привет мир")["rouge1"].fmeasure == 0.0
    unicode_scorer = rouge_scorer.RougeScorer(["rouge1"], tokenizer=tokenizers.UnicodeTokenizer())
    assert unicode_scorer.score("привет мир", "привет мир")["rouge1"].fmeasure == 1.0


@pytest.mark.parametrize("profile", PROFILES)
def test_native_scoring_matches_rouge_score_with_same_tokens(profile):
    rng = random.Random(41)
    ours = rouge_scorer.RougeScorer(ALL_TYPES, tokenizer=profile())
    theirs = reference.RougeScorer(ALL_TYPES, tokenizer=PythonSideTokenizer(profile()))
    for _ in range(1000):
        target, prediction = random_text(rng), random_text(rng)
        assert as_tuples(ours.score(target, prediction)) == as_tuples(
            theirs.score(target, prediction)
        ), (target, prediction)


@pytest.mark.parametrize("profile", PROFILES)
def test_batch_multi_and_pickle_agree(profile):
    rng = random.Random(43)
    scorer = rouge_scorer.RougeScorer(ALL_TYPES, tokenizer=profile())
    theirs = reference.RougeScorer(ALL_TYPES, tokenizer=PythonSideTokenizer(profile()))
    targets = [random_text(rng) for _ in range(200)]
    predictions = [random_text(rng) for _ in range(200)]
    expected = [scorer.score(t, p) for t, p in zip(targets, predictions)]
    assert scorer.score_batch(targets, predictions) == expected
    for clone in (pickle.loads(pickle.dumps(scorer)), copy.deepcopy(scorer)):
        assert clone.score_batch(targets, predictions) == expected
    multi = [targets[i : i + 3] for i in range(0, 60, 3)]
    for refs, prediction in zip(multi, predictions):
        assert as_tuples(scorer.score_multi(refs, prediction)) == as_tuples(
            theirs.score_multi(refs, prediction)
        )


@pytest.mark.parametrize("profile", PROFILES)
def test_stemming_flag_does_not_apply(profile):
    with_flag = rouge_scorer.RougeScorer(["rouge1"], use_stemmer=True, tokenizer=profile())
    without = rouge_scorer.RougeScorer(["rouge1"], tokenizer=profile())
    assert with_flag.score("running cats", "runs cat") == without.score("running cats", "runs cat")


@settings(max_examples=300, deadline=None)
@given(st.text(max_size=120))
def test_token_invariants(text):
    for profile in PROFILES:
        tokens = profile().tokenize(text)
        lines = [token for line in text.split("\n") for token in profile().tokenize(line)]
        assert tokens == lines
        for token in tokens:
            assert token
            assert profile().tokenize(token) == [token]
            # Python's Unicode database may be older than 17.0 (unassigned there: Cn).
            assert any(ch.isalpha() or unicodedata.category(ch)[0] in "NMC" for ch in token)
