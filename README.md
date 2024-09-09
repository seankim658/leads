# LEADS

LEADS is a **L**azy **E**xploratory **A**nalysis **D**ata **S**ummarizer.

Writing the same boilerplate exploratory analysis code in a Jupyter notebook or Excel spreadsheet for each new dataset can be tedious. This tool automates the generation of a consistent, comprehensive, and human readable exploratory analysis report that allows you to immediately become familiar with a dataset. The generated PDF report contains the below features.

Currently supports `.csv`, `.tsv`, and `.parquet` files for inputs and `.pdf` files for report formats (eventually will work on additional report formats such as markdown).

## Feature List

- Report features:
    - [x] Title page.
    - [x] Table of contents.
    - [x] Page numbers.
    - [ ] Run metadata.
    - [x] Glossary of statistical terms (will be continually updated as new features are built out).
- Report analysis sections:
  - Data type analysis:
    - [x] Identification of feature data types.
  - Basic dataset information and descriptive statistics:
    - [x] Number of rows and columns.
    - [x] Column names and data types.
    - [x] Min, max, mean, median, standard deviation.
    - [x] Quartiles and interquartile ranges.
    - [x] Skewness and kurtosis.
  - Missing value analysis:
    - [x] Count and percentage of missing values per column.
    - [ ] Visualization of missing value patterns.
  - Distribution analysis:
    - [ ] Normality tests (Shapiro-Wilk, Anderson-Darling).
    - [ ] Q-Q plots.
  - Outlier detection:
    - [ ] Z-score method.
    - [ ] IQR method.
    - [ ] Local outlier factor (LOF).
    - [ ] Visualization of outliers.
  - Visualizations:
    - [ ] Histograms.
    - [ ] Box plots.
    - [ ] Scatter plots.
    - [ ] Correlation heatmaps.
    - [ ] Pair plots for multivariate data.
    - [ ] Unique value counts for categorical variables.
  - Multicollinearity checks:
    - [ ] Correlation matrix.
    - [ ] Variance inflation factor (VIF).
  - Pairwise data exploration:
    - [ ] Scatter plot matrix.
    - [ ] Correlation analysis.
  - Dimensionality reduction:
    - [ ] Principal component analysis (PCA).
    - [ ] t-SNE visualization.
  - Feature importance:
    - [ ] For categorical variables: chi-squared test.
    - [ ] For numerical target variables: correlation analysis.
