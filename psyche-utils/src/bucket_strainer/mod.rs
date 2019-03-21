mod bucket;
mod layer;
mod rules;
#[cfg(test)]
mod tests;

pub use bucket::*;
pub use layer::*;
pub use rules::*;
use std::mem;

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
    pub fn new(layers: Vec<Layer<T>>) -> Self {
        Self { layers }
    }

    pub fn layers(&self) -> &[Layer<T>] {
        &self.layers
    }

    pub fn replace_layers(&mut self, layers: Vec<Layer<T>>) -> Vec<Layer<T>> {
        mem::replace(&mut self.layers, layers)
    }

    pub fn bucket(&self, id: &str) -> Option<&Bucket<T>> {
        for layer in &self.layers {
            if let Some(bucket) = layer.bucket(id) {
                return Some(bucket);
            }
        }
        None
    }

    pub fn clear_layers_buckets(&mut self) {
        for layer in &mut self.layers {
            layer.clear_buckets();
        }
    }

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
