from abc import ABC, abstractmethod
from pandas import DataFrame


class Visualizations(ABC):
    """Parent class for a visualizations object."""

    def __init__(
        self, df: DataFrame, output_path: str, dimensions: tuple[int, int] = (10, 6)
    ):
        """Constructor.

        Parameters
        ----------
        df : DataFrame
            The data frame containing the data to generate the visualizations for.
        output_path : str
            The output path to save the visualizations.
        """
        self.df = df
        self.output_path = output_path
        self.dimensions = dimensions

    @abstractmethod
    def generate(self) -> object:
        """Entrypoint for generating the visualizations."""
        pass
