"""Fast ROUGE metrics with scores identical to ``rouge-score``."""

from rougers._rougers import __version__
from rougers.rouge_scorer import RougeScorer
from rougers.scoring import AggregateScore, BootstrapAggregator, Score

__all__ = ["AggregateScore", "BootstrapAggregator", "RougeScorer", "Score", "__version__"]
