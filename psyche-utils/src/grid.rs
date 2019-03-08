use crate::Scalar;
use std::ops::{Add, Index, IndexMut, Mul};

#[derive(Clone, Default)]
pub struct Grid<T> {
    cols: usize,
    rows: usize,
    fields: Vec<T>,
}

impl<T> Grid<T>
where
    T: Clone,
{
    #[inline]
    pub fn new(cols: usize, rows: usize, value: T) -> Self {
        Self {
            cols,
            rows,
            fields: vec![value; cols * rows],
        }
    }

    #[inline]
    pub fn cols(&self) -> usize {
        self.cols
    }

    #[inline]
    pub fn rows(&self) -> usize {
        self.rows
    }

    #[inline]
    pub fn fields(&self) -> &[T] {
        &self.fields
    }

    pub fn fields_mut(&mut self) -> &mut [T] {
        &mut self.fields
    }

    #[inline]
    pub fn fill_all(&mut self, value: T) {
        self.fields = vec![value; self.cols * self.rows];
    }

    pub fn fill(&mut self, col_row: (usize, usize), size: (usize, usize), value: T) {
        for y in col_row.1.min(self.rows)..(col_row.1 + size.1).min(self.rows) {
            for x in col_row.0.min(self.cols)..(col_row.0 + size.0).min(self.cols) {
                let index = y * self.cols + x;
                self.fields[index] = value.clone();
            }
        }
    }

    pub fn fill_with<F>(&mut self, mut with: F)
    where
        F: FnMut(usize, usize) -> Option<T>,
    {
        for y in 0..self.rows {
            for x in 0..self.cols {
                let index = y * self.cols + x;
                if let Some(value) = with(x, y) {
                    self.fields[index] = value;
                }
            }
        }
    }

    pub fn with<F>(&mut self, mut with: F)
    where
        F: FnMut(usize, usize, &mut T),
    {
        for (index, field) in self.fields.iter_mut().enumerate() {
            let x = index % self.cols;
            let y = index / self.rows;
            with(x, y, field);
        }
    }

    pub fn sample<S, W>(&self, sampler: S) -> Option<(T, W)>
    where
        S: GridSampler<T, W>,
    {
        sampler.sample(self)
    }
}

impl<T> Index<(usize, usize)> for Grid<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: (usize, usize)) -> &T {
        let index = index.1 * self.cols + index.0;
        self.fields.index(index)
    }
}

impl<T> IndexMut<(usize, usize)> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut T {
        let index = index.1 * self.cols + index.0;
        self.fields.index_mut(index)
    }
}

impl<T> Index<[usize; 2]> for Grid<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: [usize; 2]) -> &T {
        let index = index[1] * self.cols + index[0];
        self.fields.index(index)
    }
}

impl<T> IndexMut<[usize; 2]> for Grid<T> {
    #[inline]
    fn index_mut(&mut self, index: [usize; 2]) -> &mut T {
        let index = index[1] * self.cols + index[0];
        self.fields.index_mut(index)
    }
}

pub trait GridSampler<T, W> {
    fn sample(self, grid: &Grid<T>) -> Option<(T, W)>;
}

pub trait GridSampleZeroValue<T> {
    fn sample_zero_value() -> T;
}

impl GridSampleZeroValue<Self> for f32 {
    fn sample_zero_value() -> Self {
        0.0
    }
}

impl GridSampleZeroValue<Self> for f64 {
    fn sample_zero_value() -> Self {
        0.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridSamplerCluster {
    pub from: (usize, usize),
    pub to: (usize, usize),
}

impl GridSamplerCluster {
    #[inline]
    pub fn new(from: (usize, usize), to: (usize, usize)) -> Self {
        Self { from, to }
    }

    pub fn center(center: (usize, usize), size: (usize, usize)) -> Self {
        let extents = (size.0 / 2, size.1 / 2);
        let from = (
            if extents.0 > center.0 {
                0
            } else {
                center.0 - extents.0
            },
            if extents.1 > center.1 {
                0
            } else {
                center.1 - extents.1
            },
        );
        let to = (center.0 + extents.0, center.1 + extents.1);
        Self { from, to }
    }

    pub fn center_extents(center: (usize, usize), extents: (usize, usize)) -> Self {
        let from = (
            if extents.0 > center.0 {
                0
            } else {
                center.0 - extents.0
            },
            if extents.1 > center.1 {
                0
            } else {
                center.1 - extents.1
            },
        );
        let to = (center.0 + extents.0, center.1 + extents.1);
        Self { from, to }
    }

    pub fn valid_from(&self) -> (usize, usize) {
        (self.from.0.min(self.to.0), self.from.1.min(self.to.1))
    }

    pub fn valid_to(&self) -> (usize, usize) {
        (self.from.0.max(self.to.0), self.from.1.max(self.to.1))
    }

    pub fn cells(&self) -> usize {
        let from = self.valid_from();
        let to = self.valid_to();
        (to.0 - from.0) * (to.1 - from.1)
    }
}

impl<T> GridSampler<T, usize> for GridSamplerCluster
where
    T: GridSampleZeroValue<T> + Add<Output = T> + Clone,
{
    fn sample(self, grid: &Grid<T>) -> Option<(T, usize)> {
        if grid.cols() > 0 && grid.rows() > 0 {
            let from = self.valid_from();
            let mut to = self.valid_to();
            to.0 = to.0.min(grid.cols() - 1);
            to.1 = to.1.min(grid.rows() - 1);
            let mut result = T::sample_zero_value();
            let mut count = 0;
            for y in from.1..=to.1 {
                for x in from.0..=to.0 {
                    result = result + grid[(x, y)].clone();
                    count += 1;
                }
            }
            Some((result, count))
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridSamplerDistance {
    pub center: (Scalar, Scalar),
    pub range: Scalar,
    pub cell_size: (Scalar, Scalar),
}

impl GridSamplerDistance {
    #[inline]
    pub fn new(center: (Scalar, Scalar), range: Scalar, cell_size: (Scalar, Scalar)) -> Self {
        Self {
            center,
            range,
            cell_size,
        }
    }
}

impl<T> GridSampler<T, Scalar> for GridSamplerDistance
where
    T: GridSampleZeroValue<T> + Add<Output = T> + Clone + Mul<Scalar, Output = T>,
{
    fn sample(self, grid: &Grid<T>) -> Option<(T, Scalar)> {
        if grid.cols() > 0 && grid.rows() > 0 {
            let mut result = T::sample_zero_value();
            let mut total_weight = 0.0;
            for y in 0..grid.rows() {
                for x in 0..grid.cols() {
                    let value = grid[(x, y)].clone();
                    let dx = x as Scalar * self.cell_size.0 - self.center.0;
                    let dy = y as Scalar * self.cell_size.1 - self.center.1;
                    let distance = (dx * dx + dy * dy).sqrt();
                    if distance < self.range {
                        let weight = 1.0 - distance / self.range;
                        result = result + value * weight;
                        total_weight += weight;
                    }
                }
            }
            Some((result, total_weight))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample() {
        let sampler = GridSamplerCluster::center((1, 1), (2, 2));
        assert_eq!(sampler, GridSamplerCluster::new((0, 0), (2, 2)));

        let grid = Grid::new(3, 3, 1.0);
        let sample = grid.sample(sampler).unwrap();
        assert_eq!((sample.0 as i32, sample.1), (9, 9));

        let sampler = GridSamplerCluster::center((0, 0), (2, 2));
        let sample = grid.sample(sampler).unwrap();
        assert_eq!((sample.0 as i32, sample.1), (4, 4));

        let sampler = GridSamplerCluster::center((2, 2), (2, 2));
        let sample = grid.sample(sampler).unwrap();
        assert_eq!((sample.0 as i32, sample.1), (4, 4));

        let grid = Grid::new(9, 1, 1.0);
        let sampler = GridSamplerDistance::new((0.0, 0.0), 3.0, (1.0, 1.0));
        let sample = grid.sample(sampler).unwrap();

        assert_eq!((sample.0 as i32, sample.1 as i32), (2, 2));
        let sampler = GridSamplerDistance::new((4.0, 0.0), 3.0, (1.0, 1.0));
        let sample = grid.sample(sampler).unwrap();
        assert_eq!((sample.0 as i32, sample.1 as i32), (3, 3));
        let sampler = GridSamplerDistance::new((8.0, 0.0), 3.0, (1.0, 1.0));
        let sample = grid.sample(sampler).unwrap();
        assert_eq!((sample.0 as i32, sample.1 as i32), (2, 2));
    }
}
