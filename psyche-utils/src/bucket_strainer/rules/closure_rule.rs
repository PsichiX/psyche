use crate::bucket_strainer::{Bucket, Rule};
use crate::Scalar;

/// Bucket rule that applies score based on closure result.
#[derive(Clone)]
pub struct ClosureRule<T>
where
    T: Clone,
{
    /// Closure that will calculate item score.
    pub closure: fn(&T, &Bucket<T>) -> Scalar,
}

impl<T> ClosureRule<T>
where
    T: Clone,
{
    pub fn new(closure: fn(&T, &Bucket<T>) -> Scalar) -> Self {
        Self { closure }
    }
}

impl<T> Rule<T> for ClosureRule<T>
where
    T: Clone + 'static,
{
    fn score(&self, item: &T, bucket: &Bucket<T>) -> Scalar {
        (self.closure)(item, bucket)
    }

    fn box_clone(&self) -> Box<dyn Rule<T>> {
        Box::new((*self).clone())
    }
}
