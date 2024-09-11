pub enum SampleModeEnum {
    Limit,
    Ratio,
    Full,
}

const SAMPLE_LIMIT: u64 = 100;
const SAMPLE_RATIO: f64 = 0.5;
const SAMPLE_MODE: SampleModeEnum = SampleModeEnum::Ratio;

pub mod missing_value_viz;
