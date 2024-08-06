//!

pub struct Glossary {
    pub terms: [&'static str; 15],
    pub definitions: [&'static str; 15],
}

impl Glossary {
    pub fn new() -> Self {
        Self {
            terms: TERMS,
            definitions: DEFINITIONS,
        }
    }
}

pub const TERMS: [&'static str; 15] = [
    "bool",
    "count",
    "i64",
    "iqr",
    "kurtosis",
    "max",
    "mean",
    "median",
    "min",
    "q1",
    "q3",
    "std_dev",
    "skewness_bias",
    "skewness_raw",
    "str",
];

pub const DEFINITIONS: [&'static str; 15] = [
    "A boolean value, either true or false.", // bool
    "The number of items in a dataset or column.", // count
    "A 64-bit signed integer. An i64 can represent both positive and negative integers, with a max possible value of 9,223,372,036,854,775,807 and a minimum possible value of -9,223,372,036,854,775,808.", // i64
    "Interquartile range, the difference between the third quartile (Q3) and the first quartile (Q1). The interquartile range is a measure of statistical dispersion, or the spread of the data.", // iqr
    "A measure of the 'tailedness' of the probability distribution of a real-valued random variable. Kurtosis is the fourth central moment divided by the square of the variance. In this report's case, Fisher's definition is used, which results in `3.0` being subtracted from the result to give `0.0` for a normal distribution.", // kurtosis
    "The highest value in a dataset or column.", // max
    "The average value of a dataset or column, calculated by summing all values and dividing by the count.", // mean
    "The middle value in a sorted dataset or column.", // median
    "The lowest value in a dataset or column.", // min
    "First quartile, the median of the lower half of the dataset or column.", // q1
    "Third quartile, the median of the upper half of the dataset or column.", // q3
    "Standard deviation, a measure of the amount of variation or dispersion of a set of values.", // std_dev
    "Skewness calculated with a bias correction factor. Skewness is a metric for asymmetry or distortion, measuring the deviation of a given distribution of a random variable from a normal distribution.", // skewness_bias
    "Skewness calculated without bias correction. Skewness is a metric for asymmetry or distortion, measuring the deviation of a given distribution of a random variable from a normal distribution.", // skewness_raw
    "A string, or text value.", // str
];
