"""Tokenizers, compatible with ``rouge_score.tokenizers``, plus two Unicode profiles."""

import abc

from rouge_score_rs import _native, _rougers


class Tokenizer(abc.ABC):
    """Base class for custom tokenizers passed to ``RougeScorer``."""

    @abc.abstractmethod
    def tokenize(self, text):
        raise NotImplementedError("Tokenizer must override tokenize() method")


class DefaultTokenizer(Tokenizer):
    """Lowercases, splits on anything outside ``[a-z0-9]`` and optionally
    applies NLTK's Porter stemmer to tokens longer than three characters.

    This is the tokenizer of ``rouge-score``; text in other scripts produces
    no tokens.
    """

    profile = "rouge-score"

    def __init__(self, use_stemmer=False):
        self.use_stemmer = bool(use_stemmer)

    def tokenize(self, text):
        return _native.call(_rougers.tokenize, text, self.use_stemmer)


class UnicodeTokenizer(Tokenizer):
    """Splits text into words by the Unicode word boundary rules (UAX #29).

    Profile ``unicode-v1``: the text is normalized (NFKC, full case folding, NFKC
    again), and every word that contains an alphabetic character or a number
    becomes a token, so punctuation inside a word is kept (``don't``, ``3.50``).
    Suitable for languages that separate words with spaces, in any script.
    Chinese and Japanese ideographs become one token each; Thai, Lao, Khmer and
    Burmese are not segmented into words. Uses Unicode 17.0 data and no stemming.
    """

    profile = "unicode-v1"
    use_stemmer = False

    def tokenize(self, text):
        return _native.call(_rougers.tokenize, text, False, self.profile)

    def __repr__(self):
        return "UnicodeTokenizer()"


class CharacterTokenizer(Tokenizer):
    """Makes every character a token.

    Profile ``character-v1``: the text is normalized like ``unicode-v1``, and every
    grapheme cluster (a user-perceived character, UAX #29) that contains an
    alphabetic character or a number becomes a token; whitespace, punctuation and
    symbols are dropped. Character-level ROUGE is a common convention for Chinese.
    Uses Unicode 17.0 data.
    """

    profile = "character-v1"
    use_stemmer = False

    def tokenize(self, text):
        return _native.call(_rougers.tokenize, text, False, self.profile)

    def __repr__(self):
        return "CharacterTokenizer()"


NATIVE_TOKENIZERS = (DefaultTokenizer, UnicodeTokenizer, CharacterTokenizer)
