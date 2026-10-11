"""Tokenizers, compatible with ``rouge_score.tokenizers``."""

import abc

from rougers import _native, _rougers


class Tokenizer(abc.ABC):
    """Base class for custom tokenizers passed to ``RougeScorer``."""

    @abc.abstractmethod
    def tokenize(self, text):
        raise NotImplementedError("Tokenizer must override tokenize() method")


class DefaultTokenizer(Tokenizer):
    """Lowercases, splits on anything outside ``[a-z0-9]`` and optionally
    applies NLTK's Porter stemmer to tokens longer than three characters."""

    def __init__(self, use_stemmer=False):
        self.use_stemmer = bool(use_stemmer)

    def tokenize(self, text):
        return _native.call(_rougers.tokenize, text, self.use_stemmer)
