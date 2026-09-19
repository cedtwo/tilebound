use std::marker::PhantomData;
use std::ops::{ControlFlow, Range};

use bitvec::prelude::*;
use tilebound::plane::axis::AxisVec;
use tilebound::plane::endpoint::EndpointPair;
use tilebound::prelude::*;
use tilebound::view::index::SliceIndex;
use tilebound::view::inspect::TileMapInspect;

use crate::tile::SolidTile;

/// # Collision
///
/// Provides context for the [`tilebound::Break::Collision`] variant.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Collision<A> {
    /// Collided on a tile bound.
    TileBound(TileRange<A>),
    /// Collided on a map bound.
    MapBound,
}

impl<A> From<Collision<A>> for ControlFlow<Break<Collision<A>>> {
    fn from(c: Collision<A>) -> Self {
        ControlFlow::Break(Break::Collision(c))
    }
}

/// # TileRange
///
/// A collection of (at least partially) in-bound tiles in a tilemap slice.
///
/// `EdgeRange` is an aggregate of tiles for a tile slice. It notably exposes methods for iteration
/// over solid tiles and their respective indices.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TileRange<A> {
    /// The tile range axis.
    pub axis: PhantomData<A>,
    /// The index of the first tile.
    pub start_idx: AxisVec<i32>,
    /// The number of out of bound tiles on each endpoint of axis `A`.
    pub padding: EndpointPair<i32>,
    /// The backing [`BitVec`]. Stores each solid tile as `1`.
    pub(super) array: BitVec,
}

impl<A: Axis> TileRange<A> {
    /// Create a new `TileRange` from a slice index, a range and padding (out-of-bound len on each
    /// endpoint).
    ///
    /// ## Example
    ///
    /// ```
    /// # use tilebound_solid::prelude::*;
    /// // A range of elements on `AxisY` (a column).
    /// TileRange::<AxisY>::new(
    ///     0,                // The column index.
    ///     1..5,             // The range of elements.
    /// );
    /// ```
    pub fn new(t_index: i32, range: Range<i32>) -> Self {
        Self {
            axis: PhantomData,
            start_idx: AxisVec::new_mapped::<A>(range.start, t_index),
            padding: EndpointPair::new(0, 0),
            array: BitVec::repeat(false, range.len()),
        }
    }

    /// Create a new `TileRange` from a starting index and a [`BitVec`] array.
    pub fn from_array(index: impl Into<AxisVec<i32>>, array: BitVec) -> Self {
        Self {
            axis: PhantomData,
            start_idx: index.into(),
            array,
            padding: EndpointPair::new(0, 0),
        }
    }

    /// Create a new `TileRange` from a [`SliceIndex`].
    pub fn from_index(index: &SliceIndex<A>) -> Self {
        Self {
            axis: PhantomData,
            start_idx: AxisVec::new_mapped::<A>(index.inbound_range_start(), index.t_index as i32),
            array: BitVec::with_capacity(index.len_cropped()),
            padding: index.padding(),
        }
    }

    /// Returns `true` if any tile is solid.
    pub fn collided(&self) -> bool {
        self.array.any()
    }

    /// Return an iterator of `(AxisVec<i32>, bool)` tuples where the left element is the tile index
    /// and the right element is `true` if the tile is solid. Returns all element indices regardless
    /// of being in/out of bounds.
    pub fn iter_all(&self) -> impl Iterator<Item = (AxisVec<i32>, bool)> {
        let lower_pad = (1..=self.padding.lower())
            .into_iter()
            .rev()
            .map(|pad| (self.start_idx.map_axis::<A, _>(|idx| idx - pad), false));
        let upper_pad = (self.array.len() as i32..(self.array.len() as i32 + self.padding.upper()))
            .into_iter()
            .map(|pad| (self.start_idx.map_axis::<A, _>(|idx| idx + pad), false));
        lower_pad.chain(self.iter_in_bounds()).chain(upper_pad)
    }

    /// Return an iterator of `(AxisVec<i32>, bool)` tuples where the left element is the tile index
    /// and the right element is `true` if the tile is solid. Returns only the elements that are
    /// within the map bounds.
    pub fn iter_in_bounds(&self) -> impl Iterator<Item = (AxisVec<i32>, bool)> {
        self.array.iter().enumerate().map(|(offs, is_solid)| {
            (
                self.start_idx.map_axis::<A, _>(|idx| idx + offs as i32),
                *is_solid,
            )
        })
    }

    /// Return an iterator of `AxisVec<i32>` tile indices for only the solid tiles.
    pub fn iter_solid(&self) -> impl Iterator<Item = AxisVec<i32>> {
        self.array
            .iter_ones()
            .map(|offs| self.start_idx.map_axis::<A, _>(|idx| idx + offs as i32))
    }
}

impl<A: Axis, Map: TileMapView<A, El: SolidTile>> TileMapInspect<A, Map> for TileRange<A> {
    fn inspect(map: &Map, index: &SliceIndex<A>) -> Self {
        let mut tiles = TileRange::from_index(index);
        tiles.array.extend(
            map.view(index.t_index, index.range_cropped())
                .into_iter()
                .map(|tile| tile.is_solid()),
        );

        tiles
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[rustfmt::skip]
    const MAP: ArrayMap<3, 3, { 3 * 3 }, u8> = ArrayMap::new_unchecked([
        1, 1, 1,
        0, 1, 0,
        1, 0, 0,
    ]);

    #[test]
    fn iter_in_bounds_col() {
        let idx_0 = SliceIndex::<AxisY>::new_unchecked(0, -2..5, 3, Endpoint::Lower);
        let idx_1 = SliceIndex::<AxisY>::new_unchecked(1, -2..5, 3, Endpoint::Lower);
        let idx_2 = SliceIndex::<AxisY>::new_unchecked(2, -2..5, 3, Endpoint::Lower);

        let c0 = TileRange::inspect(&MAP, &idx_0);
        let c0 = c0.iter_in_bounds().collect::<Vec<_>>();
        let c1 = TileRange::inspect(&MAP, &idx_1);
        let c1 = c1.iter_in_bounds().collect::<Vec<_>>();
        let c2 = TileRange::inspect(&MAP, &idx_2);
        let c2 = c2.iter_in_bounds().collect::<Vec<_>>();

        #[rustfmt::skip]
        assert_eq!(c0, vec![((0, 0).into(), true), ((0, 1).into(), false), ((0, 2).into(), true)]);
        #[rustfmt::skip]
        assert_eq!(c1, vec![((1, 0).into(), true), ((1, 1).into(), true), ((1, 2).into(), false)]);
        #[rustfmt::skip]
        assert_eq!(c2, vec![((2, 0).into(), true), ((2, 1).into(), false), ((2, 2).into(), false)]);
    }

    #[test]
    fn iter_in_bounds_row() {
        let idx_0 = SliceIndex::<AxisX>::new_unchecked(0, -2..5, 3, Endpoint::Lower);
        let idx_1 = SliceIndex::<AxisX>::new_unchecked(1, -2..5, 3, Endpoint::Lower);
        let idx_2 = SliceIndex::<AxisX>::new_unchecked(2, -2..5, 3, Endpoint::Lower);

        let r0 = TileRange::inspect(&MAP, &idx_0);
        let r0 = r0.iter_in_bounds().collect::<Vec<_>>();
        let r1 = TileRange::inspect(&MAP, &idx_1);
        let r1 = r1.iter_in_bounds().collect::<Vec<_>>();
        let r2 = TileRange::inspect(&MAP, &idx_2);
        let r2 = r2.iter_in_bounds().collect::<Vec<_>>();

        #[rustfmt::skip]
        assert_eq!(r0, vec![((0, 0).into(), true), ((1, 0).into(), true), ((2, 0).into(), true)]);
        #[rustfmt::skip]
        assert_eq!(r1, vec![((0, 1).into(), false), ((1, 1).into(), true), ((2, 1).into(), false)]);
        #[rustfmt::skip]
        assert_eq!(r2, vec![((0, 2).into(), true), ((1, 2).into(), false), ((2, 2).into(), false)]);
    }

    #[test]
    fn iter_all_col() {
        let idx_0 = SliceIndex::<AxisY>::new_unchecked(0, -2..5, 3, Endpoint::Lower);
        let idx_1 = SliceIndex::<AxisY>::new_unchecked(1, -2..5, 3, Endpoint::Lower);
        let idx_2 = SliceIndex::<AxisY>::new_unchecked(2, -2..5, 3, Endpoint::Lower);

        let c0 = TileRange::inspect(&MAP, &idx_0);
        let c0 = c0.iter_all().collect::<Vec<_>>();
        let c1 = TileRange::inspect(&MAP, &idx_1);
        let c1 = c1.iter_all().collect::<Vec<_>>();
        let c2 = TileRange::inspect(&MAP, &idx_2);
        let c2 = c2.iter_all().collect::<Vec<_>>();

        #[rustfmt::skip]
        assert_eq!(c0, vec![((0, -2).into(), false), ((0, -1).into(), false), ((0, 0).into(), true), ((0, 1).into(), false), ((0, 2).into(), true), ((0, 3).into(), false), ((0, 4).into(), false)]);
        #[rustfmt::skip]
        assert_eq!(c1, vec![((1, -2).into(), false), ((1, -1).into(), false), ((1, 0).into(), true), ((1, 1).into(), true), ((1, 2).into(), false), ((1, 3).into(), false), ((1, 4).into(), false)]);
        #[rustfmt::skip]
        assert_eq!(c2, vec![((2, -2).into(), false), ((2, -1).into(), false), ((2, 0).into(), true), ((2, 1).into(), false), ((2, 2).into(), false), ((2, 3).into(), false), ((2, 4).into(), false)]);
    }

    #[test]
    fn iter_all_row() {
        let idx_0 = SliceIndex::<AxisX>::new_unchecked(0, -2..5, 3, Endpoint::Lower);
        let idx_1 = SliceIndex::<AxisX>::new_unchecked(1, -2..5, 3, Endpoint::Lower);
        let idx_2 = SliceIndex::<AxisX>::new_unchecked(2, -2..5, 3, Endpoint::Lower);

        let r0 = TileRange::inspect(&MAP, &idx_0);
        let r0 = r0.iter_all().collect::<Vec<_>>();
        let r1 = TileRange::inspect(&MAP, &idx_1);
        let r1 = r1.iter_all().collect::<Vec<_>>();
        let r2 = TileRange::inspect(&MAP, &idx_2);
        let r2 = r2.iter_all().collect::<Vec<_>>();

        #[rustfmt::skip]
        assert_eq!(r0, vec![((-2, 0).into(), false), ((-1, 0).into(), false), ((0, 0).into(), true), ((1, 0).into(), true), ((2, 0).into(), true), ((3, 0).into(), false), ((4, 0).into(), false)]);
        #[rustfmt::skip]
        assert_eq!(r1, vec![((-2, 1).into(), false), ((-1, 1).into(), false), ((0, 1).into(), false), ((1, 1).into(), true), ((2, 1).into(), false), ((3, 1).into(), false), ((4, 1).into(), false)]);
        #[rustfmt::skip]
        assert_eq!(r2, vec![((-2, 2).into(), false), ((-1, 2).into(), false), ((0, 2).into(), true), ((1, 2).into(), false), ((2, 2).into(), false), ((3, 2).into(), false), ((4, 2).into(), false)]);
    }

    #[test]
    fn iter_solid_col() {
        let idx_0 = SliceIndex::<AxisY>::new_unchecked(0, -2..5, 3, Endpoint::Lower);
        let idx_1 = SliceIndex::<AxisY>::new_unchecked(1, -2..5, 3, Endpoint::Lower);
        let idx_2 = SliceIndex::<AxisY>::new_unchecked(2, -2..5, 3, Endpoint::Lower);

        let c0 = TileRange::inspect(&MAP, &idx_0);
        let c0 = c0.iter_solid().collect::<Vec<_>>();
        let c1 = TileRange::inspect(&MAP, &idx_1);
        let c1 = c1.iter_solid().collect::<Vec<_>>();
        let c2 = TileRange::inspect(&MAP, &idx_2);
        let c2 = c2.iter_solid().collect::<Vec<_>>();

        assert_eq!(c0, vec![(0, 0).into(), (0, 2).into()]);
        assert_eq!(c1, vec![(1, 0).into(), (1, 1).into()]);
        assert_eq!(c2, vec![(2, 0).into()]);
    }

    #[test]
    fn iter_solid_row() {
        let idx_0 = SliceIndex::<AxisX>::new_unchecked(0, -2..5, 3, Endpoint::Lower);
        let idx_1 = SliceIndex::<AxisX>::new_unchecked(1, -2..5, 3, Endpoint::Lower);
        let idx_2 = SliceIndex::<AxisX>::new_unchecked(2, -2..5, 3, Endpoint::Lower);

        let r0 = TileRange::inspect(&MAP, &idx_0);
        let r0 = r0.iter_solid().collect::<Vec<_>>();
        let r1 = TileRange::inspect(&MAP, &idx_1);
        let r1 = r1.iter_solid().collect::<Vec<_>>();
        let r2 = TileRange::inspect(&MAP, &idx_2);
        let r2 = r2.iter_solid().collect::<Vec<_>>();

        assert_eq!(r0, vec![(0, 0).into(), (1, 0).into(), (2, 0).into()]);
        assert_eq!(r1, vec![(1, 1).into()]);
        assert_eq!(r2, vec![(0, 2).into()]);
    }
}
