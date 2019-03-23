use crate::bucket_strainer::{Bucket, Rule};
use crate::Scalar;

/// Bucket rule that multiply scores of all of its subrules.
#[derive(Clone)]
pub struct MulRule<T> {
    /// List of subrules to score.
    pub rules: Vec<Box<dyn Rule<T>>>,
}

impl<T> MulRule<T> {
    pub fn new(rules: Vec<Box<dyn Rule<T>>>) -> Self {
        Self { rules }
    }
}

impl<T> Rule<T> for MulRule<T>
where
    T: Clone + 'static,
{
    fn score(&self, item: &T, bucket: &Bucket<T>) -> Scalar {
        self.rules
            .iter()
            .fold(1.0, |a, r| a * r.score(item, bucket))
    }

    fn box_clone(&self) -> Box<dyn Rule<T>> {
        Box::new((*self).clone())
    }
}
