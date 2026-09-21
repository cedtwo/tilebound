use std::cmp::Ordering;
use std::fmt::Debug;

use crate::ops::{index_to_raw_pos, raw_pos_to_index};
use crate::plane::axis::{Axis, DynAxis};
use crate::plane::endpoint::{Endpoint, EndpointBound};
use crate::plane::scale::Scale;

use crate::topology::edge::Edge;

/// A [`Vertex`] with no axis specified. This should only be used where an [`Axis`] can be inferred
/// and cast by context (eg. in [`AxisVec`](tilebound_plane::axis::AxisVec)).
pub type RawVertex = Vertex<DynAxis>;

/// # Vertex
///
/// A one-dimensional endpoint position.
///
/// `Vertex` differs from an `f32` position in how it handles tile indices. Vertices use their inner
/// [`Endpoint`] to decide which tile is being intersected when *exactly* on a tile bound. This
/// allows two vertices to be positioned at the same point, but occupy different tiles. See also
/// [`VertexOffset`] and [`Edge`].
///
/// ```rust
/// # use tilebound::prelude::*;
/// # use tilebound::topology::vertex::Vertex;
/// type Sc = ConSc<16>;
///
/// // Two vertices sharing the same tile bound at position `16.0`.
/// let tile_0_end = Vertex::from_pos::<Sc>(16.0, Endpoint::RIGHT);
/// let tile_1_start = Vertex::from_pos::<Sc>(16.0, Endpoint::LEFT);
///
/// // Both vertices intersect different tiles.
/// assert_eq!(tile_0_end.index(), 0);
/// assert_eq!(tile_1_start.index(), 1);
/// // Both vertices have an equal position.
/// assert_eq!(tile_0_end.to_pos::<Sc>(), tile_1_start.to_pos::<Sc>());
/// ```
#[derive(Default)]
pub struct Vertex<A> {
    /// The endpoint orientation.
    pub(crate) end: Endpoint<A>,
    /// The intersected tile index.
    pub(crate) index: i32,
    /// The offset within the tile.
    offset: VertexOffset,
}

impl<A> Debug for Vertex<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Vertex")
            .field("end", &self.end)
            .field("index", &self.index)
            .field("offset", &self.offset)
            .finish()
    }
}

impl<A> Clone for Vertex<A> {
    fn clone(&self) -> Self {
        Self {
            end: self.end.clone(),
            index: self.index.clone(),
            offset: self.offset.clone(),
        }
    }
}

impl<A> Copy for Vertex<A> {}

impl<A> Vertex<A> {
    /// Create a new `Vertex` from the given [`Endpoint`] and position. This method will orientate
    /// the variables for the given endpoint resulting in a vertex that may not have the same `index`
    /// that was passed in. This is the most common method used for creating a `Vertex` from
    /// positions on a plane.
    ///
    /// # Example
    /// ```rust
    /// # use tilebound::prelude::*;
    /// # use tilebound::topology::vertex::Vertex;
    /// type Sc = ConSc<16>;
    ///
    /// // A vertex endpoint on the left of tile index `0` (intersects index `-1`).
    /// let vertex_on_bound = Vertex::from_pos::<Sc>(0.0, Endpoint::RIGHT);
    /// assert_eq!(vertex_on_bound.index(), -1);
    /// assert_eq!(vertex_on_bound.offset_value(), 16.0);
    ///
    /// // A vertex endpoint exceeding the leftmost bound of tile index `0` (intersects index `0`).
    /// let vertex_offset = Vertex::from_pos::<Sc>(0.1, Endpoint::RIGHT);
    /// assert_eq!(vertex_offset.index(), 0);
    /// assert_eq!(vertex_offset.offset_value(), 0.1);
    /// ```
    pub fn from_pos<Sc: Scale>(pos: f32, end: Endpoint<A>) -> Self {
        let index = raw_pos_to_index::<Sc>(pos);
        let offset = pos.rem_euclid(Sc::SCALE);

        Self::from_parts::<Sc>(index, offset, end)
    }

    /// Create a new `Vertex` endpoint from an index, offset and endpoint. This method expects a
    /// tile `index`, an `f32` offset of `0.0..Scale::SCALE` for a given [`Scale`], and the
    /// [`Endpoint`]. This method will orientate the variables for the given endpoint resulting in a
    /// vertex that may not have the same `index` that was passed in.
    ///
    /// # Example
    /// ```rust
    /// # use tilebound::prelude::*;
    /// # use tilebound::topology::vertex::Vertex;
    /// type Sc = ConSc<16>;
    ///
    /// let vertex_on_bound = Vertex::from_parts::<Sc>(0, 0.0, Endpoint::RIGHT);
    /// let vertex_offset = Vertex::from_parts::<Sc>(0, 8.0, Endpoint::RIGHT);
    ///
    /// assert_eq!(vertex_on_bound.index(), -1);
    /// assert_eq!(vertex_offset.index(), 0);
    /// ```
    pub fn from_parts<Sc: Scale>(
        index: i32,
        offset: impl Into<VertexOffset>,
        end: Endpoint<A>,
    ) -> Vertex<A> {
        let mut vert = Vertex {
            end,
            index,
            offset: offset.into(),
        };
        vert.align_to_bound::<Sc>();
        vert
    }

    /// Create a new `Vertex` on the given bound. This method expects a tile `index` and the
    /// endpoint [`Endpoint`]. Unlike [`Self::from_parts`], this method maintains the given `index`.
    ///
    /// # Example
    /// ```rust
    /// # use tilebound::prelude::*;
    /// # use tilebound::topology::vertex::Vertex;
    /// type Sc = ConSc<16>;
    ///
    /// let bound_left = Vertex::from_bound::<Sc>(0, Endpoint::RIGHT);
    /// let bound_right = Vertex::from_bound::<Sc>(1, Endpoint::LEFT);
    ///
    /// assert_eq!(bound_left.index(), 0);
    /// assert_eq!(bound_right.index(), 1);
    /// ```
    pub fn from_bound<Sc: Scale>(index: i32, end: Endpoint<A>) -> Self {
        Vertex {
            end,
            index,
            offset: VertexOffset::from_bound::<_, Sc>(end),
        }
    }

    /// Align the index and offset to the tilebound. Ensures that a newly created vertex has an
    /// offset and index relative to the intersecting tile. This only affects vertices on a tile
    /// bound.
    #[inline]
    fn align_to_bound<Sc: Scale>(&mut self) {
        if let Some(f) = self.offset.align_to_bound::<_, Sc>(self.end) {
            f(&mut self.index)
        }
    }

    // Get the index of the intersected tile.
    #[inline]
    pub const fn index(&self) -> i32 {
        self.index
    }

    /// Get the vertex [`Endpoint`].
    #[inline]
    pub const fn endpoint(&self) -> Endpoint<A> {
        self.end
    }

    /// Get the vertex offset.
    #[inline]
    pub const fn offset(&self) -> VertexOffset {
        self.offset
    }

    /// Get the vertex offset as an `f32` value.
    #[inline]
    pub const fn offset_value(&self) -> f32 {
        self.offset.0
    }

    /// Returns `true` if the `Vertex` is on a tile bound.
    #[inline]
    pub fn is_on_bound<Sc: Scale>(&self) -> bool {
        self.offset.is_on_bound::<Sc>()
    }

    /// Get the `offset` normalized to [`Endpoint::Upper`]. This returns a value `<= 0` and `< Sc`
    /// indicating the distance to the next tile bound, regardless of `Endpoint`.
    #[inline]
    pub fn offset_normalized<Sc: Scale>(&self) -> VertexOffset {
        match *self.end {
            EndpointBound::Lower => self.offset.reversed::<Sc>(),
            EndpointBound::Upper => self.offset,
        }
    }

    /// Return the `f32` vertex position. See also [`Edge::pos`].
    pub fn to_pos<Sc: Scale>(&self) -> f32 {
        index_to_raw_pos::<Sc>(self.index) + self.offset_value()
    }

    /// Translate the vertex by the given `delta`.
    #[inline]
    pub fn translate<Sc: Scale>(&mut self, delta: f32) {
        *self = Vertex::from_pos::<Sc>(self.to_pos::<Sc>() + delta, self.end);
    }

    /// Set the `Vertex` to the next outer bound, incrementing the index if already
    /// on a bound.
    #[inline]
    pub fn incr_bound<Sc: Scale>(&mut self) {
        if self.offset.is_on_bound::<Sc>() {
            self.incr_index();
        } else {
            self.set_offset_unchecked(VertexOffset::from_bound::<_, Sc>(self.end));
        }
    }

    /// Set the `Vertex` to the last inner bound, decrementing the index if already
    /// on a bound.
    #[inline]
    pub fn decr_bound<Sc: Scale>(&mut self) {
        if self.offset.is_on_bound::<Sc>() {
            self.decr_index();
        } else {
            self.decr_index();
            self.set_offset_unchecked(VertexOffset::from_bound::<_, Sc>(self.end));
        }
    }

    /// Increment the index toward the inner `Endpoint`.
    pub fn incr_index(&mut self) {
        self.index = self.end.incr(self.index);
    }

    /// Decrement the index away from the inner `Endpoint`.
    pub fn decr_index(&mut self) {
        self.index = self.end.decr(self.index);
    }

    /// Get the adjacent `Vertex`. See also [`Edge`].
    pub fn into_adjacent<Sc: Scale>(self, edge_len: f32) -> Self {
        let mut edge = self.into_edge(edge_len);
        edge.reverse::<Sc>();
        edge.take_vertex()
    }

    /// Get the `Vertex` with the given [`Endpoint`]. See also [`Edge`].
    pub fn into_endpoint<Sc: Scale>(self, end: Endpoint<A>, edge_len: f32) -> Self {
        if end == self.endpoint() {
            self
        } else {
            self.into_adjacent::<Sc>(edge_len)
        }
    }

    /// Create an `Edge` with the vertex as an *inbound* endpoint. Returns an `Edge` of the
    /// given length.
    pub const fn into_edge(self, edge_len: f32) -> Edge<A> {
        Edge::new(self, edge_len)
    }

    /// Return the [`Endpoint::Lower`] position.
    pub fn lower_pos<Sc: Scale>(&self, edge_len: f32) -> f32 {
        self.into_edge(edge_len).lower_pos::<Sc>()
    }

    /// Set the vertex offset from the given value.
    #[inline]
    pub(crate) fn set_offset<Sc: Scale>(&mut self, offset: f32) {
        self.offset = VertexOffset::new::<Sc>(offset);
        self.align_to_bound::<Sc>();
    }

    /// Set an offset assuming bound alignment.
    #[inline]
    pub const fn set_offset_unchecked(&mut self, offset: VertexOffset) {
        self.offset = offset;
    }

    /// Set the offset from the given value, relative to the given endpoint.
    #[inline]
    pub fn set_endpoint_offset<Sc: Scale>(&mut self, mut offset: f32, end: Endpoint<A>) {
        offset = match *end {
            EndpointBound::Lower => Sc::SCALE - offset,
            EndpointBound::Upper => offset,
        };
        self.set_offset::<Sc>(offset);
    }

    /// Reverse the vertex endpoint. Returns a vertex of an equal transform, but not necessarily an
    /// equal index.
    ///
    /// # Example
    /// ```rust
    /// # use tilebound::prelude::*;
    /// # use tilebound::topology::vertex::Vertex;
    /// type Sc = ConSc<16>;
    ///
    /// let mut vertex_on_bound = Vertex::from_parts::<Sc>(0, 0.0, Endpoint::RIGHT);
    /// let mut vertex_offset = Vertex::from_parts::<Sc>(0, 8.0, Endpoint::RIGHT);
    /// assert_eq!(vertex_on_bound.index(), -1);
    /// assert_eq!(vertex_offset.index(), 0);
    ///
    /// vertex_on_bound.reverse::<Sc>();
    /// assert_eq!(vertex_on_bound.index(), 0);
    /// assert_eq!(vertex_on_bound.offset_value(), 0.0);
    /// vertex_offset.reverse::<Sc>();
    /// assert_eq!(vertex_offset.index(), 0);
    /// assert_eq!(vertex_offset.offset_value(), 8.0);
    /// ```
    pub fn reverse<Sc: Scale>(&mut self) {
        self.end = self.end.invert();
        self.align_to_bound::<Sc>();
    }

    /// Returns a vertex of an equal transform with the endpoint reversed. See also [`Vertex::reverse`].
    pub fn reversed<Sc: Scale>(mut self) -> Self {
        self.end = self.end.invert();
        self.align_to_bound::<Sc>();

        self
    }

    /// Cast the `Edge` axis marker to another type.
    pub const fn cast<B>(self) -> Vertex<B> {
        Vertex {
            end: self.end.cast(),
            index: self.index,
            offset: self.offset,
        }
    }
}

impl<A: Axis> Vertex<A> {
    /// Transpose the vertex to another axis.
    ///
    /// This function returns a `Vertex` at the given transpose index and endpoint, with a
    /// transposed offset. That is; a transposed vertex with an offset **near** the next bound will
    /// return a transpose vertex with an offset **far** from the next bound.
    ///
    /// ```rust
    /// # use tilebound::prelude::*;
    /// # use tilebound::topology::vertex::Vertex;
    /// type Sc = ConSc<16>;
    ///
    /// let vert = Vertex::from_pos::<Sc>(4.0, Endpoint::LEFT);
    /// let t_vert = vert.transpose::<Sc>(0, Endpoint::TOP);
    ///
    ///
    /// // Returns values of an equal disatance to the next tile bound.
    /// assert_eq!(vert.offset_value(), 4.0);
    /// assert_eq!(t_vert.offset_value(), 12.0);
    /// assert_eq!(vert.offset().reversed::<Sc>(), t_vert.offset());
    /// ```
    pub fn transpose<Sc: Scale>(mut self, t_index: i32, t_end: Endpoint<A::T>) -> Vertex<A::T> {
        self.offset.transpose::<_, Sc>(self.end, t_end);

        let mut vertex = Vertex {
            end: t_end,
            index: t_index,
            offset: self.offset,
        };
        vertex.align_to_bound::<Sc>();
        vertex
    }
}

impl<A> PartialEq for Vertex<A> {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index && self.offset == other.offset
    }
}

impl<A> PartialOrd for Vertex<A> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.end == other.end {
            match self.index.cmp(&other.index) {
                Ordering::Equal => self.offset.partial_cmp(&other.offset),
                ord => Some(ord),
            }
        } else {
            // PartialOrd only supports vertices of the same axis and endpoint.
            None
        }
    }
}

/// # VertexOffset
///
/// An offset within a tile.
///
/// `VertexOffset` is a fraction of a tile spanning `0.0..Scale::scale()` for a given [`Scale`],
/// aligned to a [`Vertex`] endpoint. A valid `VertexOffset` should ensure that a value on the
/// [`Endpoint::Lower`] can only equal `0.0` while a value on the [`Endpoint::Upper`] bound can only
/// equal [`Scale::scale`]. This should usually be delegated to the [`Vertex`] instantiation.
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct VertexOffset(f32);

impl VertexOffset {
    /// Create a new `VertexOffset` of the given value.
    pub(crate) fn new<Sc: Scale>(offset: f32) -> Self {
        debug_assert!(offset >= 0.0);
        debug_assert!(offset <= Sc::SCALE);

        VertexOffset(offset)
    }

    /// Create a new `VertexOffset` with a value aligned to the given [`Endpoint`] bound.
    pub fn new_oriented<A, Sc: Scale>(offset: f32, end: Endpoint<A>) -> VertexOffset {
        debug_assert!(offset < Sc::SCALE);

        match *end {
            EndpointBound::Lower => VertexOffset(offset),
            EndpointBound::Upper => VertexOffset(Sc::SCALE - offset),
        }
    }

    /// Create a new `VertexOffset` from an endpoint bound.
    pub fn from_bound<A, Sc: Scale>(end: Endpoint<A>) -> VertexOffset {
        match *end {
            EndpointBound::Lower => VertexOffset(0.0),
            EndpointBound::Upper => VertexOffset(Sc::SCALE),
        }
    }

    /// Align the `VertexOffset`, ensuring a bound value is equal to the given [`Endpoint`]. Returns
    /// a `fn` for aligning an index. See [`VertexOffset`] documentation for more on bound alignment.
    pub(crate) fn align_to_bound<A, Sc: Scale>(
        &mut self,
        end: Endpoint<A>,
    ) -> Option<fn(&mut i32)> {
        match (*end, self.0) {
            (EndpointBound::Upper, 0.0) => {
                self.0 = Sc::SCALE;
                Some(|i| *i -= 1)
            }
            (EndpointBound::Lower, offset) if offset == Sc::SCALE => {
                self.0 = 0.0;
                Some(|i| *i += 1)
            }
            _ => None,
        }
    }

    /// Get the `VertexOffset` as an `f32` value.
    pub const fn value(&self) -> f32 {
        self.0
    }

    /// Returns `true` if the `VertexOffset` is on a tile bound.
    pub fn is_on_bound<Sc: Scale>(&self) -> bool {
        // Avoid using modulo because `VertexOffset` is guaranteed to be `0.0..=sc`.
        self.0 == 0.0 || self.0 == Sc::SCALE
    }

    /// Transposes the `VertexOffset` value from `end_a` of the current axis to `end_b` of the
    /// transpose axis.
    pub(crate) fn transpose<A: Axis, Sc: Scale>(
        &mut self,
        end_a: Endpoint<A>,
        end_b: Endpoint<A::T>,
    ) {
        match *end_a == *end_b {
            true => self.0 = Sc::SCALE - self.0,
            false => {}
        }
    }

    /// Reverse the `VertexOffset`.
    pub fn reverse<Sc: Scale>(&mut self) {
        self.0 = Sc::SCALE - self.0;
    }

    /// Return a reversed `VertexOffset`.
    pub fn reversed<Sc: Scale>(mut self) -> Self {
        self.reverse::<Sc>();
        self
    }
}

impl From<f32> for VertexOffset {
    fn from(value: f32) -> Self {
        VertexOffset(value)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::plane::axis::AxisX;
    use crate::plane::scale::ConSc;

    type Sc = ConSc<16>;

    mod vertex {

        use super::*;

        #[test]
        fn incr_index() {
            let mut vert_neg = Vertex::<AxisX>::from_pos::<Sc>(8.0, Endpoint::LOWER);
            vert_neg.incr_index();
            let mut vert_pos = Vertex::<AxisX>::from_pos::<Sc>(8.0, Endpoint::UPPER);
            vert_pos.incr_index();

            assert_eq!(vert_neg, Vertex::from_pos::<Sc>(-8.0, Endpoint::LOWER));
            assert_eq!(vert_pos, Vertex::from_pos::<Sc>(24.0, Endpoint::UPPER));
        }

        #[test]
        fn decr_index() {
            let mut vert_neg = Vertex::<AxisX>::from_pos::<Sc>(8.0, Endpoint::LOWER);
            vert_neg.decr_index();
            let mut vert_pos = Vertex::<AxisX>::from_pos::<Sc>(8.0, Endpoint::UPPER);
            vert_pos.decr_index();

            assert_eq!(vert_neg, Vertex::from_pos::<Sc>(24.0, Endpoint::LOWER));
            assert_eq!(vert_pos, Vertex::from_pos::<Sc>(-8.0, Endpoint::UPPER));
        }

        mod incr_bound {

            use super::*;

            #[test]
            fn incr_from_offset() {
                let mut vert_neg = Vertex::<AxisX>::from_pos::<Sc>(8.0, Endpoint::LOWER);
                vert_neg.incr_bound::<Sc>();
                let mut vert_pos = Vertex::<AxisX>::from_pos::<Sc>(8.0, Endpoint::UPPER);
                vert_pos.incr_bound::<Sc>();

                assert_eq!(vert_neg, Vertex::from_pos::<Sc>(0.0, Endpoint::LOWER));
                assert_eq!(vert_pos, Vertex::from_pos::<Sc>(16.0, Endpoint::UPPER));
            }

            #[test]
            fn incr_from_bound() {
                let mut vert_neg = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
                vert_neg.incr_bound::<Sc>();
                let mut vert_pos = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::UPPER);
                vert_pos.incr_bound::<Sc>();

                assert_eq!(vert_neg, Vertex::from_pos::<Sc>(-16.0, Endpoint::LOWER));
                assert_eq!(vert_pos, Vertex::from_pos::<Sc>(16.0, Endpoint::UPPER));
            }
        }

        mod decr_bound {

            use super::*;

            #[test]
            fn decr_from_offset() {
                let mut vert_neg = Vertex::<AxisX>::from_pos::<Sc>(8.0, Endpoint::LOWER);
                vert_neg.decr_bound::<Sc>();
                let mut vert_pos = Vertex::<AxisX>::from_pos::<Sc>(8.0, Endpoint::UPPER);
                vert_pos.decr_bound::<Sc>();

                assert_eq!(vert_neg, Vertex::from_pos::<Sc>(16.0, Endpoint::LOWER));
                assert_eq!(vert_pos, Vertex::from_pos::<Sc>(0.0, Endpoint::UPPER));
            }

            #[test]
            fn decr_from_bound() {
                let mut vert_neg = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
                vert_neg.decr_bound::<Sc>();
                let mut vert_pos = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::UPPER);
                vert_pos.decr_bound::<Sc>();

                assert_eq!(vert_neg, Vertex::from_pos::<Sc>(16.0, Endpoint::LOWER));
                assert_eq!(vert_pos, Vertex::from_pos::<Sc>(-16.0, Endpoint::UPPER));
            }
        }
    }
}
