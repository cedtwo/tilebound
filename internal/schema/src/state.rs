use std::ops::Range;

use tilebound_plane::axis::{Axis, AxisMask, AxisVec, AxisX, AxisY};
use tilebound_plane::endpoint::Endpoint;
use tilebound_plane::scale::Scale;
use tilebound_topology::edge::{Edge, RawEdge};
use tilebound_topology::vertex::{RawVertex, Vertex};
use tilebound_view::index::SliceIndex;
use tilebound_view::inspect::TileMapInspect;
use tilebound_view::tilemap::TileMapView;

use crate::scene::Scene;

/// # State
///
/// Rectangle state variables. Stores the current and last position of the collider, and the
/// attachment (collision) [`AxisMask`]. Provides operations for state assertion and mutation.
/// For use in displacement operations, `State` can be created with
///
/// `State` accepts a generic resource `R`, accessible through the [`State::res`] and
/// [`State::res_mut`] methods. This can be used to provide additional context to handlers, or
/// return additional data to the caller.
///
/// Note that assertions on recent changes (eg. [`State::index_changed`]) are relative to the last call to
/// [`State::update`]. In addition to assertions, `State` is primarily used for updating a position
/// with [`State::set_vertex`], and attachments with [`State::attach`] and [`State::detach`]. Use
/// [`State::pos`] or [`State::pos_vec`] to get the top/left position(s).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct State<R = ()> {
    /// The current position and size.
    curr_edges: AxisVec<RawEdge>,
    /// The current attachments.
    curr_attmask: AxisMask,
    /// The last position.
    last_verts: AxisVec<RawVertex>,
    /// External resources.
    res: R,
}

impl<R> State<R> {
    /// Create a new `State`. Requires a tuple containing an *x* and *y* axis position, the lengths
    /// on either axis, and a collision [`AxisMask`]. See [`StatePayload`] for tuple patterns.
    pub fn new<Sc: Scale>(payload: impl StatePayload<R>) -> Self
    where
        R: Default,
    {
        let (pos, len, attmask, res) = payload.split_payload();
        let curr_edges = Self::bounding_box::<Sc>(pos, len);
        let last_verts = curr_edges.map(|e| e.take_vertex());
        Self {
            curr_edges,
            curr_attmask: attmask,
            last_verts,
            res,
        }
    }

    /// Return the `State` with the last position set. Handles orientating the current position away
    /// from the last position. This is generally only useful for debug assertions.
    pub fn with_last_pos<Sc: Scale>(mut self, last_pos: impl Into<AxisVec<f32>>) -> Self {
        let last_pos = last_pos.into();
        let ends = self.pos_vec::<Sc>().map_with(last_pos, |curr, last| {
            Endpoint::from_partial_cmp(curr, last).unwrap_or_default()
        });
        self.curr_edges.x_mut().set_inbound_end::<Sc>(ends.x());
        self.curr_edges.y_mut().set_inbound_end::<Sc>(ends.y());
        self.last_verts = last_pos.map_with(ends, |pos, end| Vertex::from_pos::<Sc>(pos, end));
        debug_assert_eq!(self.curr_edges.x().end(), self.last_verts.x().endpoint());
        debug_assert_eq!(self.curr_edges.y().end(), self.last_verts.y().endpoint());

        self
    }

    /// Return an `AxisVec<RawEdge>` for the given position and length. This is roughly equivalent
    /// to a bounding box with [`Vertex`] alignment.
    fn bounding_box<S: Scale>(
        pos: impl Into<AxisVec<f32>>,
        len: impl Into<AxisVec<f32>>,
    ) -> AxisVec<RawEdge> {
        let pos = pos.into();
        let len = len.into();
        AxisVec::new(
            Edge::from_pos::<S>(pos.get::<AxisX>(), len.get::<AxisX>(), Endpoint::LOW),
            Edge::from_pos::<S>(pos.get::<AxisY>(), len.get::<AxisY>(), Endpoint::LOW),
        )
    }

    /// Get a reference to the state resource(s).
    pub fn res(&self) -> &R {
        &self.res
    }

    /// Get a mutable reference to the state resource(s).
    pub fn res_mut(&mut self) -> &mut R {
        &mut self.res
    }

    /// Attach the given endpoint of the generic axis `A`. Detaches the inverse of the given `end`.
    pub fn attach<A: Axis>(&mut self, end: Endpoint) {
        self.curr_attmask.0 =
            self.curr_attmask.isolate_axis::<A::T>().0 | AxisMask::from_end::<A>(end).0;
    }

    /// Detach both endpoints of the generic axis `A`.
    pub fn detach<A: Axis>(&mut self) {
        self.curr_attmask.clear_axis::<A>();
    }

    /// Return the inner [`AxisMask`].
    pub fn attmask(&self) -> AxisMask {
        self.curr_attmask
    }

    /// Returns `true` if attached on the given [`Endpoint`] of axis `A`.
    pub fn end_is_attached<A: Axis>(&self, end: Endpoint) -> bool {
        self.curr_attmask.end_is_set::<A>(end)
    }

    /// Returns `true` if attached on either endpoint of the given axis `A`.
    pub fn axis_is_attached<A: Axis>(&self) -> bool {
        self.curr_attmask.any_on_axis::<A>()
    }

    /// Return the attached [`Endpoint`] of axis `A`.
    pub fn attached_endpoint<A: Axis>(&self) -> Option<Endpoint> {
        self.curr_attmask.first_end::<A>()
    }

    /// Return the displaced [`Endpoint`] of axis `A` or `None` if no displacement has occured since
    /// the last call to [`State::update`]. If certain a new position has been set with a [`Vertex`]
    /// of an [`Endpoint`] equal to displacement, then calling [`State::inbound_endpoint`] can be
    /// used to avoid assertions.
    pub fn displaced_endpoint<A: Axis, Sc: Scale>(&self) -> Option<Endpoint> {
        let a = self.curr_edges.get::<A>().inbound_vertex();
        let b = self
            .last_verts
            .get::<A>()
            .into_endpoint::<Sc>(a.endpoint(), self.len::<A>());
        Endpoint::from_partial_cmp(a, b)
    }

    /// Return the inbound [`Endpoint`] of the edge on axis `A`.
    pub fn inbound_endpoint<A: Axis>(&self) -> Endpoint {
        self.curr_edges.get::<A>().inbound_vertex().endpoint()
    }

    /// Set the [`Vertex`] for the given axis `A`.
    pub fn set_vertex<A: Axis>(&mut self, vertex: Vertex<A>) {
        self.curr_edges.get_mut::<A>().set_vertex(vertex.cast());
    }

    /// Get the collider size.
    pub fn size(&self) -> AxisVec<f32> {
        AxisVec::new(self.curr_edges.x().len(), self.curr_edges.y().len())
    }

    /// Get the `f32` position on the given axis. See [`Edge::lower_pos`].
    pub fn pos<A: Axis, Sc: Scale>(&self) -> f32 {
        self.curr_edges.get::<A>().lower_pos::<Sc>()
    }

    /// Return an `AxisVec<f32>` containing the top-left position of both axes. See [`Edge::lower_pos`].
    pub fn pos_vec<Sc: Scale>(&self) -> AxisVec<f32> {
        self.curr_edges.map(|edge| edge.lower_pos::<Sc>())
    }

    /// Get the [`Edge`] for axis `A`.
    pub fn edge<A: Axis>(&self) -> Edge<A> {
        self.curr_edges.get::<A>().cast()
    }

    /// Get the [`Vertex`] for the given `end` of axis `A`.
    pub fn vertex<A: Axis, Sc: Scale>(&self, end: Endpoint) -> Vertex<A> {
        self.curr_edges.get::<A>().vertex::<Sc>(end).cast()
    }

    /// Get the last [`Vertex`] for the given `end` of axis `A`.
    pub fn last_vertex<A: Axis, Sc: Scale>(&self, end: Endpoint) -> Vertex<A> {
        self.last_verts
            .get::<A>()
            .into_endpoint::<Sc>(end, self.len::<A>())
            .cast()
    }

    /// Get the [`Vertex`] tile index for the given `end` of axis `A`.
    pub fn index<A: Axis, Sc: Scale>(&self, end: Endpoint) -> i32 {
        self.vertex::<A, Sc>(end).index()
    }

    /// Get the last [`Vertex`] tile index for the given `end` of axis `A`.
    pub fn last_index<A: Axis, Sc: Scale>(&self, end: Endpoint) -> i32 {
        self.last_verts
            .get::<A>()
            .into_edge(self.len::<A>())
            .index::<Sc>(end)
    }

    /// Get the [`Edge`] tile range for the given axis `A`.
    pub fn index_range<A: Axis, Sc: Scale>(&self) -> Range<i32> {
        self.edge::<A>().index_range::<Sc>()
    }

    /// Get the length of the edge on the given axis `A`.
    pub fn len<A: Axis>(&self) -> f32 {
        self.curr_edges.get::<A>().len()
    }

    /// Return an `AxisVec<f32>` containing the length on both axes.
    pub fn len_vec(&self) -> AxisVec<f32> {
        self.curr_edges.map(|edge| edge.len())
    }

    /// Get the difference between the length of the intersecting tiles for the edge of axis `A`.
    /// Returns zero if the edge is a multiple of the [`Scale`].
    pub fn padding<A: Axis, Sc: Scale>(&self) -> f32 {
        self.curr_edges.get::<A>().padding::<Sc>()
    }

    /// Returns `true` if the edge is a multiple of the [`Scale`].
    pub fn is_not_padded<A: Axis, Sc: Scale>(&self) -> bool {
        self.curr_edges.get::<A>().is_not_padded::<Sc>()
    }

    /// Returns `true` if the given axis endpoint is on a tile bound.
    pub fn is_on_bound<A: Axis, Sc: Scale>(&self, end: Endpoint) -> bool {
        self.vertex::<A, Sc>(end).is_on_bound::<Sc>()
    }

    /// Returns `true` if the given vertex [`Endpoint`] intersects the given tile `index`.
    pub fn intersects_index<A: Axis, Sc: Scale>(&self, end: Endpoint, index: i32) -> bool {
        self.index::<A, Sc>(end) == index
    }

    /// Returns `true` if the given vertex [`Endpoint`] intersects the given [`Vertex`] tile index.
    pub fn intersects_vertex_index<A: Axis, Sc: Scale>(&self, vert: Vertex<A>) -> bool {
        vert.index() == self.index::<A, Sc>(vert.endpoint())
    }

    /// Returns `true` if the index of the given vertex [`Endpoint`] has changed (relative to the
    /// last call to [`State::update`]).
    pub fn index_changed<A: Axis, Sc: Scale>(&self, end: Endpoint) -> bool {
        self.index::<A, Sc>(end) != self.last_index::<A, Sc>(end)
    }

    /// Returns `true` if the given vertex **index** is greater than that of the current state. This
    /// function is primarily used in `#[cfg(debug_assertions)]`.
    pub fn lt_vertex_index<A: Axis, Sc: Scale>(&self, vert: Vertex<A>) -> bool {
        let bound = vert.endpoint();
        bound.partial_gt(vert.index(), &self.index::<A, Sc>(bound))
    }

    /// Returns `true` if the given vertex **offset** is greater than that of the current state.
    /// This function is primarily used in `#[cfg(debug_assertions)]`.
    pub fn lt_vertex_offset<A: Axis, Sc: Scale>(&self, vert: Vertex<A>) -> bool {
        let bound = vert.endpoint();
        bound.partial_gt(vert.offset(), &self.vertex::<A, Sc>(bound).offset())
    }

    /// Detach the endpoint of axis `A` if *stale*. How an endpoint is determined stale is left to
    /// the caller by providing two predicate functions, `state_filter` for `State` assertions and
    /// and `insp_filter` for [`TileMapInspect`] assertions. `state` filter is used for state
    /// assertions that exit early prior to checking the tilemap (returning [`DetachOp`] indicating
    /// the next operation) while while `insp_filter` receives the [`TileMapInspect`] type returning
    /// `true` if detaching.
    ///
    /// This function returns the [`TileMapInspect`] type or `None` if the tilemap was not checked.
    /// `detach_stale` if often used when displacing out of a tile while attached on the transpose
    /// axis.
    pub fn detach_stale<A, Sc, Map, Ins, F0, F1>(
        &mut self,
        scene: &Scene<Sc, Map>,
        state_filter: F0,
        insp_filter: F1,
    ) -> Option<Ins>
    where
        A: Axis,
        Sc: Scale,
        Map: TileMapView<A::T>,
        Ins: TileMapInspect<A::T, Map>,
        F0: FnOnce(&Self, Endpoint) -> DetachOp,
        F1: FnOnce(&Self, Endpoint, &Ins) -> bool,
    {
        if let Some(end) = self.attached_endpoint::<A>() {
            match state_filter(self, end) {
                DetachOp::Break => {}
                DetachOp::Detach => {
                    self.detach::<A>();
                }
                DetachOp::CheckNextTiles => {
                    let index = end.incr(self.index::<A, Sc>(end));
                    let t_range = self.index_range::<A::T, Sc>();

                    match SliceIndex::try_new(index, t_range, !end, scene.map.size()) {
                        Ok(index) => {
                            let insp = Ins::inspect(&scene.map, &index);
                            if insp_filter(self, end, &insp) {
                                self.detach::<A>();
                            }
                            return Some(insp);
                        }
                        Err(_) => {
                            // Assert if attached to the inside or outside of a map bound.
                            match end.partial_gt(index, &(scene.map.bound_index::<A>(end) as i32)) {
                                // Attached to a map bound from the inside.
                                true => {}
                                // Attached to a map bound from the outside.
                                false => self.detach::<A>(),
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Set the last position to the current.
    pub fn update(&mut self) {
        self.last_verts = self.curr_edges.map(|edge| edge.inbound_vertex());
    }

    /// Consume the `State`, apply all changes. Requires a tuple containing mutable *x* and *y* axis
    /// positions and a mutable collision [`AxisMask`]. See [`StatePayloadMut`] for tuple patterns.
    pub fn apply<Sc: Scale>(self, mut payload: impl StatePayloadMut<R>) {
        payload.apply::<Sc>(self);
    }
}

/// # DetachOp
///
/// Detach operation for [`State::detach_stale`]. Specifies the next operation during `state_filter`
/// assertions.
pub enum DetachOp {
    /// Do not detach.
    Break,
    /// Detach immediately.
    Detach,
    /// Check for a collision on the next non-intersecting tiles of the attached endpoint. Calls the
    /// next filtering function with the [`ViewInspect`] type.
    CheckNextTiles,
}

/// # StatePayload
///
/// State instantiation payload ([`State::new`]). Implemented on `(position, size, attmask, resource)`
/// tuples where each variables is represented as follows:
/// - `position`: "`f32, f32`", "`(f32, f32)`" or "`AxisVec<f32>`",
/// - `size`: "`f32, f32`", "`(f32, f32)`" or "`AxisVec<f32>`",
/// - `attmask`: `AxisMask`,
/// - `resource`: The generic type `R` variable in [`State<R>`], or omitted if `R` is `()`.
pub trait StatePayload<R> {
    /// Split the payload into a tuple containing the `f32` position, size and resource respectively.
    fn split_payload(self) -> (AxisVec<f32>, AxisVec<f32>, AxisMask, R);
}

impl StatePayload<()> for (f32, f32, f32, f32, AxisMask) {
    fn split_payload(self) -> (AxisVec<f32>, AxisVec<f32>, AxisMask, ()) {
        ((self.0, self.1).into(), (self.2, self.3).into(), self.4, ())
    }
}

impl StatePayload<()> for (AxisVec<f32>, AxisVec<f32>, AxisMask) {
    fn split_payload(self) -> (AxisVec<f32>, AxisVec<f32>, AxisMask, ()) {
        (self.0, self.1, self.2, ())
    }
}

impl StatePayload<()> for ((f32, f32), (f32, f32), AxisMask) {
    fn split_payload(self) -> (AxisVec<f32>, AxisVec<f32>, AxisMask, ()) {
        (self.0.into(), self.1.into(), self.2, ())
    }
}

impl<R> StatePayload<R> for (f32, f32, f32, f32, AxisMask, R) {
    fn split_payload(self) -> (AxisVec<f32>, AxisVec<f32>, AxisMask, R) {
        (
            (self.0, self.1).into(),
            (self.2, self.3).into(),
            self.4,
            self.5,
        )
    }
}

impl<R> StatePayload<R> for (AxisVec<f32>, AxisVec<f32>, AxisMask, R) {
    fn split_payload(self) -> (AxisVec<f32>, AxisVec<f32>, AxisMask, R) {
        (self.0, self.1, self.2, self.3)
    }
}

impl<R> StatePayload<R> for ((f32, f32), (f32, f32), AxisMask, R) {
    fn split_payload(self) -> (AxisVec<f32>, AxisVec<f32>, AxisMask, R) {
        (self.0.into(), self.1.into(), self.2, self.3)
    }
}

/// # StatePayloadMut
///
/// State mutation payload ([`State::apply`]). Implemented on `(position, attmask, resource)` tuples
/// where each variables is represented as follows:
/// - `position`: "`&mut f32, &mut f32`", "`(&mut f32, &mut f32)`" "`&mut (f32, f32)`",
/// "`AxisVec<&mut f32>`" or "`&mut AxisVec<f32>`",
/// - `attmask`: "`&mut AxisMask`",
/// - `resource`: The generic type `R` variable in [`State<R>`], or omitted if `R` is `()`.
pub trait StatePayloadMut<R> {
    /// Apply the changes in [`State`] to the tuple variables. Consumes `state`.
    fn apply<Sc: Scale>(&mut self, state: State<R>);
}

impl StatePayloadMut<()> for (&mut f32, &mut f32, &mut AxisMask) {
    fn apply<Sc: Scale>(&mut self, state: State<()>) {
        let pos = state.pos_vec::<Sc>();
        *self.0 = pos.x();
        *self.1 = pos.y();
        *self.2 = state.curr_attmask;
    }
}

impl StatePayloadMut<()> for (AxisVec<&mut f32>, &mut AxisMask) {
    fn apply<Sc: Scale>(&mut self, state: State<()>) {
        let pos = state.pos_vec::<Sc>();
        **self.0.x_mut() = pos.x();
        **self.0.y_mut() = pos.y();
        *self.1 = state.curr_attmask;
    }
}

impl StatePayloadMut<()> for (&mut AxisVec<f32>, &mut AxisMask) {
    fn apply<Sc: Scale>(&mut self, state: State<()>) {
        let pos = state.pos_vec::<Sc>();
        *self.0.x_mut() = pos.x();
        *self.0.y_mut() = pos.y();
        *self.1 = state.curr_attmask;
    }
}

impl StatePayloadMut<()> for ((&mut f32, &mut f32), &mut AxisMask) {
    fn apply<Sc: Scale>(&mut self, state: State<()>) {
        let pos = state.pos_vec::<Sc>();
        *self.0.0 = pos.x();
        *self.0.1 = pos.y();
        *self.1 = state.curr_attmask;
    }
}

impl StatePayloadMut<()> for (&mut (f32, f32), &mut AxisMask) {
    fn apply<Sc: Scale>(&mut self, state: State<()>) {
        let pos = state.pos_vec::<Sc>();
        self.0.0 = pos.x();
        self.0.1 = pos.y();
        *self.1 = state.curr_attmask;
    }
}

impl<R> StatePayloadMut<R> for (&mut f32, &mut f32, &mut AxisMask, &mut R) {
    fn apply<Sc: Scale>(&mut self, state: State<R>) {
        let pos = state.pos_vec::<Sc>();
        *self.0 = pos.x();
        *self.1 = pos.y();
        *self.2 = state.curr_attmask;
        *self.3 = state.res;
    }
}

impl<R> StatePayloadMut<R> for (AxisVec<&mut f32>, &mut AxisMask, &mut R) {
    fn apply<Sc: Scale>(&mut self, state: State<R>) {
        let pos = state.pos_vec::<Sc>();
        **self.0.x_mut() = pos.x();
        **self.0.y_mut() = pos.y();
        *self.1 = state.curr_attmask;
        *self.2 = state.res;
    }
}

impl<R> StatePayloadMut<R> for (&mut AxisVec<f32>, &mut AxisMask, &mut R) {
    fn apply<Sc: Scale>(&mut self, state: State<R>) {
        let pos = state.pos_vec::<Sc>();
        *self.0.x_mut() = pos.x();
        *self.0.y_mut() = pos.y();
        *self.1 = state.curr_attmask;
        *self.2 = state.res;
    }
}

impl<R> StatePayloadMut<R> for ((&mut f32, &mut f32), &mut AxisMask, &mut R) {
    fn apply<Sc: Scale>(&mut self, state: State<R>) {
        let pos = state.pos_vec::<Sc>();
        *self.0.0 = pos.x();
        *self.0.1 = pos.y();
        *self.1 = state.curr_attmask;
        *self.2 = state.res;
    }
}

impl<R> StatePayloadMut<R> for (&mut (f32, f32), &mut AxisMask, &mut R) {
    fn apply<Sc: Scale>(&mut self, state: State<R>) {
        let pos = state.pos_vec::<Sc>();
        self.0.0 = pos.x();
        self.0.1 = pos.y();
        *self.1 = state.curr_attmask;
        *self.2 = state.res;
    }
}

#[cfg(test)]
mod tests {
    use tilebound_plane::scale::ConSc;

    use super::*;

    type Sc = ConSc<16>;

    #[test]
    fn with_last_position() {
        let state0: State<()> = State::new::<Sc>(((0.0, 0.0), (1.0, 1.0), AxisMask::NONE))
            .with_last_pos::<Sc>((-10.0, 10.0));
        let state1: State<()> = State::new::<Sc>(((0.0, 0.0), (1.0, 1.0), AxisMask::NONE))
            .with_last_pos::<Sc>((10.0, -10.0));

        assert_eq!(
            state0.curr_edges.x().inbound_vertex(),
            Vertex::from_pos::<Sc>(1.0, Endpoint::Upper)
        );
        assert_eq!(
            state0.last_verts.x(),
            Vertex::from_pos::<Sc>(-10.0, Endpoint::Upper)
        );
        assert_eq!(
            state0.curr_edges.y().inbound_vertex(),
            Vertex::from_pos::<Sc>(0.0, Endpoint::Lower)
        );
        assert_eq!(
            state0.last_verts.y(),
            Vertex::from_pos::<Sc>(10.0, Endpoint::Lower)
        );
        assert_eq!(
            state1.curr_edges.x().inbound_vertex(),
            Vertex::from_pos::<Sc>(0.0, Endpoint::Lower)
        );
        assert_eq!(
            state1.last_verts.x(),
            Vertex::from_pos::<Sc>(10.0, Endpoint::Lower)
        );
        assert_eq!(
            state1.curr_edges.y().inbound_vertex(),
            Vertex::from_pos::<Sc>(1.0, Endpoint::Upper)
        );
        assert_eq!(
            state1.last_verts.y(),
            Vertex::from_pos::<Sc>(-10.0, Endpoint::Upper)
        );
    }
}
