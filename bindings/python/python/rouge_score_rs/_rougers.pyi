from typing import TypeAlias

__version__: str

ScoreRow: TypeAlias = tuple[float, float, float]

class Scorer:
    def __init__(
        self, rouge_types: list[str], use_stemmer: bool, profile: str = "rouge-score"
    ) -> None: ...
    def score(self, target: str, prediction: str) -> list[ScoreRow]: ...
    def score_documents(
        self,
        target_tokens: list[str],
        target_sentences: list[list[str]],
        prediction_tokens: list[str],
        prediction_sentences: list[list[str]],
    ) -> list[ScoreRow]: ...
    def score_multi(self, targets: list[str], prediction: str) -> list[ScoreRow] | None: ...
    def score_batch(self, targets: list[str], predictions: list[str]) -> list[list[ScoreRow]]: ...

def tokenize(text: str, use_stemmer: bool, profile: str = "rouge-score") -> list[str]: ...
