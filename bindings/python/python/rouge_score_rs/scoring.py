"""Score types and bootstrap aggregation, compatible with ``rouge_score.scoring``."""

import abc
import collections


class Score(collections.namedtuple("Score", ["precision", "recall", "fmeasure"])):
    """Precision, recall and F1 for one ROUGE variant."""


class AggregateScore(collections.namedtuple("AggregateScore", ["low", "mid", "high"])):
    """Bootstrap confidence interval: ``mid`` is the mean, ``low`` and ``high`` the bounds."""


class BaseScorer(metaclass=abc.ABCMeta):
    @abc.abstractmethod
    def score(self, target: str, prediction: str) -> dict[str, Score]:
        """Returns a mapping from score type to score."""


class BootstrapAggregator:
    """Aggregates scores into bootstrap confidence intervals of the mean.

    Uses NumPy's global random state exactly as ``rouge_score`` does, so a
    seeded run gives the same intervals with either package.
    """

    def __init__(self, confidence_interval=0.95, n_samples=1000):
        if confidence_interval < 0 or confidence_interval > 1:
            raise ValueError("confidence_interval must be in range [0, 1]")
        if n_samples <= 0:
            raise ValueError("n_samples must be positive")
        self._n_samples = n_samples
        self._confidence_interval = confidence_interval
        self._scores = collections.defaultdict(list)

    def add_scores(self, scores):
        for score_type, score in scores.items():
            self._scores[score_type].append(score)

    def aggregate(self):
        np = _numpy()
        result = {}
        for score_type, scores in self._scores.items():
            score_matrix = np.vstack(tuple(scores))
            percentiles = self._bootstrap_resample(np, score_matrix)
            intervals = tuple(scores[0].__class__(*percentiles[j, :]) for j in range(3))
            result[score_type] = AggregateScore(
                low=intervals[0], mid=intervals[1], high=intervals[2]
            )
        return result

    def _bootstrap_resample(self, np, matrix):
        sample_mean = np.zeros((self._n_samples, matrix.shape[1]))
        for i in range(self._n_samples):
            sample_idx = np.random.choice(np.arange(matrix.shape[0]), size=matrix.shape[0])
            sample = matrix[sample_idx, :]
            sample_mean[i, :] = np.mean(sample, axis=0)
        percentile_delta = (1 - self._confidence_interval) / 2
        q = 100 * np.array([percentile_delta, 0.5, 1 - percentile_delta])
        return np.percentile(sample_mean, q, axis=0)


def fmeasure(precision, recall):
    if precision + recall > 0:
        return 2 * precision * recall / (precision + recall)
    return 0.0


def _numpy():
    try:
        import numpy
    except ImportError as error:
        raise ImportError(
            "BootstrapAggregator requires NumPy: pip install 'rouge-score-rs[aggregate]'"
        ) from error
    return numpy
