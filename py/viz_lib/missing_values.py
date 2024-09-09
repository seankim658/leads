from pandas import DataFrame
from typing import TypedDict
import os
import seaborn as sns
import matplotlib.pyplot as plt
from viz_lib.visualizations import Visualizations


class MissingValuesReturn(TypedDict):
    missing_values_heatmap: str


class MissingValues(Visualizations):
    """ """

    def __init__(self, df: DataFrame, output_path: str):
        """Constructor.

        Parameters
        ----------
        df : DataFrame
            The data frame containing the data to generate the visualizations for.
        output_path : str
            The output path to save the visualizations.
        """
        super().__init__(df=df, output_path=output_path)

    def generate(self) -> MissingValuesReturn:
        """Generate the missing values visualizations.

        Returns
        -------
        MissingValuesReturn
            The MissingValuesReturn TypedDict.
        """
        missing_values_heatmap_path = self._missing_values_heatmap()
        return_data: MissingValuesReturn = {
            "missing_values_heatmap": missing_values_heatmap_path
        }
        return return_data

    def _missing_values_heatmap(self) -> str:
        """Generates the missing values heatmap visualization.

        Returns
        -------
        str
            The path to the missing values heatmap chart.
        """
        missing = self.df.isnull()
        plt.figure(figsize=self.dimensions)
        sns.heatmap(missing, cbar=False, cmap="viridis")
        output_file = os.path.join(self.output_path, "missing_values_heatmap.png")
        plt.savefig(output_file)
        plt.close()
        return output_file
