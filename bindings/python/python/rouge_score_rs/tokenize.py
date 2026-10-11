"""Text tokenization, compatible with ``rouge_score.tokenize``."""

import re

from rouge_score_rs import _native, _rougers

VALID_TOKEN_RE = re.compile(r"^[a-z0-9]+$")


def tokenize(text, stemmer):
    """Lowercases ``text`` and splits it on anything outside ``[a-z0-9]``.

    ``stemmer`` is any object with a ``stem`` method, applied to tokens longer
    than three characters, or ``None``.
    """
    tokens = _native.call(_rougers.tokenize, text, False)
    if stemmer:
        tokens = [_text(stemmer.stem(x)) if len(x) > 3 else x for x in tokens]
        tokens = [x for x in tokens if VALID_TOKEN_RE.match(x)]
    return tokens


def _text(value):
    return value.decode("utf-8") if isinstance(value, bytes) else value
