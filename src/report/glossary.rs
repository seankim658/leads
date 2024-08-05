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
    "A 64-bit signed integer.", // i64
    "Interquartile range, the difference between the third quartile (Q3) and the first quartile (Q1).", // iqr
    "A measure of the 'tailedness' of the probability distribution of a real-valued random variable.", // kurtosis
    "The highest value in a dataset or column.", // max
    "The average value of a dataset or column, calculated by summing all values and dividing by the count.", // mean
    "The middle value in a sorted dataset or column.", // median
    "The lowest value in a dataset or column.", // min
    "First quartile, the median of the lower half of the dataset or column.", // q1
    "Third quartile, the median of the upper half of the dataset or column.", // q3
    "Standard deviation, a measure of the amount of variation or dispersion of a set of values.", // std_dev
    "Skewness calculated with a bias correction factor.", // skewness_bias
    "Skewness calculated without bias correction.", // skewness_raw
    "A string, or text value.", // str
];
