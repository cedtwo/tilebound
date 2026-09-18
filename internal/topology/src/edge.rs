use std::ops::{Range, RangeInclusive};

use crate::vertex::{Vertex, VertexOffset};
use tilebound_plane::axis::DynAxis;
use tilebound_plane::endpoint::Endpoint;
use tilebound_plane::scale::Scale;

/// An [`Edge`] with no axis specified. This should only be used where an [`Axis`] can be inferred
/// and cast by context (eg. in [`AxisVec`](tilebound_plane::axis::AxisVec)).
pub type RawEdge = Edge<DynAxis>;

/// # Edge
///
/// A one-dimensional endpoint position and length.
///
/// In contrast to [`Vertex`], `Edge` allows for operations for both adjacent vertices. `Edge` is
/// considered *directed*, referring to the contained [`Vertex`] as the *inbound* [`Vertex`].
#[derive(Debug, PartialEq)]
pub struct Edge<A> {
    /// The *inbound* vertex.
    pub(crate) vertex: Vertex<A>,
    /// The edge length.
    len: f32,
}

impl<A> Clone for Edge<A> {
    fn clone(&self) -> Self {
        Self {
            vertex: self.vertex.clone(),
            len: self.len.clone(),
        }
    }
}

impl<A> Copy for Edge<A> {}

impl<A> Edge<A> {
    /// Create a new `Edge` from a vertex and a length.
    pub const fn new(vertex: Vertex<A>, len: f32) -> Self {
        debug_assert!(len.is_sign_positive());
        Self { vertex, len }
    }

    /// Create a new `Edge` from the given `f32` position and [`Endpoint`].
    #[inline]
    pub fn from_pos<Sc: Scale>(p: f32, len: f32, end: Endpoint) -> Self {
        let vertex = Vertex::from_pos::<Sc>(p, end);
        Self::new(vertex, len)
    }

    /// Returns the vertex at the given endpoint [`Endpoint`].
    pub fn vertex<Sc: Scale>(&self, end: Endpoint) -> Vertex<A> {
        if end.const_eq(self.end() as u8) {
            self.vertex
        } else {
            Vertex::<A>::from_pos::<Sc>(self.pos::<Sc>(end), end)
        }
    }

    /// Get the *inbound* vertex. See also [`Edge::vertex`].
    pub const fn inbound_vertex(&self) -> Vertex<A> {
        self.vertex
    }

    /// Set the inbound vertex. Modifying the `Vertex` is primarily used for translation or changing
    /// the *inbound* [`Endpoint`]. See also [`Edge::reverse`] and [`Edge::translate`].
    pub const fn set_vertex(&mut self, vertex: Vertex<A>) {
        self.vertex = vertex;
    }

    /// Get the inbound vertex [`Endpoint`].
    #[inline]
    pub const fn end(&self) -> Endpoint {
        self.vertex.endpoint()
    }

    /// Get the length of the edge.
    #[inline]
    pub const fn len(&self) -> f32 {
        self.len
    }

    /// Get the `f32` position at the given [`Endpoint`]. See also [`Edge::lower_pos`].
    pub fn pos<Sc: Scale>(&self, end: Endpoint) -> f32 {
        if end.const_eq(self.end() as u8) {
            self.vertex.to_pos::<Sc>()
        } else {
            self.end().decr_n(self.vertex.to_pos::<Sc>(), self.len)
        }
    }

    /// Return the [`Endpoint::Lower`] `f32` position. This is usually used as shorthand to return
    /// the top/left position of an `Edge`.
    #[inline]
    pub fn lower_pos<Sc: Scale>(&self) -> f32 {
        self.pos::<Sc>(Endpoint::LOW)
    }

    /// Get the [`VertexOffset`] at the given [`Endpoint`].
    pub fn offset<Sc: Scale>(&self, end: Endpoint) -> VertexOffset {
        if end.const_eq(self.end() as u8) {
            self.vertex.offset()
        } else {
            let mut offset = VertexOffset::new::<Sc>(
                self.end().decr_n(self.vertex.offset_value(), self.len) % Sc::SCALE,
            );
            offset.align_to_bound::<Sc>(end);
            offset
        }
    }

    /// Returns `true` if the given `vertex` intersects the `Edge` (inclusive of endpoints).
    pub fn contains<Sc: Scale>(&self, vertex: Vertex<A>) -> bool {
        self.range::<Sc>().contains(&vertex.to_pos::<Sc>())
    }

    /// Translate the [`Vertex`].
    #[inline]
    pub fn translate<Sc: Scale>(&mut self, delta: f32) {
        self.vertex.translate::<Sc>(delta);
    }

    /// Return an `Edge` translated by the given `delta`. This does not change the *inbound*
    /// endpoint.
    #[inline]
    pub fn translated<Sc: Scale>(mut self, delta: f32) -> Self {
        self.vertex.translate::<Sc>(delta);
        self
    }

    /// Set the inbound [`Endpoint`], possibly reversing the edge.
    pub fn set_inbound_end<Sc: Scale>(&mut self, end: Endpoint) {
        if self.vertex.endpoint() != end {
            self.reverse::<Sc>();
        }
    }

    /// Return an `Edge` with the given inbound [`Endpoint`].
    #[inline]
    pub fn with_inbound_end<Sc: Scale>(mut self, end: Endpoint) -> Self {
        self.set_inbound_end::<Sc>(end);
        self
    }

    /// Sets the inbound [`Vertex`] to the adjacent vertex.
    pub fn reverse<Sc: Scale>(&mut self) {
        let vertex = self.vertex;
        let end = vertex.endpoint().invert();

        let transform = self.pos::<Sc>(end);
        self.vertex = Vertex::from_pos::<Sc>(transform, end);
    }

    /// Return an `Edge` with the inbound [`Vertex`] set to the adjacent vertex.
    pub fn reversed<Sc: Scale>(mut self) -> Self {
        let vertex = self.vertex;
        let end = !vertex.endpoint();

        let transform = self.pos::<Sc>(end);
        self.vertex = Vertex::from_pos::<Sc>(transform, end);

        self
    }

    /// Consumes the `Edge`, returning the inbound [`Vertex`].
    pub fn take_vertex(self) -> Vertex<A> {
        self.vertex
    }

    /// Consumes the `Edge`, returning the outbound [`Vertex`].
    pub fn adjacent_vertex<Sc: Scale>(self) -> Vertex<A> {
        self.reversed::<Sc>().take_vertex()
    }

    /// Get the vertex endpoint closest to an outer tile bound. Returns `None` where both vertices
    /// are of an equal distance from their respective outer bound.
    pub fn nearest_bound_end<Sc: Scale>(self) -> Option<Endpoint> {
        let a = self.inbound_vertex();
        let b = self.adjacent_vertex::<Sc>();

        let mut a_offset = a.offset();
        let mut b_offset = b.offset();

        // Compare values relative to the same endpoint.
        match a.endpoint() {
            Endpoint::LOW => a_offset.reverse::<Sc>(),
            Endpoint::UPP => b_offset.reverse::<Sc>(),
        };

        match a_offset.partial_cmp(&b_offset) {
            Some(std::cmp::Ordering::Less) => Some(!self.end()),
            Some(std::cmp::Ordering::Equal) => None,
            Some(std::cmp::Ordering::Greater) => Some(self.end()),
            None => unreachable!("Expected vertices of an equal `Endpoint`"),
        }
    }

    /// Get the vertex closest to an outer tile bound. Returns `None` where both vertices are of an
    /// equal distance from their respective outer bound.
    pub fn nearest_bound_vertex<Sc: Scale>(self) -> Option<Vertex<A>> {
        let a = self.inbound_vertex();
        let b = self.adjacent_vertex::<Sc>();

        let mut a_offset = a.offset();
        let mut b_offset = b.offset();

        // Compare values relative to the same endpoint.
        match a.endpoint() {
            Endpoint::LOW => a_offset.reverse::<Sc>(),
            Endpoint::UPP => b_offset.reverse::<Sc>(),
        };

        match a_offset.partial_cmp(&b_offset) {
            Some(std::cmp::Ordering::Less) => Some(b),
            Some(std::cmp::Ordering::Equal) => None,
            Some(std::cmp::Ordering::Greater) => Some(a),
            None => unreachable!("Expected vertices of an equal `Endpoint`"),
        }
    }

    /// Returns `true` if the edge is a multiple of the [`Scale`].
    #[inline]
    pub fn is_not_padded<Sc: Scale>(&self) -> bool {
        self.len % Sc::SCALE == 0.0
    }

    /// Get the length of the `Edge` in tiles (rounded up).
    #[inline]
    pub fn tile_len<Sc: Scale>(&self) -> u32 {
        (self.len() / Sc::SCALE).ceil() as u32
    }

    /// Get the tile index at the given [`Endpoint`].
    pub fn index<Sc: Scale>(&self, end: Endpoint) -> i32 {
        if end.const_eq(self.end() as u8) {
            self.vertex.index()
        } else {
            // Index requires calculating the offset so must create a vertex regardless.
            Vertex::<A>::from_pos::<Sc>(self.pos::<Sc>(end), end).index
        }
    }

    /// Get the edge as an `f32` position range. This returns [`RangeInclusive`] to reflect that the
    /// upper bound is intersected.
    pub fn range<Sc: Scale>(&self) -> RangeInclusive<f32> {
        let transform = self.vertex.to_pos::<Sc>();
        match self.vertex.endpoint() {
            Endpoint::LOW => transform..=(transform + self.len),
            Endpoint::UPP => (transform - self.len)..=transform,
        }
    }

    /// Get the edge as an `i32` tile range. This returns [`RangeInclusive`] to reflect that the
    /// upper bound is intersected.
    pub fn index_range<Sc: Scale>(&self) -> Range<i32> {
        let (start, end) = match self.end() {
            Endpoint::LOW => {
                let start = self.vertex.index();
                let end_t = self.vertex.to_pos::<Sc>() + self.len;
                let end = Vertex::<A>::from_pos::<Sc>(end_t, Endpoint::UPP).index();
                (start, end)
            }
            Endpoint::UPP => {
                let end = self.vertex.index();
                let start_t = self.vertex.to_pos::<Sc>() - self.len;
                let start = Vertex::<A>::from_pos::<Sc>(start_t, Endpoint::LOW).index();
                (start, end)
            }
        };
        start..end + 1
    }

    /// Get the distance between the edge length and the next tile bound. Returns zero if the edge
    /// is a multiple of the [`Scale`].
    pub fn padding<Sc: Scale>(&self) -> f32 {
        let len = self.len;
        let tile_len = (len / Sc::SCALE).ceil() * Sc::SCALE;

        tile_len - len
    }

    /// Returns `true` if the endpoint for the given [`Endpoint`] is on a tile bound.
    pub fn is_on_bound<Sc: Scale>(&self, end: Endpoint) -> bool {
        self.pos::<Sc>(end) % Sc::SCALE == 0.0
    }

    /// Returns `true` if either endpoint is on a tile bound.
    pub fn is_on_either_bound<Sc: Scale>(&self) -> bool {
        self.vertex.offset().is_on_bound::<Sc>()
            || self.end().decr_n(self.vertex.offset_value(), self.len) % Sc::SCALE == 0.0
    }

    /// Cast the `Edge` axis marker to another type.
    pub const fn cast<B>(self) -> Edge<B> {
        Edge {
            vertex: self.vertex.cast(),
            len: self.len,
        }
    }
}

#[cfg(test)]
mod tests {

    use tilebound_plane::axis::AxisX;
    use tilebound_plane::scale::ConSc;

    type Sc = ConSc<16>;

    use super::*;

    mod edge {

        use super::*;

        #[test]
        fn as_range() {
            let edge_neg = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOW), 16.0);
            let edge_pos = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(16.0, Endpoint::UPP), 16.0);

            let edge_neg_range = edge_neg.range::<Sc>();
            let edge_pos_range = edge_pos.range::<Sc>();

            assert_eq!(edge_neg_range, 0.0..=16.0);
            assert_eq!(edge_pos_range, 0.0..=16.0);
        }

        #[test]
        fn as_index_range() {
            let edge_neg = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOW), 32.0);
            let edge_pos = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(32.0, Endpoint::UPP), 32.0);

            let edge_neg_idx_range = edge_neg.index_range::<Sc>();
            let edge_pos_idx_range = edge_pos.index_range::<Sc>();

            assert_eq!(edge_neg_idx_range, (0..2).into());
            assert_eq!(edge_pos_idx_range, (0..2).into());
        }

        #[test]
        fn origin() {
            let edge_neg = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOW), 32.0);
            let edge_pos = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(32.0, Endpoint::UPP), 32.0);

            let edge_neg_origin = edge_neg.lower_pos::<Sc>();
            let edge_pos_origin = edge_pos.lower_pos::<Sc>();

            assert_eq!(edge_neg_origin, 0.0);
            assert_eq!(edge_pos_origin, 0.0);
        }

        #[test]
        fn padding() {
            let edge_neg_nopad =
                Edge::new(Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOW), 32.0);
            let edge_pos_nopad =
                Edge::new(Vertex::<AxisX>::from_pos::<Sc>(32.0, Endpoint::UPP), 32.0);
            let edge_neg_pad = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOW), 31.0);
            let edge_pos_pad =
                Edge::new(Vertex::<AxisX>::from_pos::<Sc>(31.0, Endpoint::UPP), 31.0);

            assert_eq!(edge_neg_nopad.padding::<Sc>(), 0.0);
            assert_eq!(edge_neg_pad.padding::<Sc>(), 1.0);
            assert_eq!(edge_pos_nopad.padding::<Sc>(), 0.0);
            assert_eq!(edge_pos_pad.padding::<Sc>(), 1.0);
        }

        #[test]
        fn is_on_end() {
            let edge_neg = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOW), 15.0);
            let edge_pos = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::UPP), 15.0);

            assert!(!edge_neg.is_on_bound::<Sc>(Endpoint::LOW,));
            assert!(edge_neg.is_on_bound::<Sc>(Endpoint::UPP,));
            assert!(edge_pos.is_on_bound::<Sc>(Endpoint::LOW,));
            assert!(!edge_pos.is_on_bound::<Sc>(Endpoint::UPP,));
        }

        #[test]
        fn is_on_either_end() {
            let edge_neg_on_pos =
                Edge::new(Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOW), 15.0);
            let edge_pos_on_neg =
                Edge::new(Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::UPP), 15.0);
            let edge_neg_not_on_bound =
                Edge::new(Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOW), 14.0);
            let edge_pos_not_on_bound =
                Edge::new(Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::UPP), 14.0);

            assert!(edge_neg_on_pos.is_on_either_bound::<Sc>());
            assert!(edge_pos_on_neg.is_on_either_bound::<Sc>());
            assert!(!edge_neg_not_on_bound.is_on_either_bound::<Sc>());
            assert!(!edge_pos_not_on_bound.is_on_either_bound::<Sc>());
        }

        #[test]
        fn endpoint_index() {
            let edge_neg_0 = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOW), 32.0);
            let edge_neg_1 = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOW), 32.0);
            let edge_pos_0 = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(32.0, Endpoint::UPP), 32.0);
            let edge_pos_1 = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(31.0, Endpoint::UPP), 32.0);

            assert_eq!(edge_neg_0.index::<Sc>(Endpoint::LOW,), 0);
            assert_eq!(edge_neg_0.index::<Sc>(Endpoint::UPP,), 1);
            assert_eq!(edge_neg_1.index::<Sc>(Endpoint::LOW,), 0);
            assert_eq!(edge_neg_1.index::<Sc>(Endpoint::UPP,), 2);
            assert_eq!(edge_pos_0.index::<Sc>(Endpoint::LOW,), 0);
            assert_eq!(edge_pos_0.index::<Sc>(Endpoint::UPP,), 1);
            assert_eq!(edge_pos_1.index::<Sc>(Endpoint::LOW,), -1);
            assert_eq!(edge_pos_1.index::<Sc>(Endpoint::UPP,), 1);
        }

        #[test]
        fn endpoint_offset() {
            let edge_neg = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOW), 15.0);
            let edge_pos = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::UPP), 15.0);

            assert_eq!(
                edge_neg.offset::<Sc>(Endpoint::LOW),
                VertexOffset::new::<Sc>(1.0)
            );
            assert_eq!(
                edge_neg.offset::<Sc>(Endpoint::UPP),
                VertexOffset::new::<Sc>(16.,)
            );
            assert_eq!(
                edge_pos.offset::<Sc>(Endpoint::LOW),
                VertexOffset::new::<Sc>(0.0)
            );
            assert_eq!(
                edge_pos.offset::<Sc>(Endpoint::UPP),
                VertexOffset::new::<Sc>(15.,)
            );
        }

        #[test]
        fn endpoint_tfm() {
            let edge_neg = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOW), 16.0);
            let edge_pos = Edge::new(Vertex::<AxisX>::from_pos::<Sc>(16.0, Endpoint::UPP), 15.0);

            assert_eq!(edge_neg.pos::<Sc>(Endpoint::LOW,), 1.0);
            assert_eq!(edge_neg.pos::<Sc>(Endpoint::UPP,), 17.0);
            assert_eq!(edge_pos.pos::<Sc>(Endpoint::LOW,), 1.0);
            assert_eq!(edge_pos.pos::<Sc>(Endpoint::UPP,), 16.0);
        }

        #[test]
        fn tile_len() {
            let edge_neg = Edge::new(
                Vertex::<AxisX>::from_parts::<Sc>(0, 0.0, Endpoint::LOW),
                32.0,
            );
            let edge_pos = Edge::new(
                Vertex::<AxisX>::from_parts::<Sc>(3, 0.0, Endpoint::UPP),
                32.0,
            );

            assert_eq!(edge_neg.tile_len::<Sc>(), 2);
            assert_eq!(edge_pos.tile_len::<Sc>(), 2);
        }

        #[test]
        fn nearest_bound_end() {
            let edge_0 = Edge::<AxisX>::from_pos::<Sc>(8.0, 16.0, Endpoint::LOW);
            let edge_0_rev = edge_0.reversed::<Sc>();
            let edge_1 = Edge::<AxisX>::from_pos::<Sc>(8.0, 17.0, Endpoint::LOW);
            let edge_1_rev = edge_1.reversed::<Sc>();
            let edge_2 = Edge::<AxisX>::from_pos::<Sc>(6.0, 17.0, Endpoint::LOW);
            let edge_2_rev = edge_2.reversed::<Sc>();

            assert_eq!(edge_0.nearest_bound_end::<Sc>(), None);
            assert_eq!(edge_0_rev.nearest_bound_end::<Sc>(), None);
            assert_eq!(edge_1.nearest_bound_end::<Sc>(), Some(Endpoint::UPP));
            assert_eq!(edge_1_rev.nearest_bound_end::<Sc>(), Some(Endpoint::UPP));
            assert_eq!(edge_2.nearest_bound_end::<Sc>(), Some(Endpoint::LOW));
            assert_eq!(edge_2_rev.nearest_bound_end::<Sc>(), Some(Endpoint::LOW));
        }
    }
}
