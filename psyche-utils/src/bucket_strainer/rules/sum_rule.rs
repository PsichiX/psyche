use crate::bucket_strainer::{Bucket, Rule};
use crate::Scalar;

#[derive(Clone)]
pub struct SumRule<T> {
    pub rules: Vec<Box<dyn Rule<T>>>,
}

impl<T> SumRule<T> {
    pub fn new(rules: Vec<Box<dyn Rule<T>>>) -> Self {
        Self { rules }
    }
}

impl<T> Rule<T> for SumRule<T>
where
    T: Clone + 'static,
{
    fn score(&self, item: &T, bucket: &Bucket<T>) -> Scalar {
        self.rules
            .iter()
            .fold(0.0, |a, r| a + r.score(item, bucket))
    }

    fn box_clone(&self) -> Box<dyn Rule<T>> {
        Box::new((*self).clone())
    }
}
