import os
import pandas as pd
from typing import Literal
from viz_lib import generate, Visualizations


def generate_visualizations(
    dataset_path: str, file_type: Literal["csv", "tsv", "parquet"], output_path: str
) -> Visualizations:
    """
    Generates the dataset visualizations.

    Parameters
    ----------
    dataset_path : str
        Path to the data to generate the visualizations for.
    file_type : Literal["csv", "tsv", "parquet"]
        The type of the data file.
    output_path : str
        Path to store the visualization images.

    Returns
    -------
    Visualizations
        The visualizations map.
    """
    if not os.path.isfile(path=dataset_path):
        raise FileNotFoundError(f"File not found at path: {dataset_path}")

    match file_type:
        case "csv":
            df = pd.read_csv(filepath_or_buffer=dataset_path)
        case "tsv":
            df = pd.read_csv(filepath_or_buffer=dataset_path, sep="\t")
        case "parquet":
            df = pd.read_parquet(path=dataset_path)

    return generate(df=df, output_path=output_path)
