use std::ops::Range;

use mdarray::*;

use crate::plane::axis::{Axis, AxisX, AxisY};
use crate::view::tilemap::{TileMap, TileMapView};

impl<T, const X: usize, const Y: usize> TileMap
    for Array<T, (mdarray::Const<X>, mdarray::Const<Y>)>
{
    type El = T;

    fn axis_len<A: Axis>(&self) -> usize {
        self.dim(A::VALUE)
    }
}

impl<T, const X: usize, const Y: usize> TileMapView<AxisX>
    for mdarray::Array<T, (mdarray::Const<X>, mdarray::Const<Y>)>
{
    type View<'a>
        = View<'a, T, (usize,), Dense>
    where
        Self: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        Slice::<_, (_, _), _>::view(self, idx, range)
    }
}

impl<T, const X: usize, const Y: usize> TileMapView<AxisY>
    for mdarray::Array<T, (mdarray::Const<X>, mdarray::Const<Y>)>
{
    type View<'a>
        = View<'a, T, (usize,), Strided>
    where
        Self: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        Slice::<_, (_, _), _>::view(self, range, idx)
    }
}

// impl_dynamic!([T: Deref<Target = Self::El>, const X: usize, const Y: usize], Array<T, (mdarray::Const<X>, mdarray::Const<Y>)>);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_size() {
        let map: Array<u8, _> = Array::zeros((Const::<3>, Const::<4>));
        assert_eq!(map.axis_len::<AxisX>(), 3);
        assert_eq!(map.axis_len::<AxisY>(), 4);
    }

    #[test]
    fn is_in_bounds() {
        let map: Array<u8, _> = Array::zeros((Const::<3>, Const::<4>));
        assert!(map.is_in_bounds::<AxisX>(0));
        assert!(map.is_in_bounds::<AxisX>(2));
        assert!(!map.is_in_bounds::<AxisX>(-1));
        assert!(!map.is_in_bounds::<AxisX>(3));
    }

    #[test]
    fn range_is_in_bounds() {
        let map: Array<u8, _> = Array::zeros((Const::<3>, Const::<4>));
        assert!(map.range_is_in_bounds::<AxisX>(0, -1..1));
        assert!(map.range_is_in_bounds::<AxisX>(2, 3..5));
        assert!(!map.range_is_in_bounds::<AxisX>(0, -2..-1));
        assert!(!map.range_is_in_bounds::<AxisX>(2, 4..5));
    }

    #[test]
    fn view() {
        let mut map: Array<u8, _> = Array::from_elem((Const::<3>, Const::<4>), 0);
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
