//! Types for collision representation.
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{ControlFlow, Range};

use bitvec::prelude::*;
use tilebound::plane::axis::{Axis, AxisVec};
use tilebound::plane::endpoint::{Endpoint, EndpointBound, EndpointPair};
use tilebound::view::index::SliceIndex;
use tilebound::view::inspect::TileMapInspect;
use tilebound::view::tilemap::TileMapView;

use crate::tile::{VertexEdge, VertexTile};
use crate::vertex_mask::VertexMask;

/// # Collision
///
/// Collision data and context for displacement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Collision<A: Axis> {
    /// Collided with a tile bound. Returns an [`EdgeRange`] on the **transpose** axis of displacement.
    TileBound(EdgeRange<A>),
    /// Collided with a map bound.
    MapBound,
    /// Collided with a triangle hypotenuse.
    TriangleHypotenuse,
    /// Collided with the narrowest extreme of a triangle and a tile/map bound.
    TriangleApex,
    /// Collided at the narrowest extreme of two adjacent triangles.
    WedgeApex,
}

impl<A: Axis> From<Collision<A>> for ControlFlow<Collision<A>> {
    fn from(c: Collision<A>) -> Self {
        ControlFlow::Break(c)
    }
}

/// # EdgeRange
///
/// A collection of adjacent edge vertices on a tile bound.
///
/// `EdgeRange` is an aggregate of adjacent tile vertices for one endpoint of a tile slice. It
/// notably exposes methods for iteration over edges.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeRange<A: Axis> {
    /// The edge range axis.
    pub axis: PhantomData<A>,
    /// The index of the first tile.
    pub start_idx: AxisVec<i32>,
    /// An [`AxisMask`] indicating if part of the inspected slice was out-of-bounds.
    pub padding: EndpointPair<A, i32>,
    /// The tile bound [`Endpoint`] on the transpose axis.
    pub t_end: Endpoint<A::T>,
    /// The backing [`BitVec`]. Stores each vertex as a two bit pair.
    pub(crate) array: BitVec,
}

impl<A: Axis> EdgeRange<A> {
    /// Create a new `EdgeRange` from a slice index, a range and the transpose endpoint to aggregate
    /// vertices.
    ///
    /// ## Example
    ///
    /// ```
    /// # use tilebound_halfslope::prelude::*;
    /// // A range of elements on `AxisY` (a column).
    /// EdgeRange::<AxisY>::new(
    ///     0,                // The column index.
    ///     1..5,             // The range of elements.
    ///     Endpoint::LEFT,   // Aggregate vertices on the left.
    /// );
    /// ```
    pub fn new(t_index: i32, range: Range<i32>, t_end: Endpoint<A::T>) -> Self {
        debug_assert!(t_index >= 0);
        debug_assert!(range.start >= 0);
        Self {
            axis: PhantomData,
            start_idx: AxisVec::new_mapped::<A>(range.start, t_index),
            t_end,
            padding: EndpointPair::new(0, 0),
            array: BitVec::repeat(false, range.len() * 2),
        }
    }

    /// Create a new `EdgeRange` from a starting index, the transpose endpoint to aggregate
    /// verteices, and a [`BitVec`] array.
    pub fn from_array(
        index: impl Into<AxisVec<i32>>,
        t_end: Endpoint<A::T>,
        array: BitVec,
    ) -> Self {
        Self {
            axis: PhantomData,
            t_end,
            start_idx: index.into(),
            array,
            padding: EndpointPair::new(0, 0),
        }
    }

    /// Create a new `EdgeRange` from a [`SliceIndex`].
    pub fn from_index(index: &SliceIndex<A>) -> Self {
        Self {
            axis: PhantomData,
            t_end: index.end,
            start_idx: AxisVec::new_mapped::<A>(index.inbound_range_start(), index.t_index as i32),
            array: BitVec::repeat(false, index.len_cropped() * 2),
            padding: index.padding(),
        }
    }

    /// Return the vertex [`Endpoint`] for elements in the array.
    pub fn end(&self) -> Endpoint<A::T> {
        self.t_end
    }

    /// Set the bits of the given `mask`, offset at index `i`.
    #[inline]
    pub(crate) fn insert_mask(&mut self, i: usize, mask: u8) {
        self.array[i..] |= mask.view_bits::<Lsb0>();
    }

    /// Get a intersecting [`VertexMask`]. This is not intended to be used where [`Self::inner_collision`]
    /// returns `true`.
    ///
    /// Note that the returned [`VertexMask`] will only ever
    /// contain (at most) the two bits of
    /// the view axis and endpoint. Therefore [`VertexMask::isolate_axis`],
    /// [`AxisMask::end_is_set`] and [`AxisMask::count_ones`](u8::count_ones) can be used to
    /// make assertions against the bits with minimum overhead.
    pub(crate) fn trimask(&self) -> VertexMask {
        assert!(!self.inner_collision());
        match (
            self.array.starts_with(bits![static usize, Lsb0; 1, 0]),
            self.array.ends_with(bits![static usize, Lsb0; 0, 1]),
        ) {
            (true, true) => VertexMask::from_axis(!self.t_end),
            (true, false) => VertexMask::from_vertex(!self.t_end, Endpoint::LOWER),
            (false, true) => VertexMask::from_vertex(!self.t_end, Endpoint::UPPER),
            (false, false) => VertexMask::NONE,
        }
        .into()
    }

    /// Return the number of **tiles** in the array.
    pub(crate) fn len(&self) -> usize {
        self.array.len() / 2
    }

    /// Returns `true` if any non-terminating triangle tile vertex is set.
    pub(crate) fn inner_collision(&self) -> bool {
        match self.array.len() {
            #[cfg(debug_assertions)]
            0 | 1 => {
                unreachable!("Expected an array of >= 2 elements")
            }
            2 => self.array.all(),
            _ => {
                let len = self.array.len();
                self.array[1..len - 1].any()
            }
        }
    }

    /// Returns `true` if there any vertex is set outside of a terminating triangle tile vertex
    /// after removing `n` **tiles** from the given [`Endpoint`].
    pub(crate) fn truncated_inner_collision(&self, end: Endpoint<A>, n: usize) -> bool {
        let len = self.array.len();
        let n = n * 2;
        if len <= n + 2 {
            false
        } else {
            match *end {
                EndpointBound::Lower => self.array[n + 1..len - 1].any(),
                EndpointBound::Upper => self.array[1..self.len() - (n + 1)].any(),
            }
        }
    }

    /// Return an iterator of `(AxisVec<i32>, VertexEdge)` tuples where the left element is the tile
    /// index and the right element is the edge bound on the transpose axis. Returns all element
    /// indices regardless of being in/out of bounds.
    pub fn iter_all(&self) -> impl Iterator<Item = (AxisVec<i32>, VertexEdge)> {
        let lower_pad = (1..=self.padding.lower()).into_iter().rev().map(|pad| {
            (
                self.start_idx.map_axis::<A, _>(|idx| idx - pad),
                VertexEdge::None,
            )
        });
        let upper_pad = (self.array.len() as i32..(self.array.len() as i32 + self.padding.upper()))
            .into_iter()
            .map(|pad| {
                (
                    self.start_idx.map_axis::<A, _>(|idx| idx + pad),
                    VertexEdge::None,
                )
            });
        lower_pad.chain(self.iter_in_bounds()).chain(upper_pad)
    }

    /// Return an iterator of `(AxisVec<i32>, VertexEdge)` tuples where the left element is the tile
    /// index and the right element is the edge bound on the transpose axis. Returns only the
    /// elements that are within the map bounds.
    pub fn iter_in_bounds(&self) -> impl Iterator<Item = (AxisVec<i32>, VertexEdge)> {
        let base_index = self.start_idx.map(|idx| idx as i32);
        self.array
            .chunks_exact(2)
            .enumerate()
            .map(move |(offs, slice)| {
                (
                    base_index.map_axis::<A, _>(|idx| idx + offs as i32),
                    slice.load::<u8>().into(),
                )
            })
    }
}

#[allow(unused)]
#[cfg(debug_assertions)]
impl<A: Axis> EdgeRange<A> {
    /// Return the `EdgeRange` with the given `padding` field set.
    pub(crate) fn with_padding(mut self, padding: EndpointPair<A, i32>) -> Self {
        self.padding = padding;
        self
    }

    /// Create a new `EdgeRange` from a slice index, a range and the transpose endpoint. Sets the
    /// vertex on the given `end` extreme.
    pub fn extreme(
        t_index: i32,
        range: Range<i32>,
        end: Endpoint<A>,
        t_end: Endpoint<A::T>,
    ) -> Self {
        let mut edges = Self::new(t_index, range, t_end);

        match *end {
            EndpointBound::Lower => edges.set_lower_vertex(),
            EndpointBound::Upper => edges.set_upper_vertex(),
        }
        edges
    }

    /// Create a new `EdgeRange` from a slice index, a range and the transpose endpoint to aggregate
    /// vertices. Sets both vertex extremes on axis `A`.
    pub fn extremes(t_index: i32, range: Range<i32>, t_end: Endpoint<A::T>) -> Self {
        let mut edges = Self::new(t_index, range, t_end);

        edges.set_lower_vertex();
        edges.set_upper_vertex();

        edges
    }

    pub(crate) fn set_lower_vertex(&mut self) {
        self.array.set(0, true);
    }

    pub(crate) fn set_upper_vertex(&mut self) {
        let index = self.array.len() - 1;
        self.array.set(index, true);
    }
}

impl<A: Axis, Map: TileMapView<A, El: VertexTile>> TileMapInspect<A, Map> for EdgeRange<A> {
    fn inspect(map: &Map, index: &SliceIndex<A>) -> Self {
        let sign = index.end;

        let mut edges = EdgeRange::from_index(index);
        let view = map.view(index.t_index, index.range_cropped());

        let mut i = 0;
        view.into_iter().for_each(|mask| {
            let mask: VertexMask = (*mask).mask();
            let is_oneway = mask.count_ones() == 2;
            let compact = mask.compact::<A::T>(sign);
            // Filter out oneway tiles if accessed from the side.
            if !is_oneway || compact.count_ones() == 2 {
                edges.insert_mask(i, compact);
            }
            i += 2;
        });
        edges
    }
}

#[cfg(all(test, feature = "arraymap"))]
mod tests {

    use super::*;
    use tilebound::prelude::*;

    #[rustfmt::skip]
    const TRI: ArrayMap<3, 1, 3, u8> = ArrayMap::new_unchecked([
        00, 07, 00, // Top-left triangle.
    ]);

    #[rustfmt::skip]
    const COLUMNS: ArrayMap<4, 2, { 4 * 2 }, u8> = ArrayMap::new_unchecked([
        07, 13,
        00, 15,
        00, 00,
        11, 14,
    ]);

    #[test]
    fn sets_single_vertex() {
        let index = SliceIndex::<AxisY>::new_unchecked(1, 0..1, 1, Endpoint::UPPER);
        let edges = EdgeRange::inspect(&TRI, &index);

        assert_eq!(edges.array, bits![1, 0]);
    }

    #[test]
    fn set_edge_vertices() {
        let index = SliceIndex::<AxisY>::new_unchecked(1, 0..1, 1, Endpoint::LOWER);
        let edges = EdgeRange::inspect(&TRI, &index);

        assert_eq!(edges.array, bits![1, 1]);
    }

    #[test]
    fn lower_padding_is_cropped() {
        let index = SliceIndex::<AxisY>::new_unchecked(1, -1..1, 1, Endpoint::LOWER);
        let edges = EdgeRange::inspect(&TRI, &index);

        assert_eq!(edges.array, bits![1, 1]);
        assert_eq!(edges.padding, EndpointPair::new(1, 0));
    }

    #[test]
    fn upper_padding_is_cropped() {
        let index = SliceIndex::<AxisY>::new_unchecked(1, 0..2, 1, Endpoint::UPPER);
        let edges = EdgeRange::inspect(&TRI, &index);

        assert_eq!(edges.array, bits![1, 0]);
        assert_eq!(edges.padding, EndpointPair::new(0, 1));
    }

    #[test]
    fn iter_in_bounds_col() {
        let idx_0 = SliceIndex::<AxisY>::new_unchecked(0, -2..5, 4, Endpoint::LOWER);
        let idx_1 = SliceIndex::<AxisY>::new_unchecked(0, -2..5, 4, Endpoint::UPPER);
        let idx_2 = SliceIndex::<AxisY>::new_unchecked(1, 1..10, 4, Endpoint::LOWER);

        let c0 = EdgeRange::inspect(&COLUMNS, &idx_0);
        let c0 = c0.iter_in_bounds().collect::<Vec<_>>();
        let c1 = EdgeRange::inspect(&COLUMNS, &idx_1);
        let c1 = c1.iter_in_bounds().collect::<Vec<_>>();
        let c2 = EdgeRange::inspect(&COLUMNS, &idx_2);
        let c2 = c2.iter_in_bounds().collect::<Vec<_>>();

        assert_eq!(
            c0,
            vec![
                ((0, 0).into(), VertexEdge::All),
                ((0, 1).into(), VertexEdge::None),
                ((0, 2).into(), VertexEdge::None),
                ((0, 3).into(), VertexEdge::All)
            ]
        );
        assert_eq!(
            c1,
            vec![
                ((0, 0).into(), VertexEdge::Lower),
                ((0, 1).into(), VertexEdge::None),
                ((0, 2).into(), VertexEdge::None),
                ((0, 3).into(), VertexEdge::Upper)
            ]
        );
        assert_eq!(
            c2,
            vec![
                ((1, 1).into(), VertexEdge::All),
                ((1, 2).into(), VertexEdge::None),
                ((1, 3).into(), VertexEdge::Upper)
            ]
        );
    }

    #[test]
    fn iter_in_bounds_row() {
        let idx_0 = SliceIndex::<AxisX>::new_unchecked(0, -2..5, 2, Endpoint::LOWER);
        let idx_1 = SliceIndex::<AxisX>::new_unchecked(0, -2..5, 2, Endpoint::UPPER);
        let idx_2 = SliceIndex::<AxisX>::new_unchecked(3, 1..2, 2, Endpoint::LOWER);

        let r0 = EdgeRange::inspect(&COLUMNS, &idx_0);
        let r0 = r0.iter_in_bounds().collect::<Vec<_>>();
        let r1 = EdgeRange::inspect(&COLUMNS, &idx_1);
        let r1 = r1.iter_in_bounds().collect::<Vec<_>>();
        let r2 = EdgeRange::inspect(&COLUMNS, &idx_2);
        let r2 = r2.iter_in_bounds().collect::<Vec<_>>();

        assert_eq!(
            r0,
            vec![
                ((0, 0).into(), VertexEdge::All),
                ((1, 0).into(), VertexEdge::All)
            ]
        );
        assert_eq!(
            r1,
            vec![
                ((0, 0).into(), VertexEdge::Lower),
                ((1, 0).into(), VertexEdge::Upper)
            ]
        );
        assert_eq!(r2, vec![((1, 3).into(), VertexEdge::Upper)]);
    }
}
