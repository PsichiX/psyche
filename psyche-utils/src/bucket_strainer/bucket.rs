use crate::bucket_strainer::{Layer, Rule};
use crate::Scalar;
use std::mem;

/// Bucket strainer bucket that contains rule that scores items and collection of items that fallen
/// into it on processing phase.
#[derive(Clone)]
pub struct Bucket<T>
where
    T: Clone,
{
    id: String,
    items: Vec<T>,
    rule: Box<dyn Rule<T>>,
    pub acceptable_score_treshold: Scalar,
}

impl<T> Bucket<T>
where
    T: Clone,
{
    pub fn new(id: String, rule: Box<dyn Rule<T>>) -> Self {
        Self {
            id,
            items: vec![],
            rule,
            acceptable_score_treshold: 0.0,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn items(&self) -> &[T] {
        &self.items
    }

    pub fn rule(&self) -> &Box<dyn Rule<T>> {
        &self.rule
    }

    pub fn replace_rule(&mut self, rule: Box<dyn Rule<T>>) -> Box<dyn Rule<T>> {
        mem::replace(&mut self.rule, rule)
    }

    pub(crate) fn score(&self, item: &T) -> Option<Scalar> {
        let score = self.rule.score(item, self);
        if score > self.acceptable_score_treshold {
            Some(score)
        } else {
            None
        }
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
    }

    pub(crate) fn insert(&mut self, item: T) {
        self.items.push(item);
    }
}

impl<T> Into<Layer<T>> for Bucket<T>
where
    T: Clone,
{
    fn into(self) -> Layer<T> {
        Layer::new(vec![self])
    }
}
