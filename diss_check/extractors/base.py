from abc import ABC, abstractmethod
from pathlib import Path
from diss_check.document import ExtractionContext


class BaseExtractor(ABC):
    name: str

    @abstractmethod
    def extract(self, source: Path, ctx: ExtractionContext) -> None:
        ...
