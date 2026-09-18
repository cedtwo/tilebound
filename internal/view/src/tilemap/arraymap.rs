use std::ops::Range;

use marker_value::MarkerValue;
use tilebound_plane::axis::{Axis, AxisVec, AxisX, AxisY};

use crate::tilemap::{TileMap, TileMapView};

/// A simple two dimensional array implementation. Requires specifying the number of rows and
/// columns (`X` and `Y` respectively), and the total number of elements, `N`.
#[derive(Debug, Clone, Copy)]
pub struct ArrayMap<const X: usize, const Y: usize, const N: usize, T>(pub [T; N]);

impl<const X: usize, const Y: usize, const N: usize, T> ArrayMap<X, Y, N, T> {
    /// Create a new `ArrayMap` containing the given elements.
    pub fn new(array: [T; N]) -> Self {
        assert_eq!(N, X * Y);
        Self(array)
    }

    /// Create a new `ArrayMap` containing the given elements.
    pub const fn new_unchecked(array: [T; N]) -> Self {
        Self(array)
    }
}

/// [`ArrayMap`] implementation with **row major** ordering.
impl<const X: usize, const Y: usize, const N: usize, T: Copy> TileMap for ArrayMap<X, Y, N, T> {
    type El = T;

    fn axis_len<A: Axis>(&self) -> usize {
        match A::VALUE {
            AxisX::VALUE => X,
            AxisY::VALUE => Y,
            _ => unreachable!(),
        }
    }
}

impl<const X: usize, const Y: usize, const N: usize, T: Copy> TileMapView<AxisX>
    for ArrayMap<X, Y, N, T>
{
    type View<'a>
        = &'a [T]
    where
        T: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        let start = idx * Y + range.start;
        let end = start + range.len();

        &self.0[start..end]
    }
}

impl<const X: usize, const Y: usize, const N: usize, T: Copy> TileMapView<AxisY>
    for ArrayMap<X, Y, N, T>
{
    type View<'a>
        = ColStrideIter<'a, T>
    where
        T: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        ColStrideIter {
            slice: self.0.as_slice(),
            row_len: Y,
            col_idx: idx,
            range,
        }
    }
}

/// A simple two dimensional vector implementation. Dimensions are stored as a variable rather than
/// a constant (unlike [`ArrayMap`]). Dimensions are stored as an [`AxisVec`] containing the number
/// of rows and columns for the *x* and *y* axis respectively.
#[derive(Debug, Clone)]
pub struct VecMap<T> {
    /// The element array.
    pub vec: Vec<T>,
    /// The array dimensions.
    pub dim: AxisVec<usize>,
}

impl<T> VecMap<T> {
    /// Create a new `VecMap` containing the given element vector of the given dimensions.
    /// **Panics** if the dimensions do not match the vector size.
    pub fn new(vec: Vec<T>, dim: impl Into<AxisVec<usize>>) -> Self {
        let dim = dim.into();
        assert_eq!(vec.len(), dim.flat_map(|[x, y]| x * y));
        Self { vec, dim }
    }

    /// Create a new `VecMap` containing the given element vector of the given dimensions. Does not
    /// assert if the dimensions match the vector size.
    pub fn new_unchecked(vec: Vec<T>, dim: impl Into<AxisVec<usize>>) -> Self {
        let dim = dim.into();
        Self { vec, dim }
    }
}

/// [`VecMap`] implementation with **row major** ordering.
impl<T: Copy> TileMap for VecMap<T> {
    type El = T;

    fn axis_len<A: Axis>(&self) -> usize {
        match A::VALUE {
            AxisX::VALUE => self.dim.x(),
            AxisY::VALUE => self.dim.y(),
            _ => unreachable!(),
        }
    }
}

impl<T: Copy> TileMapView<AxisX> for VecMap<T> {
    type View<'a>
        = &'a [T]
    where
        T: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        let start = idx * self.dim.y() + range.start;
        let end = start + range.len();

        &self.vec[start..end]
    }
}

impl<T: Copy> TileMapView<AxisY> for VecMap<T> {
    type View<'a>
        = ColStrideIter<'a, T>
    where
        T: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        ColStrideIter {
            slice: self.vec.as_slice(),
            row_len: self.dim.y(),
            col_idx: idx,
            range,
        }
    }
}

/// [`ArrayMap`]/[`VecMap`] column iterator.
pub struct ColStrideIter<'a, T> {
    /// A reference to the array as a slice.
    slice: &'a [T],
    /// The number of elements in a row.
    row_len: usize,
    /// The column index.
    col_idx: usize,
    /// The row element range.
    range: Range<usize>,
}

impl<'a, T> Iterator for ColStrideIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.range
            .next()
            .map(|i| &self.slice[i * self.row_len + self.col_idx])
    }
}

// impl_dynamic!([T: Copy + Deref<Target = Self::El>], VecMap<T>);

#[cfg(test)]
mod tests {

    use super::*;

    mod array_map {
        use super::*;

        #[rustfmt::skip]
        const MAP: ArrayMap<4, 3, 12, u8> = ArrayMap([
            00, 01, 02,
            03, 04, 05,
            06, 07, 08,
            09, 10, 11,
        ]);

        #[test]
        fn axis_size() {
            assert_eq!(MAP.axis_len::<AxisX>(), 4);
            assert_eq!(MAP.axis_len::<AxisY>(), 3);
        }

        #[test]
        fn is_in_bounds() {
            assert!(MAP.is_in_bounds::<AxisX>(0));
            assert!(MAP.is_in_bounds::<AxisX>(3));
            assert!(!MAP.is_in_bounds::<AxisX>(-1));
            assert!(!MAP.is_in_bounds::<AxisX>(4));
        }

        #[test]
        fn range_is_in_bounds() {
            assert!(MAP.range_is_in_bounds::<AxisX>(0, -1..1));
            assert!(MAP.range_is_in_bounds::<AxisX>(3, 2..5));
            assert!(!MAP.range_is_in_bounds::<AxisX>(0, -2..-1));
            assert!(!MAP.range_is_in_bounds::<AxisX>(3, 3..5));
        }

        #[test]
        fn iter_row() {
            assert_eq!(
                TileMapView::<AxisX>::view(&MAP, 0, 0..3)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                [0, 1, 2]
            );
            assert_eq!(
                TileMapView::<AxisX>::view(&MAP, 3, 0..3)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                [9, 10, 11]
            );
        }

        #[test]
        fn iter_col() {
            assert_eq!(
                TileMapView::<AxisY>::view(&MAP, 0, 0..3)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                [0, 3, 6]
            );
            assert_eq!(
                TileMapView::<AxisY>::view(&MAP, 2, 1..4)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                [5, 8, 11]
            );
        }
    }

    mod vec_map {
        use super::*;

        #[test]
        fn axis_size() {
            let map = VecMap::<u8>::new(vec![0; 12], (4, 3));
            assert_eq!(map.axis_len::<AxisX>(), 4);
            assert_eq!(map.axis_len::<AxisY>(), 3);
        }

        #[test]
        fn is_in_bounds() {
            let map = VecMap::<u8>::new(vec![0; 12], (4, 3));
            assert!(map.is_in_bounds::<AxisX>(0));
            assert!(map.is_in_bounds::<AxisX>(3));
            assert!(!map.is_in_bounds::<AxisX>(-1));
            assert!(!map.is_in_bounds::<AxisX>(4));
        }

        #[test]
        fn range_is_in_bounds() {
            let map = VecMap::<u8>::new(vec![0; 12], (4, 3));
            assert!(map.range_is_in_bounds::<AxisX>(0, -1..1));
            assert!(map.range_is_in_bounds::<AxisX>(3, 2..5));
            assert!(!map.range_is_in_bounds::<AxisX>(0, -2..-1));
            assert!(!map.range_is_in_bounds::<AxisX>(3, 3..5));
        }

        #[test]
        fn iter_row() {
            #[rustfmt::skip]
            let map = VecMap::<u8>::new([
                00, 01, 02,
                03, 04, 05,
                06, 07, 08,
                09, 10, 11,
            ].to_vec(), (4, 3));

            assert_eq!(
                TileMapView::<AxisX>::view(&map, 0, 0..3)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                [0, 1, 2]
            );
            assert_eq!(
                TileMapView::<AxisX>::view(&map, 3, 0..3)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                [9, 10, 11]
            );
        }

        #[test]
        fn iter_col() {
            #[rustfmt::skip]
            let map = VecMap::<u8>::new([
                00, 01, 02,
                03, 04, 05,
                06, 07, 08,
                09, 10, 11,
            ].to_vec(), (4, 3));

            assert_eq!(
                TileMapView::<AxisY>::view(&map, 0, 0..3)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                [0, 3, 6]
            );
            assert_eq!(
                TileMapView::<AxisY>::view(&map, 2, 1..4)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>(),
                [5, 8, 11]
            );
        }
    }
}
