from typing import TypedDict
from pandas import DataFrame
from viz_lib.missing_values import MissingValues, MissingValuesReturn


class Visualizations(TypedDict):
    missing_values: MissingValuesReturn


def generate(df: DataFrame, output_path: str) -> Visualizations:
    """Generates"""
    missing_values = MissingValues(df=df, output_path=output_path).generate()

    return_data: Visualizations = {"missing_values": missing_values}
    return return_data
