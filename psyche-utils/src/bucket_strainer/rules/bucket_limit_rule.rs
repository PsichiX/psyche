use crate::bucket_strainer::{Bucket, Rule};
use crate::Scalar;

/// Bucket rule that scores `1.0` as long as given bucket has less items than specified limit.
#[derive(Clone)]
pub struct BucketLimitRule {
    /// Number of items that bucket can have.
    pub limit: usize,
}

impl BucketLimitRule {
    pub fn new(limit: usize) -> Self {
        Self { limit }
    }
}

impl<T> Rule<T> for BucketLimitRule
where
    T: Clone,
{
    fn score(&self, _: &T, bucket: &Bucket<T>) -> Scalar {
        if bucket.items().len() < self.limit {
            1.0
        } else {
            0.0
        }
    }

    fn box_clone(&self) -> Box<dyn Rule<T>> {
        Box::new((*self).clone())
    }
}
