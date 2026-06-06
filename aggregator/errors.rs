use thiserror::Error;

#[derive(Debug, Error)]
pub enum AggregatorError {
    #[error("github: {0}")]
    GitHub(String),
    #[error("codeforces: {0}")]
    Codeforces(String),
    #[error("codewars: {0}")]
    Codewars(String),
    #[error("leetcode: {0}")]
    LeetCode(String),
    #[error("visitors: {0}")]
    Visitors(String),
}
