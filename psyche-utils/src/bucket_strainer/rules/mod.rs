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

/// Trait used to tell how much successfuly bucket has to get incoming item.
///
/// # Example
/// ```
/// use psyche_utils::bucket_strainer::{Bucket, Rule};
/// use psyche_utils::Scalar;
///
/// #[derive(Clone)]
/// struct SuccessRule;
///
/// impl<T> Rule<T> for SuccessRule
/// where
///     T: Clone,
/// {
///     fn score(&self, _: &T, _: &Bucket<T>) -> Scalar {
///         1.0
///     }
///
///     fn box_clone(&self) -> Box<dyn Rule<T>> {
///         Box::new((*self).clone())
///     }
/// }
/// ```
pub trait Rule<T>
where
    T: Clone,
{
    /// Score incoming item.
    ///
    /// # Arguments
    /// * `item` - Incoming item.
    /// * `bucket` - Bucket that tests incoming item.
    ///
    /// # Return
    /// Score for given item that tell how lucky it is to fall into given bucket.
    fn score(&self, item: &T, bucket: &Bucket<T>) -> Scalar;

    /// Create boxed clone for this rule.
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
