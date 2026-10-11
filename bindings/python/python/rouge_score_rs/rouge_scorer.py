"""ROUGE scoring, compatible with ``rouge_score.rouge_scorer``."""

from rouge_score_rs import _native, _rougers, scoring, tokenizers


class RougeScorer(scoring.BaseScorer):
    """Computes ROUGE scores between a target and a prediction.

    Accepts the same arguments as ``rouge_score.rouge_scorer.RougeScorer`` and
    returns identical scores::

        scorer = RougeScorer(["rouge1", "rougeL"], use_stemmer=True)
        scores = scorer.score("The quick brown fox jumps over the lazy dog",
                              "The quick brown dog jumps on the log.")

    Valid types are ``rouge1`` to ``rouge9``, ``rougeL`` and ``rougeLsum``.
    For ``rougeLsum`` sentences are separated by newlines, or found with
    ``nltk.sent_tokenize`` when ``split_summaries`` is true.

    For text that is not in English, pass ``tokenizer=UnicodeTokenizer()`` or
    ``tokenizer=CharacterTokenizer()`` from ``rouge_score_rs.tokenizers``; they
    run natively, like the default tokenizer.

    Unlike ``rouge_score``, an invalid type raises ``ValueError`` here rather
    than on the first call to ``score``.
    """

    def __init__(self, rouge_types, use_stemmer=False, split_summaries=False, tokenizer=None):
        self.rouge_types = rouge_types
        self._split_summaries = split_summaries
        self._tokenizer = tokenizer or tokenizers.DefaultTokenizer(use_stemmer)
        self._scorer_key = None
        self._scorer = None
        self._native_scorer()

    def score(self, target, prediction):
        """Returns a dict mapping each ROUGE type to a ``Score``."""
        scorer = self._native_scorer()
        if self._tokenizes_natively():
            return _scores(_native.call(scorer.score, target, prediction))
        return _scores(self._score_with_tokenizer(scorer, target, prediction))

    def score_multi(self, targets, prediction):
        """Scores the prediction against several targets and keeps, for each
        ROUGE type, the score of the target with the highest F1."""
        if self._scores_natively():
            best = _native.call(self._native_scorer().score_multi, list(targets), prediction)
            if best is not None:
                return _scores(best)
            if self.rouge_types:
                raise ValueError("attempt to get argmax of an empty sequence")
            return {}

        all_scores = [self.score(target, prediction) for target in targets]
        result = {}
        for rouge_type in self.rouge_types:
            if not all_scores:
                raise ValueError("attempt to get argmax of an empty sequence")
            best = max(range(len(all_scores)), key=lambda i: all_scores[i][rouge_type].fmeasure)
            result[rouge_type] = all_scores[best][rouge_type]
        return result

    def score_batch(self, targets, predictions):
        """Scores pairs of targets and predictions, which must have equal length.

        Returns the same as ``[self.score(t, p) for t, p in zip(targets, predictions)]``.
        With the default tokenizer and ``split_summaries=False`` the pairs are
        scored on all CPU cores without holding the GIL; the thread count can be
        limited with the ``RAYON_NUM_THREADS`` environment variable. Otherwise,
        and in a process forked after a parallel call, pairs are scored one by one.
        """
        targets = list(targets)
        predictions = list(predictions)
        if len(targets) != len(predictions):
            raise ValueError(
                f"targets and predictions differ in length: {len(targets)} != {len(predictions)}"
            )
        if self._scores_natively():
            rows = _native.call(self._native_scorer().score_batch, targets, predictions)
            return [_scores(r) for r in rows]
        return [self.score(t, p) for t, p in zip(targets, predictions)]

    def _native_scorer(self):
        if self._has_native_tokenizer():
            key = (tuple(self.rouge_types), self._tokenizer.use_stemmer, self._tokenizer.profile)
        else:
            key = (tuple(self.rouge_types), False, tokenizers.DefaultTokenizer.profile)
        if key != self._scorer_key:
            self._scorer = _rougers.Scorer(list(key[0]), key[1], key[2])
            self._scorer_key = key
        return self._scorer

    def _has_native_tokenizer(self):
        return type(self._tokenizer) in tokenizers.NATIVE_TOKENIZERS

    def _tokenizes_natively(self):
        return self._has_native_tokenizer() and not self._split_summaries

    def _scores_natively(self):
        score = getattr(self.score, "__func__", None)
        return self._tokenizes_natively() and score is RougeScorer.score

    def _score_with_tokenizer(self, scorer, target, prediction):
        tokenize = self._tokenizer.tokenize
        if list(self.rouge_types) == ["rougeLsum"]:
            target_tokens, prediction_tokens = [], []
        else:
            target_tokens, prediction_tokens = tokenize(target), tokenize(prediction)
        if "rougeLsum" in self.rouge_types:
            target_sentences = [tokenize(s) for s in self._sentences(target)]
            prediction_sentences = [tokenize(s) for s in self._sentences(prediction)]
        else:
            target_sentences, prediction_sentences = [], []

        # Tokens may be any hashable objects; give equal tokens equal ids.
        ids = {}

        def encode(tokens):
            return [ids.setdefault(token, str(len(ids))) for token in tokens]

        return scorer.score_documents(
            encode(target_tokens),
            [encode(s) for s in target_sentences],
            encode(prediction_tokens),
            [encode(s) for s in prediction_sentences],
        )

    def _sentences(self, text):
        sentences = _sent_tokenize(text) if self._split_summaries else text.split("\n")
        return [s for s in sentences if s]


def lcs_ind(ref, can):
    """Returns the indices into ``ref`` of one longest common subsequence,
    chosen by the same backtracking order as ``rouge_score``."""
    rows, cols = len(ref), len(can)
    table = [[0] * (cols + 1) for _ in range(rows + 1)]
    for i in range(1, rows + 1):
        for j in range(1, cols + 1):
            if ref[i - 1] == can[j - 1]:
                table[i][j] = table[i - 1][j - 1] + 1
            else:
                table[i][j] = max(table[i - 1][j], table[i][j - 1])

    indices = []
    i, j = rows, cols
    while i > 0 and j > 0:
        if ref[i - 1] == can[j - 1]:
            indices.append(i - 1)
            i -= 1
            j -= 1
        elif table[i][j - 1] > table[i - 1][j]:
            j -= 1
        else:
            i -= 1
    indices.reverse()
    return indices


def _scores(rows):
    return {name: scoring.Score(p, r, f) for name, p, r, f in rows}


def _sent_tokenize(text):
    try:
        import nltk
    except ImportError as error:
        raise ImportError(
            "split_summaries=True requires NLTK: pip install 'rouge-score-rs[sentences]'"
        ) from error
    return nltk.sent_tokenize(text)
