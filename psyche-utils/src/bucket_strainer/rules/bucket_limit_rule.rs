use crate::bucket_strainer::{Bucket, Rule};
use crate::Scalar;

#[derive(Clone)]
pub struct BucketLimitRule {
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
        if bucket.items().len() > self.limit {
            0.0
        } else {
            1.0
        }
    }

    fn box_clone(&self) -> Box<dyn Rule<T>> {
        Box::new((*self).clone())
    }
}
