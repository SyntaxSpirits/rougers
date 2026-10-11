"""Fast ROUGE metrics with scores identical to ``rouge-score``."""

from rouge_score_rs._rougers import __version__
from rouge_score_rs.rouge_scorer import RougeScorer
from rouge_score_rs.scoring import AggregateScore, BootstrapAggregator, Score

__all__ = ["AggregateScore", "BootstrapAggregator", "RougeScorer", "Score", "__version__"]
