#![cfg(test)]
use super::*;
use crate::Scalar;
use std::ops::Range;

#[derive(Clone, Copy)]
enum EvenOrOddRule {
    Even,
    Odd,
}

impl Rule<i32> for EvenOrOddRule {
    fn score(&self, item: &i32, _: &Bucket<i32>) -> Scalar {
        let even = match self {
            EvenOrOddRule::Even => 0,
            EvenOrOddRule::Odd => 1,
        };
        if *item % 2 == even {
            1.0
        } else {
            0.0
        }
    }

    fn box_clone(&self) -> Box<dyn Rule<i32>> {
        Box::new((*self).clone())
    }
}

impl Rule<i32> for Range<i32> {
    fn score(&self, item: &i32, _: &Bucket<i32>) -> Scalar {
        if *item >= self.start && *item <= self.end {
            1.0
        } else {
            0.0
        }
    }

    fn box_clone(&self) -> Box<dyn Rule<i32>> {
        Box::new((*self).clone())
    }
}

#[test]
fn test_general() {
    let mut bs = BucketStrainer::new(vec![
        Layer::new(vec![Bucket::new("range".to_owned(), Box::new(11..20))]),
        Layer::new(vec![
            Bucket::new("even".to_owned(), Box::new(EvenOrOddRule::Even)),
            Bucket::new("odd".to_owned(), Box::new(EvenOrOddRule::Odd)),
        ]),
    ]);

    let leftovers = bs.process((0..=15).collect());
    let meta = bs.buckets_items_pairs();
    assert_eq!(meta.len(), 3);
    assert_eq!(meta[0].0, "range");
    assert_eq!(meta[0].1, [11, 12, 13, 14, 15]);
    assert_eq!(meta[1].0, "even");
    assert_eq!(meta[1].1, [0, 2, 4, 6, 8, 10]);
    assert_eq!(meta[2].0, "odd");
    assert_eq!(meta[2].1, [1, 3, 5, 7, 9]);
    assert_eq!(leftovers.len(), 0);
}
