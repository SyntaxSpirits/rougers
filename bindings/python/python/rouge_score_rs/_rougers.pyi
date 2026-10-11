from typing import Any

__version__: str

class Scorer:
    def __init__(
        self,
        rouge_types: list[str],
        use_stemmer: bool,
        profile: str = "rouge-score",
        score_class: type | None = None,
    ) -> None: ...
    def score(self, target: str, prediction: str) -> dict[str, Any]: ...
    def score_documents(
        self,
        target_tokens: list[str],
        target_sentences: list[list[str]],
        prediction_tokens: list[str],
        prediction_sentences: list[list[str]],
    ) -> dict[str, Any]: ...
    def score_multi(self, targets: list[str], prediction: str) -> dict[str, Any] | None: ...
    def score_batch(self, targets: list[str], predictions: list[str]) -> list[dict[str, Any]]: ...

def tokenize(text: str, use_stemmer: bool, profile: str = "rouge-score") -> list[str]: ...
