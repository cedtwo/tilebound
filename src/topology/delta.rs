use std::{fmt::Debug, iter};

use crate::plane::axis::Axis;
use crate::plane::endpoint::Endpoint;
use crate::plane::scale::Scale;
use crate::topology::edge::Edge;
use crate::topology::vertex::{Vertex, VertexOffset};

/// # Delta
///
/// `Delta` produces [`Vertex`] positions of a given spacing prior to the delta target.
///
/// ## Bound Spacing
///
/// `Delta` holds a target [`Edge`] where the endpoint of the *inbound* vertex defines displacement
/// orientation. All operations therefore recieve and return vertices of an equal [`Endpoint`] to
/// the target [`Vertex`]. `Delta` is *stateless*, returning a vertex positioned at a point between
/// a given *origin* vertex, and the *inbound* target vertex. Operations are intended to be called
/// in loop, updating the *origin*, or breaking on collision. `Delta` supports the following
/// spacing operations:
/// - *inbound* spacing, returning a vertex positioned on the bound of the next tile,
/// - *outbound* spacing, returning a vertex positioning the inverse of the *origin* on the bound of
/// the next tile exited,
/// - *inbound* or *outbound* checked spacing, returning the nearest of the next spaced vertex, the
/// target, or `None` if there is no further vertex.
///
/// A common workflow is to iterate over all inbound vertices (See [`Delta::iter_inbound_to_target`]),
/// checking if a vertex index is intersected, checking the tilemap if not, and handling accordingly.
/// Returning *outbound* vertices (likely interspaced with *inbound* vertices) is not necessary for
/// most use cases.
#[derive(Debug, Clone, Copy)]
pub struct Delta<A> {
    /// The target edge.
    pub(crate) target: Edge<A>,
}

impl<A: Axis> Delta<A> {
    /// Create a new `Delta` from a `target` edge.
    pub fn new(target: Edge<A>) -> Self {
        Self { target }
    }

    /// Create a new `Delta` from an `f32` position, an edge length, and an [`Endpoint`]
    /// orientation.
    pub fn from_pos<S: Scale>(t: f32, len: f32, end: Endpoint<A>) -> Self {
        let target = Edge::new(Vertex::from_pos::<S>(t, end), len);
        Self { target }
    }

    /// Create a new `Delta` from an `origin` edge and delta. This will create a target vertex where
    /// the origin is translated by, and orientated toward the `delta`.
    fn from_delta<S: Scale>(mut origin: Edge<A>, delta: f32) -> Self {
        let end = Endpoint::from_f32_unchecked(delta);

        origin.set_inbound_end::<S>(end);
        origin.translate::<S>(delta);

        Self { target: origin }
    }

    /// Return the delta [`Endpoint`].
    #[inline]
    pub const fn end(&self) -> Endpoint<A> {
        self.target.end()
    }

    /// Return the delta target [`Vertex`].
    pub fn target(&self) -> Vertex<A> {
        self.target.inbound_vertex()
    }

    /// Set a new target for the `Delta`.
    pub fn set_target(&mut self, target: Vertex<A>) {
        debug_assert_eq!(self.end(), target.endpoint());
        self.target.set_vertex(target);
    }

    /// Return the remaining delta as an `f32`.
    #[inline]
    pub fn delta<S: Scale>(&self, origin: Vertex<A>) -> f32 {
        debug_assert_eq!(origin.endpoint(), self.end());
        debug_assert!(self.end().partial_gt(self.target.inbound_vertex(), &origin));
        self.target.inbound_vertex().to_pos::<S>() - origin.to_pos::<S>()
    }

    /// Returns `true` if the given `origin` has reached the target vertex.
    pub fn reached_target(&self, origin: Vertex<A>) -> bool {
        debug_assert_eq!(origin.endpoint(), self.end());
        debug_assert!(!self.end().partial_gt(origin, &self.target.inbound_vertex()));
        // TODO: Should this use end::partial_gt as fallback?
        origin == self.target.inbound_vertex()
    }

    /// Return the next (nearest) [`Endpoint`]. This will default to [`Endpoint::Lower`] where the
    /// next position is on both endpoints.
    pub fn next_endpoint<S: Scale>(&self, origin: Vertex<A>) -> Endpoint<A> {
        let end = self.end();
        let padding = self.target.padding::<S>();

        if padding == 0.0 {
            !end
        } else {
            let lower_offset = VertexOffset::new_oriented::<A, S>(padding, end);
            if origin.is_on_bound::<S>() || end.partial_gt(lower_offset, &origin.offset()) {
                !end
            } else {
                end
            }
        }
    }

    /// Return an iterator of all inbound tile bound vertices up to and including the target vertex.
    pub fn iter_inbound_to_target<S: Scale>(
        &self,
        mut origin: Vertex<A>,
    ) -> impl Iterator<Item = Vertex<A>> {
        iter::from_fn(move || {
            self.next_inbound_or_target::<S>(origin)
                .inspect(|next| origin = *next)
        })
    }

    /// Returns a vertex positioned on the next inbound tile bound. See also
    /// [`Delta::next_inbound_or_target`].
    pub fn next_inbound<S: Scale>(&self, mut origin: Vertex<A>) -> Vertex<A> {
        debug_assert_eq!(origin.endpoint(), self.end());
        debug_assert!(
            origin
                .endpoint()
                .partial_gt(self.target.inbound_vertex(), &origin)
        );
        let end = self.end();

        if origin.is_on_bound::<S>() {
            origin.incr_index();
        } else {
            origin.set_offset_unchecked(VertexOffset::from_bound::<A, S>(end));
        }
        origin
    }

    /// Returns a vertex positioned on the nearest of the next inbound tile bound, or the target.
    /// Returns `None` if the target has already been reached.
    pub fn next_inbound_or_target<S: Scale>(&self, origin: Vertex<A>) -> Option<Vertex<A>> {
        if self.reached_target(origin) {
            None
        } else {
            Some(self.next_inbound_or_target_unchecked::<S>(origin))
        }
    }

    /// Returns a vertex positioned on the nearest of the next inbound tile bound, or the target.
    /// **Panics** in debug, and returns the target vertex regardless if already reached.
    pub fn next_inbound_or_target_unchecked<S: Scale>(&self, origin: Vertex<A>) -> Vertex<A> {
        debug_assert!(!self.reached_target(origin));
        self.end()
            .partial_min(self.next_inbound::<S>(origin), self.target.inbound_vertex())
    }

    /// Returns a vertex positioning the *outbound* vertex on the next inbound tile bound. See also
    /// [`Delta::next_outbound_or_target`].
    pub fn next_outbound<S: Scale>(&self, mut origin: Vertex<A>) -> Vertex<A> {
        debug_assert_eq!(origin.endpoint(), self.end());
        debug_assert!(
            origin
                .endpoint()
                .partial_gt(self.target.inbound_vertex(), &origin)
        );
        let end = self.end();

        let padding = self.target.padding::<S>();
        let offset = VertexOffset::new_oriented::<A, S>(padding, end);

        if end
            .partial_ordering(origin.offset(), offset)
            .is_some_and(|ord| ord.is_ge())
        {
            origin.incr_index();
        }
        origin.set_offset_unchecked(offset);

        origin
    }

    /// Returns a vertex positioning the **outbound** vertex on the nearest of the next tile bound,
    /// or the target. Returns `None` if the target has already been reached.
    pub fn next_outbound_or_target<S: Scale>(&self, origin: Vertex<A>) -> Option<Vertex<A>> {
        if self.reached_target(origin) {
            None
        } else {
            Some(self.next_outbound_or_target_unchecked::<S>(origin))
        }
    }

    /// Returns a vertex positioning the **outbound** vertex on the nearest of the next tile bound,
    /// or the target. **Panics** in debug, and returns the target vertex regardless if already reached.
    pub fn next_outbound_or_target_unchecked<S: Scale>(&self, origin: Vertex<A>) -> Vertex<A> {
        debug_assert!(!self.reached_target(origin));
        self.end().partial_min(
            self.next_outbound::<S>(origin),
            self.target.inbound_vertex(),
        )
    }

    /// Return a `Delta` on the transpose axis of an equal remaining delta. Returns `None` if
    /// `Delta` is exhausted. Note that due to floating-point error the transposed delta will (very
    /// likely) not be equal to that prior.
    pub fn transpose<S: Scale>(
        &self,
        origin: Vertex<A>,
        t_origin: Edge<A::T>,
    ) -> Option<Delta<A::T>> {
        debug_assert_eq!(origin.endpoint(), self.end());
        debug_assert!(
            origin
                .endpoint()
                .partial_gt(self.target.inbound_vertex(), &origin)
        );
        if self.reached_target(origin) {
            None
        } else {
            Some(self.transpose_unchecked::<S>(origin, t_origin))
        }
    }

    /// Return a `Delta` on the transpose axis of an equal remaining delta. **Panics** if
    /// `Delta` is exhausted. Note that due to floating-point error the transposed delta will (very
    /// likely) not be equal to that prior.
    pub fn transpose_unchecked<S: Scale>(
        &self,
        origin: Vertex<A>,
        t_origin: Edge<A::T>,
    ) -> Delta<A::T> {
        debug_assert_eq!(origin.endpoint(), self.end());
        debug_assert!(
            origin
                .endpoint()
                .partial_gt(self.target.inbound_vertex(), &origin)
        );
        let mut delta = self.delta::<S>(origin);
        if *self.end() != *t_origin.end() {
            delta = -delta
        };
        Delta::from_delta::<S>(t_origin, delta)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::plane::axis::AxisX;
    use crate::plane::scale::ConSc;

    type S = ConSc<16>;

    #[test]
    fn next_upper_endpoint() {
        let origin_r_neg = Vertex::from_pos::<S>(18.0, Endpoint::LOWER);
        let origin_l_neg = Vertex::from_pos::<S>(15.0, Endpoint::LOWER);
        let tgt_neg = Delta::new(Edge::<AxisX>::from_pos::<S>(0.0, 15.0, Endpoint::LOWER));
        let origin_r_pos = Vertex::from_pos::<S>(18.0, Endpoint::UPPER);
        let origin_l_pos = Vertex::from_pos::<S>(15.0, Endpoint::UPPER);
        let tgt_pos = Delta::new(Edge::<AxisX>::from_pos::<S>(32.0, 15.0, Endpoint::UPPER));

        assert_eq!(
            tgt_neg.next_inbound::<S>(origin_r_neg),
            Vertex::from_bound::<S>(1, Endpoint::LOWER)
        );
        assert_eq!(
            tgt_neg.next_inbound::<S>(origin_l_neg),
            Vertex::from_bound::<S>(0, Endpoint::LOWER)
        );
        assert_eq!(
            tgt_pos.next_inbound::<S>(origin_r_pos),
            Vertex::from_bound::<S>(1, Endpoint::UPPER)
        );
        assert_eq!(
            tgt_pos.next_inbound::<S>(origin_l_pos),
            Vertex::from_bound::<S>(0, Endpoint::UPPER)
        );
    }

    #[test]
    fn next_lower_endpoint() {
        let origin_r = Vertex::from_pos::<S>(18.0, Endpoint::LOWER);
        let origin_l = Vertex::from_pos::<S>(15.0, Endpoint::LOWER);
        let tgt = Delta::new(Edge::<AxisX>::from_pos::<S>(0.0, 15.0, Endpoint::LOWER));

        assert_eq!(
            tgt.next_outbound::<S>(origin_r),
            Vertex::from_parts::<S>(1, 1.0, Endpoint::LOWER)
        );
        assert_eq!(
            tgt.next_outbound::<S>(origin_l),
            Vertex::from_parts::<S>(0, 1.0, Endpoint::LOWER)
        );
    }

    #[test]
    fn next_endpoint() {
        let tgt_16 = Delta::new(Edge::<AxisX>::from_pos::<S>(1.0, 16.0, Endpoint::LOWER));
        let tgt_10 = Delta::new(Edge::<AxisX>::from_pos::<S>(1.0, 10.0, Endpoint::LOWER));

        assert_eq!(
            tgt_16.next_endpoint::<S>(Vertex::from_pos::<S>(17.0, Endpoint::LOWER)),
            Endpoint::UPPER
        );
        assert_eq!(
            tgt_16.next_endpoint::<S>(Vertex::from_pos::<S>(16.0, Endpoint::LOWER)),
            Endpoint::UPPER
        );
        assert_eq!(
            tgt_16.next_endpoint::<S>(Vertex::from_pos::<S>(15.0, Endpoint::LOWER)),
            Endpoint::UPPER
        );

        assert_eq!(
            tgt_10.next_endpoint::<S>(Vertex::from_pos::<S>(17.0, Endpoint::LOWER)),
            Endpoint::LOWER
        );
        assert_eq!(
            tgt_10.next_endpoint::<S>(Vertex::from_pos::<S>(16.0, Endpoint::LOWER)),
            Endpoint::UPPER
        );
        assert_eq!(
            tgt_10.next_endpoint::<S>(Vertex::from_pos::<S>(15.0, Endpoint::LOWER)),
            Endpoint::UPPER
        );
        assert_eq!(
            tgt_10.next_endpoint::<S>(Vertex::from_pos::<S>(6.0, Endpoint::LOWER)),
            Endpoint::LOWER
        );
    }

    #[test]
    fn transpose() {
        // 32.0 to 16.0
        let origin_neg = Vertex::from_pos::<S>(32.0, Endpoint::LOWER);
        let tgt_neg = Delta::new(Edge::<AxisX>::from_pos::<S>(16.0, 1.0, Endpoint::LOWER));
        // 0.0 to 16.0
        let origin_pos = Vertex::from_pos::<S>(0.0, Endpoint::UPPER);
        let tgt_pos = Delta::new(Edge::<AxisX>::from_pos::<S>(16.0, 1.0, Endpoint::UPPER));

        let t_edge_neg = Edge::from_pos::<S>(0.0, 1.0, Endpoint::LOWER);
        let t_edge_pos = Edge::from_pos::<S>(0.0, 1.0, Endpoint::UPPER);

        let neg_to_neg = tgt_neg.transpose_unchecked::<S>(origin_neg, t_edge_neg);
        let neg_to_pos = tgt_neg.transpose_unchecked::<S>(origin_neg, t_edge_pos);
        let pos_to_neg = tgt_pos.transpose_unchecked::<S>(origin_pos, t_edge_neg);
        let pos_to_pos = tgt_pos.transpose_unchecked::<S>(origin_pos, t_edge_pos);

        assert_eq!(
            neg_to_neg.target,
            Edge::new(Vertex::from_pos::<S>(-16.0, Endpoint::LOWER), 1.0)
        );
        assert_eq!(
            neg_to_pos.target,
            Edge::new(Vertex::from_pos::<S>(16.0, Endpoint::UPPER), 1.0)
        );

        assert_eq!(
            pos_to_neg.target,
            Edge::new(Vertex::from_pos::<S>(-16.0, Endpoint::LOWER), 1.0)
        );
        assert_eq!(
            pos_to_pos.target,
            Edge::new(Vertex::from_pos::<S>(16.0, Endpoint::UPPER), 1.0)
        );
    }
}
