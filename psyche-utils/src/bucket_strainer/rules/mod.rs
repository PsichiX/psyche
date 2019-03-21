mod bucket_limit_rule;
mod closure_rule;
mod fixed_score_rule;
mod mul_rule;
mod sum_rule;

use crate::bucket_strainer::Bucket;
use crate::Scalar;
pub use bucket_limit_rule::*;
pub use closure_rule::*;
pub use fixed_score_rule::*;
pub use mul_rule::*;
pub use sum_rule::*;

pub trait Rule<T>
where
    T: Clone,
{
    fn score(&self, item: &T, bucket: &Bucket<T>) -> Scalar;
    fn box_clone(&self) -> Box<dyn Rule<T>>;
}

impl<T> Clone for Box<Rule<T>>
where
    T: Clone,
{
    fn clone(&self) -> Box<Rule<T>> {
        self.box_clone()
    }
}
