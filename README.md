# LEADS

LEADS is a **L**azy **E**xploratory **A**nalysis **D**ata **S**ummarizer.

Writing the same boilerplate exploratory analysis code in a Jupyter notebook or Excel spreadsheet for each new dataset can be tedious. This tool automates the generation of a consistent, comprehensive, and human readable exploratory analysis report that allows you to immediately become familiar with a dataset. The generated PDF report contains the below features.

Currently supports `.csv`, `.tsv`, and `.parquet` files for inputs and `.pdf` files for report formats (eventually will work on additional report formats such as markdown).

## Current Features

- Report features:
    - Title page.
    - Table of contents.
    - Page numbers.
    - Run metadata (in progress).
    - Glossary of statistical terms (in progress).
- Report analysis sections:
  - Data type analysis:
    - Identification of feature data types.
    - Unique value counts for categorical variables (in progress).
  - Basic dataset information and descriptive statistics (in progress):
    - Number of rows and columns.
    - Column names and data types.
    - Min, max, mean, median, standard deviation.
    - Quartiles and interquartile ranges.
    - Skewness and kurtosis.
  - Missing value analysis (in progress):
    - Count and percentage of missing values per column.
    - Visualization of missing value patterns.
  - Distribution analysis (in progress):
    - Normality tests (Shapiro-Wilk, Anderson-Darling).
    - Q-Q plots.
  - Outlier detection (in progress):
    - Z-score method.
    - IQR method.
    - Local outlier factor (LOF).
    - Visualization of outliers.
  - Visualizations (in progress):
    - Histograms.
    - Box plots.
    - Scatter plots.
    - Correlation heatmaps.
    - Pair plots for multivariate data.
  - Multicollinearity checks (in progress):
    - Correlation matrix.
    - Variance inflation factor (VIF).
  - Pairwise data exploration (in progress):
    - Scatter plot matrix.
    - Correlation analysis.
  - Dimensionality reduction (in progress):
    - Principal component analysis (PCA).
    - t-SNE visualization.
  - Feature importance (in progress):
    - For categorical variables: chi-squared test.
    - For numerical target variables: correlation analysis.
