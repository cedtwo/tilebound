use std::ops::Range;

use marker_value::MarkerValue;
use ndarray::{ArrayView1, s};

use crate::plane::axis::{Axis, AxisVec, AxisX, AxisY};
use crate::view::tilemap::{TileMap, TileMapView};

impl<T> TileMap for ndarray::Array2<T> {
    type El = T;

    fn axis_len<A: Axis>(&self) -> usize {
        match A::VALUE {
            AxisX::VALUE => self.dim().0,
            AxisY::VALUE => self.dim().1,
            _ => unreachable!(),
        }
    }

    fn size(&self) -> AxisVec<usize> {
        let dim = self.dim();
        AxisVec::new(dim.1, dim.0)
    }
}

impl<T> TileMapView<AxisX> for ndarray::Array2<T> {
    type View<'a>
        = ArrayView1<'a, T>
    where
        Self: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        self.slice(s!(idx, range))
    }
}

impl<T> TileMapView<AxisY> for ndarray::Array2<T> {
    type View<'a>
        = ArrayView1<'a, T>
    where
        Self: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        self.slice(s!(range, idx))
    }
}

// impl_dynamic!([T], ndarray::Array2<T>);

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn axis_size() {
        let map: ndarray::Array2<u8> = ndarray::Array2::zeros((3, 4));
        assert_eq!(map.axis_len::<AxisX>(), 3);
        assert_eq!(map.axis_len::<AxisY>(), 4);
    }

    #[test]
    fn is_in_bounds() {
        let map: ndarray::Array2<u8> = ndarray::Array2::zeros((3, 4));
        assert!(map.is_in_bounds::<AxisX>(0));
        assert!(map.is_in_bounds::<AxisX>(2));
        assert!(!map.is_in_bounds::<AxisX>(-1));
        assert!(!map.is_in_bounds::<AxisX>(3));
    }

    #[test]
    fn range_is_in_bounds() {
        let map: ndarray::Array2<u8> = ndarray::Array2::zeros((3, 4));
        assert!(map.range_is_in_bounds::<AxisX>(0, -1..1));
        assert!(map.range_is_in_bounds::<AxisX>(2, 3..5));
        assert!(!map.range_is_in_bounds::<AxisX>(0, -2..-1));
        assert!(!map.range_is_in_bounds::<AxisX>(2, 4..5));
    }

    #[test]
    fn view() {
        let mut map: ndarray::Array2<u8> = ndarray::Array2::zeros((3, 4));
        map.row_mut(0).iter_mut().for_each(|el| *el = 1);

        assert_eq!(
            TileMapView::<AxisX>::view(&map, 0, 0..4)
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            [1, 1, 1, 1]
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 0, 0..3)
                .iter()
                .cloned()
                .collect::<Vec<_>>(),
            [1, 0, 0]
        );
    }
}
