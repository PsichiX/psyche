use crate::bucket_strainer::{Bucket, Rule};
use crate::Scalar;

/// Bucket rule that always gives specified score.
#[derive(Clone)]
pub struct FixedScoreRule {
    /// Score value.
    pub score: Scalar,
}

impl FixedScoreRule {
    pub fn new(score: Scalar) -> Self {
        Self { score }
    }
}

impl<T> Rule<T> for FixedScoreRule
where
    T: Clone,
{
    fn score(&self, _: &T, _: &Bucket<T>) -> Scalar {
        self.score
    }

    fn box_clone(&self) -> Box<dyn Rule<T>> {
        Box::new((*self).clone())
    }
}
