use crate::bucket_strainer::Bucket;
use std::mem;

/// Bucket strainer layer that contains buckets.
#[derive(Clone)]
pub struct Layer<T>
where
    T: Clone,
{
    buckets: Vec<Bucket<T>>,
}

impl<T> Layer<T>
where
    T: Clone,
{
    pub fn new(buckets: Vec<Bucket<T>>) -> Self {
        Self { buckets }
    }

    pub fn buckets(&self) -> &[Bucket<T>] {
        &self.buckets
    }

    pub fn replace_buckets(&mut self, buckets: Vec<Bucket<T>>) -> Vec<Bucket<T>> {
        mem::replace(&mut self.buckets, buckets)
    }

    pub fn bucket(&self, id: &str) -> Option<&Bucket<T>> {
        self.buckets.iter().find(|bucket| bucket.id() == id)
    }

    pub(crate) fn clear_buckets(&mut self) {
        for bucket in &mut self.buckets {
            bucket.clear();
        }
    }

    pub(crate) fn process(&mut self, items: Vec<T>) -> Vec<T> {
        items
            .into_iter()
            .filter(|item| {
                if let Some(bucket) = self.select_bucket(&item) {
                    bucket.insert(item.clone());
                    false
                } else {
                    true
                }
            })
            .collect()
    }

    fn select_bucket(&mut self, item: &T) -> Option<&mut Bucket<T>> {
        self.buckets
            .iter_mut()
            .filter_map(|bucket| bucket.score(item).map(|score| (bucket, score)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(bucket, _)| bucket)
    }
}
