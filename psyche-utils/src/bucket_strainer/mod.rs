//! Tools used to split data collection by their utility/category.

mod bucket;
mod layer;
mod rules;
#[cfg(test)]
mod tests;

pub use bucket::*;
pub use layer::*;
pub use rules::*;
use std::mem;

/// Bucket strainer is a data collection processor that splits and sorts input data into buckets
/// with rules that item must obey to fall into them. Items that does not obey any bucket rule, are
/// leftovers returned by processing.
///
/// Best problem that bucket strainer can solve is task commander that will sort AI agents into
/// buckets that represent different tasks to perform.
///
/// # How it works
/// 1. Bucket strainer contains layers of filtering organized in sequential manner, so there is
///     more possibility for items to fall into first layers than into last layers.
/// 1. Each layer contains buckets that will compete for incomming items, item can fall only into
///     one of all layer buckets and that bucket is selected based on highest score that bucket
///     will get from item based on bucket rules.
/// 1. Each Bucket contains collection of items that fall into them and main rule that will score
///     each incomming item and use that score to tell processor which bucket got highest score and
///     by that which bucket will get incoming item.
#[derive(Clone)]
pub struct BucketStrainer<T>
where
    T: Clone,
{
    layers: Vec<Layer<T>>,
}

impl<T> BucketStrainer<T>
where
    T: Clone,
{
    /// Creates bucket strainer processor.
    ///
    /// # Arguments
    /// * `layers` - List of layers that will process incoming items.
    ///
    /// # Return
    /// Instance of bucket strainer.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::Scalar;
    /// use psyche_utils::bucket_strainer::{BucketStrainer, Layer, Bucket, Rule};
    ///
    /// #[derive(Clone, Copy)]
    /// enum EvenOrOddRule {
    ///     Even,
    ///     Odd,
    /// }
    ///
    /// impl Rule<i32> for EvenOrOddRule {
    ///     fn score(&self, item: &i32, _: &Bucket<i32>) -> Scalar {
    ///         let even = match self {
    ///             EvenOrOddRule::Even => 0,
    ///             EvenOrOddRule::Odd => 1,
    ///         };
    ///         if *item % 2 == even {
    ///             1.0
    ///         } else {
    ///             0.0
    ///         }
    ///     }
    ///
    ///     fn box_clone(&self) -> Box<dyn Rule<i32>> {
    ///         Box::new((*self).clone())
    ///     }
    /// }
    ///
    /// let bs = BucketStrainer::new(vec![
    ///     Layer::new(vec![
    ///         Bucket::new("even".to_owned(), Box::new(EvenOrOddRule::Even)),
    ///     ]),
    ///     Layer::new(vec![
    ///         Bucket::new("odd".to_owned(), Box::new(EvenOrOddRule::Odd)),
    ///     ]),
    /// ]);
    /// ```
    pub fn new(layers: Vec<Layer<T>>) -> Self {
        Self { layers }
    }

    /// Gets list of layers.
    ///
    /// # Return
    /// Reference to slice of layers.
    pub fn layers(&self) -> &[Layer<T>] {
        &self.layers
    }

    /// Replace existing layers with new ones.
    ///
    /// # Arguments
    /// * `layers` - List of new layers.
    ///
    /// # Return
    /// List of old layers.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::bucket_strainer::{BucketStrainer, Layer, Bucket, BucketLimitRule};
    ///
    /// let mut bs = BucketStrainer::<()>::new(vec![]);
    /// bs.replace_layers(vec![
    ///     Bucket::new("limit".to_owned(), Box::new(BucketLimitRule::new(3))).into(),
    /// ]);
    /// ```
    pub fn replace_layers(&mut self, layers: Vec<Layer<T>>) -> Vec<Layer<T>> {
        mem::replace(&mut self.layers, layers)
    }

    /// Finds bucket by its ID.
    ///
    /// # Arguments
    /// * `id` - Bucket ID.
    ///
    /// # Return
    /// Reference to bucket.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::bucket_strainer::{BucketStrainer, Layer, Bucket, BucketLimitRule};
    ///
    /// let bs = BucketStrainer::<()>::new(vec![
    ///     Bucket::new("limit".to_owned(), Box::new(BucketLimitRule::new(3))).into(),
    /// ]);
    /// assert!(bs.bucket("limit").is_some());
    /// ```
    pub fn bucket(&self, id: &str) -> Option<&Bucket<T>> {
        for layer in &self.layers {
            if let Some(bucket) = layer.bucket(id) {
                return Some(bucket);
            }
        }
        None
    }

    /// Clears all layers buckets items collections.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::bucket_strainer::{BucketStrainer, Layer, Bucket, BucketLimitRule};
    ///
    /// let mut bs = BucketStrainer::new(vec![
    ///     Bucket::new("limit".to_owned(), Box::new(BucketLimitRule::new(3))).into(),
    /// ]);
    /// bs.process(vec![0, 1, 2, 3, 4, 5, 6]);
    /// assert_eq!(bs.bucket("limit").unwrap().items().len(), 3);
    /// bs.clear_layers_buckets();
    /// assert_eq!(bs.bucket("limit").unwrap().items().len(), 0);
    /// ```
    pub fn clear_layers_buckets(&mut self) {
        for layer in &mut self.layers {
            layer.clear_buckets();
        }
    }

    /// Process input items.
    ///
    /// # Arguments
    /// * `items` - List of items to process.
    ///
    /// # Return
    /// Processed items leftovers that does not fall into any bucket.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::bucket_strainer::{BucketStrainer, Layer, Bucket, BucketLimitRule};
    ///
    /// let mut bs = BucketStrainer::new(vec![
    ///     Bucket::new("limitA".to_owned(), Box::new(BucketLimitRule::new(3))).into(),
    ///     Bucket::new("limitB".to_owned(), Box::new(BucketLimitRule::new(2))).into(),
    /// ]);
    /// let leftovers = bs.process(vec![0, 1, 2, 3, 4, 5, 6]);
    /// assert_eq!(bs.bucket("limitA").unwrap().items(), &[0, 1, 2]);
    /// assert_eq!(bs.bucket("limitB").unwrap().items(), &[3, 4]);
    /// assert_eq!(&leftovers, &[5, 6]);
    /// ```
    pub fn process(&mut self, mut items: Vec<T>) -> Vec<T> {
        self.clear_layers_buckets();
        for layer in &mut self.layers {
            items = layer.process(items);
            if items.is_empty() {
                break;
            }
        }
        items
    }

    /// Get list of bucket with their items pairs.
    ///
    /// # Return
    /// Pairs of buckets with their items.
    ///
    /// # Example
    /// ```
    /// use psyche_utils::bucket_strainer::{BucketStrainer, Layer, Bucket, BucketLimitRule};
    ///
    /// let mut bs = BucketStrainer::new(vec![
    ///     Bucket::new("limitA".to_owned(), Box::new(BucketLimitRule::new(3))).into(),
    ///     Bucket::new("limitB".to_owned(), Box::new(BucketLimitRule::new(2))).into(),
    /// ]);
    /// bs.process(vec![0, 1, 2, 3, 4, 5, 6]);
    /// let pairs = bs.buckets_items_pairs();
    /// assert_eq!(pairs.len(), 2);
    /// assert_eq!(pairs[0].0, "limitA");
    /// assert_eq!(pairs[0].1.to_vec(), vec![0, 1, 2]);
    /// assert_eq!(pairs[1].0, "limitB");
    /// assert_eq!(pairs[1].1.to_vec(), vec![3, 4]);
    /// ```
    pub fn buckets_items_pairs<'a>(&'a self) -> Vec<(&'a str, &'a [T])> {
        self.layers
            .iter()
            .flat_map(|layer| {
                layer
                    .buckets()
                    .iter()
                    .map(|bucket| (bucket.id(), bucket.items()))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
