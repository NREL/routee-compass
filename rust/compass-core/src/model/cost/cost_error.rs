#[derive(thiserror::Error, Debug, Clone)]
pub enum CostError {
    #[error("vector of metric observations is wrong length for provided cost function")]
    MetricVectorSizeMismatch,
}
