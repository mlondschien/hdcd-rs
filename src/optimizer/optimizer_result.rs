use crate::gain::GainResult;

#[derive(Debug)]
pub struct OptimizerResult {
    pub start: usize,
    pub stop: usize,
    pub best_split: usize,
    pub max_gain: f64,
    pub gain_results: Vec<GainResult>,
}
