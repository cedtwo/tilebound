use std::marker::PhantomData;
use std::ops::{Deref, Range};

use marker_value::MarkerValue;
use subbyte_index::{BitOrd, BitWidth, Lsb, SubByteIndex};

use crate::plane::axis::{Axis, AxisX, AxisY};
use crate::view::tilemap::{TileMap, TileMapView};

/// # PackMap
///
/// A [`TileMap`] where each tile is represented by packed bits of a given width `W`.
///
/// This type supports any type representable as a byte slice where each element is a contiguous
/// two/four bit nibble. `PackMap` uses [`subbyte_index`] for indexing operations and can use that
/// library to pack elements into an array prior to passing it to this type. Below is a simple
/// example of instantiating a packed array and passing it to `PackMap`.
/// ```
/// # use subbyte_index::*;
/// # use tilebound::prelude::*;
/// # use tilebound::view::tilemap::{PackMap, TileMap, TileMapView};
/// // Bit width = 4.
/// type W = W4;
/// // 25 elements total.
/// const X: usize = 5;
/// const Y: usize = 5;
/// // 25 packed elements, 13 bytes (with the last byte containing only one element).
/// const EL_LEN: usize = X * Y;
/// const BYTE_LEN: usize = EL_LEN.div_ceil(W::N_SLOTS) as usize;
///
/// let mut array = [0u8; BYTE_LEN];
/// // Pack our 4 bit elements into the array. This method must specify an exact range otherwise the
/// // method will expect an input of 26 elements (13 **full** bytes).
/// array.subbyte_index_mut::<W, Lsb, Lsb>(..EL_LEN, [
///      0,  1,  2,  3,  4,
///      5,  6,  7,  8,  9,
///     10, 11, 12, 13, 14,
///     15,  0,  1,  2,  3,
///      4,  5,  6,  7,  8,
/// ]);
///
/// let packmap: PackMap<W, X, Y> = PackMap::new(&array);
/// assert_eq!(TileMapView::<AxisY>::view(&packmap, 1, 0..5).collect::<Vec<_>>(), [1, 6, 11, 0, 5]);
/// ```
///
/// `PackMap` requires specifying the number of rows and columns as generic arguments (`X` and `Y`
/// respectively).
pub struct PackMap<'a, W, const X: usize, const Y: usize, AOrd = Lsb, ElOrd = Lsb> {
    /// The packed array.
    array: &'a [u8],
    /// Array [`BitOrd`].
    _arr: PhantomData<AOrd>,
    /// Element type, width and [`BitOrd`].
    _el: PhantomData<(W, ElOrd)>,
}

impl<'a, W: BitWidth, const X: usize, const Y: usize, AOrd, ElOrd>
    PackMap<'a, W, X, Y, AOrd, ElOrd>
{
    /// Create a new `PackMap` from the given slice.
    pub fn new(array: &'a [u8]) -> Self {
        debug_assert!(array.len() * W::N_SLOTS >= X * Y);
        Self {
            array,
            _arr: PhantomData,
            _el: PhantomData,
        }
    }
}

impl<'a, W, const X: usize, const Y: usize, AOrd, ElOrd> TileMap
    for PackMap<'a, W, X, Y, AOrd, ElOrd>
where
    W: BitWidth,
    AOrd: BitOrd<W>,
    ElOrd: BitOrd<W>,
{
    type El = u8;

    fn axis_len<A: Axis>(&self) -> usize {
        match A::VALUE {
            AxisX::VALUE => X,
            AxisY::VALUE => Y,
            _ => unreachable!(),
        }
    }
}

impl<'a, W, const X: usize, const Y: usize, AOrd, ElOrd> TileMapView<AxisX>
    for PackMap<'a, W, X, Y, AOrd, ElOrd>
where
    W: BitWidth,
    AOrd: BitOrd<W>,
    ElOrd: BitOrd<W>,
{
    type View<'b>
        = PackRow<'b, W, X, AOrd, ElOrd>
    where
        Self: 'b;

    fn view(&self, row_idx: usize, range: Range<usize>) -> Self::View<'_> {
        PackRow {
            slice: self.array,
            row_idx,
            range,
            _arr: PhantomData,
            _el: PhantomData,
        }
    }
}

impl<'a, W, const X: usize, const Y: usize, AOrd, ElOrd> TileMapView<AxisY>
    for PackMap<'a, W, X, Y, AOrd, ElOrd>
where
    W: BitWidth,
    AOrd: BitOrd<W>,
    ElOrd: BitOrd<W>,
{
    type View<'b>
        = PackCol<'b, W, X, AOrd, ElOrd>
    where
        Self: 'b;

    fn view(&self, col_idx: usize, range: Range<usize>) -> Self::View<'_> {
        PackCol {
            slice: self.array,
            col_idx,
            range,
            _arr: PhantomData,
            _el: PhantomData,
        }
    }
}

// impl_dynamic!(['b, W: BitWidth, const X: usize, const Y: usize, AOrd: BitOrd<W>, ElOrd: BitOrd<W>], PackMap<'b, W, X, Y, AOrd, ElOrd>);

/// A row element iterator for a [`BitMap`].
pub struct PackRow<'a, W, const X: usize, AOrd = Lsb, ElOrd = Lsb> {
    /// A reference to the array as a slice.
    slice: &'a [u8],
    /// The row index.
    row_idx: usize,
    /// The row element range.
    range: Range<usize>,
    /// Array [`BitOrd`].
    _arr: PhantomData<AOrd>,
    /// Element type, width and [`BitOrd`].
    _el: PhantomData<(W, ElOrd)>,
}

impl<'a, W: BitWidth, const X: usize, AOrd: BitOrd<W>, ElOrd: BitOrd<W>> Iterator
    for PackRow<'a, W, X, AOrd, ElOrd>
{
    type Item = PackByte;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|i| {
            self.slice
                .subbyte_index::<W, AOrd, ElOrd>(self.row_idx * X + i)
                .into()
        })
    }
}

/// A column element iterator for a [`BitMap`].
pub struct PackCol<'a, W, const X: usize, AOrd = Lsb, ElOrd = Lsb> {
    /// A reference to the array as a slice.
    slice: &'a [u8],
    /// The column index.
    col_idx: usize,
    /// The row element range.
    range: Range<usize>,
    /// Array [`BitOrd`].
    _arr: PhantomData<AOrd>,
    /// Element type, width and [`BitOrd`].
    _el: PhantomData<(W, ElOrd)>,
}

impl<'a, W: BitWidth, const X: usize, AOrd: BitOrd<W>, ElOrd: BitOrd<W>> Iterator
    for PackCol<'a, W, X, AOrd, ElOrd>
{
    type Item = PackByte;

    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|i| {
            self.slice
                .subbyte_index::<W, AOrd, ElOrd>(i * X + self.col_idx)
                .into()
        })
    }
}

/// A byte wrapper. Notably implements `Deref<Target = u8>` to adhere to the [`TileMapView::View`]
/// element bound.
#[derive(Debug, Copy, Clone, Eq, PartialOrd, Ord)]
pub struct PackByte(u8);

impl PartialEq<PackByte> for PackByte {
    fn eq(&self, other: &PackByte) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<u8> for PackByte {
    fn eq(&self, other: &u8) -> bool {
        self.0 == *other
    }
}

impl From<u8> for PackByte {
    fn from(value: u8) -> Self {
        PackByte(value)
    }
}

impl Deref for PackByte {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {

    use subbyte_index::*;

    use super::*;

    type W = W4;
    const X: usize = 5;
    const Y: usize = 5;
    const LEN: usize = X * Y;
    const BYTE_LEN: usize = LEN.div_ceil(W::N_SLOTS) as usize;

    #[test]
    fn axis_size() {
        let map = PackMap::<W, 4, 3>::new(&[0; (4 * 3) / W::N_SLOTS]);

        assert_eq!(map.axis_len::<AxisX>(), 4);
        assert_eq!(map.axis_len::<AxisY>(), 3);
    }

    #[test]
    fn slice_row() {
        #[rustfmt::skip]
        let mut array = [0u8; BYTE_LEN];
        array.subbyte_index_mut::<W, Lsb, Lsb>(..LEN, (0..(X * Y) as u8).map(|int| int % 16));

        let map = PackMap::<W, X, Y>::new(&array);

        assert_eq!(
            TileMapView::<AxisX>::view(&map, 0, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [0, 1, 2, 3, 4]
        );
        assert_eq!(
            TileMapView::<AxisX>::view(&map, 1, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [5, 6, 7, 8, 9]
        );
        assert_eq!(
            TileMapView::<AxisX>::view(&map, 2, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [10, 11, 12, 13, 14]
        );
        assert_eq!(
            TileMapView::<AxisX>::view(&map, 3, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [15, 0, 1, 2, 3]
        );
        assert_eq!(
            TileMapView::<AxisX>::view(&map, 4, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [4, 5, 6, 7, 8]
        );
    }

    #[test]
    fn slice_col() {
        #[rustfmt::skip]
        let mut array = [0u8; BYTE_LEN];
        array.subbyte_index_mut::<W, Lsb, Lsb>(..LEN, (0..(X * Y) as u8).map(|int| int % 16));

        let map = PackMap::<W, X, Y>::new(&array);

        assert_eq!(
            TileMapView::<AxisY>::view(&map, 0, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [0, 5, 10, 15, 4]
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 1, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [1, 6, 11, 0, 5]
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 2, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [2, 7, 12, 1, 6]
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 3, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [3, 8, 13, 2, 7]
        );
        assert_eq!(
            TileMapView::<AxisY>::view(&map, 4, 0..5)
                .map(|el| *el)
                .collect::<Vec<_>>(),
            [4, 9, 14, 3, 8]
        );
    }
}
