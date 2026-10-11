import os

import pytest

import rouge_score_rs
from rouge_score_rs import rouge_scorer, scoring, tokenizers


def test_readme_example():
    scorer = rouge_scorer.RougeScorer(["rouge1", "rougeL"], use_stemmer=True)
    scores = scorer.score(
        "The quick brown fox jumps over the lazy dog",
        "The quick brown dog jumps on the log.",
    )
    assert scores["rouge1"] == scoring.Score(0.75, 2 / 3, 0.7058823529411765)
    assert scores["rougeL"].fmeasure == pytest.approx(0.5882352941176471)


def test_top_level_exports():
    assert rouge_score_rs.RougeScorer is rouge_scorer.RougeScorer
    assert rouge_score_rs.Score is scoring.Score
    assert rouge_score_rs.__version__


@pytest.mark.parametrize("rouge_type", ["rougen", "rouge0", "rouge10", "bleu"])
def test_rejects_invalid_types(rouge_type):
    with pytest.raises(ValueError):
        rouge_scorer.RougeScorer([rouge_type])


def test_keeps_requested_order():
    scorer = rouge_scorer.RougeScorer(["rougeL", "rouge2", "rouge1"])
    assert list(scorer.score("a b c", "a b")) == ["rougeL", "rouge2", "rouge1"]


def test_empty_texts_score_zero():
    scorer = rouge_scorer.RougeScorer(["rouge1", "rougeL", "rougeLsum"])
    for score in scorer.score("", "").values():
        assert score == scoring.Score(0, 0, 0)


def test_score_multi_requires_targets():
    with pytest.raises(ValueError):
        rouge_scorer.RougeScorer(["rouge1"]).score_multi([], "a")


def test_score_batch_checks_lengths():
    with pytest.raises(ValueError, match="differ in length"):
        rouge_scorer.RougeScorer(["rouge1"]).score_batch(["a", "b"], ["a"])


def test_default_tokenizer_instance_uses_its_stemmer_setting():
    scorer = rouge_scorer.RougeScorer(
        ["rouge1"], use_stemmer=False, tokenizer=tokenizers.DefaultTokenizer(use_stemmer=True)
    )
    assert scorer.score("running", "runs")["rouge1"].fmeasure == 1.0


def test_scorer_survives_pickle_and_deepcopy():
    import copy
    import pickle

    scorer = rouge_scorer.RougeScorer(["rouge1", "rougeLsum"], use_stemmer=True)
    expected = scorer.score("the cats sat", "a cat sat")
    for clone in (pickle.loads(pickle.dumps(scorer)), copy.deepcopy(scorer)):
        assert clone.score("the cats sat", "a cat sat") == expected


def test_scorer_works_in_worker_processes():
    from concurrent.futures import ProcessPoolExecutor

    scorer = rouge_scorer.RougeScorer(["rouge1"])
    with ProcessPoolExecutor(max_workers=2) as pool:
        results = list(pool.map(scorer.score, ["a b", "c d"], ["a", "c"]))
    assert results == [scorer.score("a b", "a"), scorer.score("c d", "c")]


def _score_batch_in_child(scorer):
    return scorer.score_batch(["a b", "c d"], ["a", "c"])


@pytest.mark.skipif(not hasattr(os, "fork"), reason="requires fork()")
@pytest.mark.filterwarnings("ignore::DeprecationWarning")
def test_score_batch_works_after_fork():
    import multiprocessing

    scorer = rouge_scorer.RougeScorer(["rouge1"])
    expected = scorer.score_batch(["a b", "c d"], ["a", "c"])
    with multiprocessing.get_context("fork").Pool(1) as pool:
        assert pool.apply_async(_score_batch_in_child, (scorer,)).get(timeout=30) == expected


# A scorer pickled by rouge-score-rs 0.1.0: RougeScorer(["rouge1", "rougeL"], use_stemmer=True).
PICKLED_BY_0_1_0 = (
    "gASV3wAAAAAAAACMG3JvdWdlX3Njb3JlX3JzLnJvdWdlX3Njb3JlcpSMC1JvdWdlU2NvcmVylJOUKYGUfZQojAtyb3VnZV90eXBl"
    "c5RdlCiMBnJvdWdlMZSMBnJvdWdlTJRljBBfc3BsaXRfc3VtbWFyaWVzlImMCl90b2tlbml6ZXKUjBlyb3VnZV9zY29yZV9ycy50"
    "b2tlbml6ZXJzlIwQRGVmYXVsdFRva2VuaXplcpSTlCmBlH2UjAt1c2Vfc3RlbW1lcpSIc2KMDV9zY29yZXJfdHlwZXOUTowHX3Nj"
    "b3JlcpROdWIu"
)


def test_unpickles_scorer_from_0_1_0():
    import base64
    import pickle

    restored = pickle.loads(base64.b64decode(PICKLED_BY_0_1_0))
    fresh = rouge_scorer.RougeScorer(["rouge1", "rougeL"], use_stemmer=True)
    assert restored.score("the cats ran", "a cat runs") == fresh.score("the cats ran", "a cat runs")


def test_native_scorer_does_not_keep_reference_cycles_alive():
    import gc
    import weakref

    from rouge_score_rs import _rougers

    class CustomScore(scoring.Score):
        pass

    native = _rougers.Scorer(["rouge1"], False, score_class=CustomScore)
    assert type(native.score("a b", "a")["rouge1"]) is CustomScore
    CustomScore.native = native
    class_ref = weakref.ref(CustomScore)
    del CustomScore, native
    gc.collect()
    assert class_ref() is None


# Scorers pickled by rouge-score-rs 0.2.0.
PICKLED_BY_0_2_0 = {
    "default_stem": (
        "gASVWgEAAAAAAACMG3JvdWdlX3Njb3JlX3JzLnJvdWdlX3Njb3JlcpSMC1JvdWdlU2NvcmVylJOUKYGUfZQojAtyb3VnZV90eXBl"
        "c5RdlCiMBnJvdWdlMZSMBnJvdWdlMpSMCXJvdWdlTHN1bZRljBBfc3BsaXRfc3VtbWFyaWVzlImMCl90b2tlbml6ZXKUjBlyb3Vn"
        "ZV9zY29yZV9ycy50b2tlbml6ZXJzlIwQRGVmYXVsdFRva2VuaXplcpSTlCmBlH2UjAt1c2Vfc3RlbW1lcpSIc2KMC19zY29yZXJf"
        "a2V5lGgHaAhoCYeUiIwLcm91Z2Utc2NvcmWUh5SMB19zY29yZXKUjBdyb3VnZV9zY29yZV9ycy5fcm91Z2Vyc5SMBlNjb3JlcpST"
        "lF2UKIwGcm91Z2UxlIwGcm91Z2UylIwJcm91Z2VMc3VtlGWIjAtyb3VnZS1zY29yZZSHlFKUdWIu"
    ),
    "unicode": (
        "gASVKwEAAAAAAACMG3JvdWdlX3Njb3JlX3JzLnJvdWdlX3Njb3JlcpSMC1JvdWdlU2NvcmVylJOUKYGUfZQojAtyb3VnZV90eXBl"
        "c5RdlCiMBnJvdWdlMZSMBnJvdWdlTJRljBBfc3BsaXRfc3VtbWFyaWVzlImMCl90b2tlbml6ZXKUjBlyb3VnZV9zY29yZV9ycy50"
        "b2tlbml6ZXJzlIwQVW5pY29kZVRva2VuaXplcpSTlCmBlIwLX3Njb3Jlcl9rZXmUaAdoCIaUiYwKdW5pY29kZS12MZSHlIwHX3Nj"
        "b3JlcpSMF3JvdWdlX3Njb3JlX3JzLl9yb3VnZXJzlIwGU2NvcmVylJOUXZQojAZyb3VnZTGUjAZyb3VnZUyUZYmMCnVuaWNvZGUt"
        "djGUh5RSlHViLg=="
    ),
}


@pytest.mark.parametrize("name", sorted(PICKLED_BY_0_2_0))
def test_unpickles_scorers_from_0_2_0(name):
    import base64
    import pickle

    restored = pickle.loads(base64.b64decode(PICKLED_BY_0_2_0[name]))
    if name == "unicode":
        fresh = rouge_scorer.RougeScorer(
            ["rouge1", "rougeL"], tokenizer=tokenizers.UnicodeTokenizer()
        )
        pair = ("привет мир", "привет")
    else:
        fresh = rouge_scorer.RougeScorer(["rouge1", "rouge2", "rougeLsum"], use_stemmer=True)
        pair = ("the cats ran\nfast", "a cat runs fast")
    assert restored.score(*pair) == fresh.score(*pair)
    assert restored.score_batch([pair[0]], [pair[1]]) == [fresh.score(*pair)]
