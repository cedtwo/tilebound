use std::ops::Range;

use bitvec::prelude::BitSlice;
use bitvec::ptr::{BitRef, Const};
use bitvec::{store::BitStore, vec::BitVec};
use marker_value::MarkerValue;

use crate::plane::axis::{Axis, AxisVec, AxisX, AxisY};
use crate::view::tilemap::{TileMap, TileMapView};

/// # BitMap
///
/// A [`TileMap`] where each colliding tile is represented by a bit. Uses [`BitVec`] as the
/// underlying storage. Note that `BitMap` is a simple wrapper around [`BitVec`] and does not
/// enforce the vector length beyond [`BitMap::new`].
pub struct BitMap<Store: BitStore = usize> {
    pub store: BitVec<Store>,
    pub size: AxisVec<usize>,
}

impl<Store: BitStore> BitMap<Store> {
    /// Create a new `BitMap` from the given [`BitVec`].
    pub fn new(store: BitVec<Store>, size: impl Into<AxisVec<usize>>) -> Self {
        let size = size.into();
        assert_eq!(
            store.len(),
            size.product(),
            "Expected a `BitVec` of exactly {}*{} bits",
            size.x(),
            size.y()
        );
        Self { store, size }
    }
}

impl<Store: BitStore> TileMap for BitMap<Store> {
    type El = bool;

    fn axis_len<A: Axis>(&self) -> usize {
        match A::VALUE {
            AxisX::VALUE => self.size.x() as usize,
            AxisY::VALUE => self.size.y() as usize,
            _ => unreachable!(),
        }
    }
}

impl<Store: BitStore> TileMapView<AxisX> for BitMap<Store> {
    type View<'a>
        = &'a BitSlice<Store>
    where
        Self: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        let start = idx * self.size.y() + range.start;
        let end = start + range.len();
        &self.store[start..end]
    }
}

impl<Store: BitStore> TileMapView<AxisY> for BitMap<Store> {
    type View<'a>
        = BitCol<'a, Store>
    where
        Self: 'a;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        BitCol {
            slice: self.store.as_bitslice(),
            col_idx: idx,
            range,
            x_len: self.size.y(),
        }
    }
}

// impl_dynamic!([Store: BitStore], BitMap<Store>);

/// A column element iterator for a [`BitMap`].
pub struct BitCol<'a, Store: BitStore> {
    /// A reference to the array as a slice.
    slice: &'a BitSlice<Store>,
    /// The column index.
    col_idx: usize,
    /// The row element range.
    range: Range<usize>,
    /// The number of elements on a slice of the *x* axis.
    x_len: usize,
}

impl<'a, Store: BitStore> Iterator for BitCol<'a, Store> {
    type Item = BitRef<'a, Const, Store>;

    fn next(&mut self) -> Option<Self::Item> {
        self.range
            .next()
            .map(|i| self.slice.get(i * self.x_len + self.col_idx))
            .flatten()
    }
}

#[cfg(test)]
mod tests {

    use bitvec::prelude::*;

    use super::*;

    /// Create a [`BitVec`] from the given `0`/`1` element array.
    macro_rules! bitarray {
        ([ $( $int:literal ),+ $(,)? ]) => {
            BitVec::<usize>::from_iter([ $( $int ),+ ].iter().map(|int: &u8| *int != 0))
        };

    }

    #[test]
    fn axis_size() {
        let map = BitMap::new(bits![0; 4 * 3].into(), (4, 3));
        assert_eq!(map.axis_len::<AxisX>(), 4);
        assert_eq!(map.axis_len::<AxisY>(), 3);
    }

    #[test]
    fn view_row() {
        #[rustfmt::skip]
        let map = BitMap::new(bits![
            1, 0, 0, 0, 0,
            0, 1, 0, 0, 0,
            0, 0, 1, 0, 0,
            0, 0, 0, 1, 0,
        ].into(), (4, 5));

        assert_eq!(
            TileMapView::<AxisX>::view(&map, 0, 0..5),
            bitarray!([1, 0, 0, 0, 0]).as_bitslice()
        );
        assert_eq!(
            TileMapView::<AxisX>::view(&map, 1, 0..5),
            bitarray!([0, 1, 0, 0, 0]).as_bitslice()
        );
        assert_eq!(
            TileMapView::<AxisX>::view(&map, 2, 0..5),
            bitarray!([0, 0, 1, 0, 0]).as_bitslice()
        );
        assert_eq!(
            TileMapView::<AxisX>::view(&map, 3, 0..5),
            bitarray!([0, 0, 0, 1, 0]).as_bitslice()
        );
    }

    #[test]
    fn view_col() {
        #[rustfmt::skip]
        let map = BitMap::new(bits![
            1, 0, 0, 0, 0,
            1, 1, 0, 0, 0,
            0, 1, 1, 0, 0,
            0, 0, 1, 1, 0,
        ].into(), (4, 5));

        assert_eq!(
            TileMapView::<AxisY>::view(&map, 0, 0..5).collect::<BitVec>(),
            bitarray!([1, 1, 0, 0]).as_bitslice()
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 1, 0..5).collect::<BitVec>(),
            bitarray!([0, 1, 1, 0]).as_bitslice()
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 2, 0..5).collect::<BitVec>(),
            bitarray!([0, 0, 1, 1]).as_bitslice()
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 3, 0..5).collect::<BitVec>(),
            bitarray!([0, 0, 0, 1]).as_bitslice()
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 4, 0..5).collect::<BitVec>(),
            bitarray!([0, 0, 0, 0]).as_bitslice()
        );
    }
}
