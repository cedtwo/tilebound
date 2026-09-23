use std::fmt::Debug;
use std::ops::Range;

use crate::ctx::bnd_box::BoundBoxView;
use crate::plane::axis::{Axis, AxisMask, AxisVec, AxisX, AxisY};
use crate::plane::endpoint::Endpoint;
use crate::plane::scale::Scale;
use crate::topology::edge::{Edge, RawEdge};
use crate::topology::vertex::{RawVertex, Vertex};
use crate::view::index::SliceIndex;
use crate::view::inspect::TileMapInspect;
use crate::view::tilemap::TileMapView;

use crate::ctx::scene::Scene;

/// # State
///
/// Bounding box variable intermediate for displacement operations.
///
/// `State` stores bounding box variables for mutation and assertion during operations. It uses
/// [`Vertex`] positions for assertions (especially in relation to tile indicex), mutation,
/// alignment and other relevant operations.
///
/// `State` Accepts a [`BoundBoxView`] implementing type exposing bounding box position, size and
/// attachments (usually
/// supplied via [`BoundBox`](crate::ctx::bnd_box::BoundBox)). [`State`] (and subsequently
/// `BoundBox`) accept a generic resource `R`, accessible through the [`State::res`] and
/// [`State::res_mut`] methods to (optionally) provide additional data during displacement.
///
/// ## Example
///
/// ```
/// # use tilebound::ctx::state::State;
/// # use tilebound::ctx::bnd_box::{BoundBox, BoundBoxView};
/// # use tilebound::plane::scale::ConSc;
/// # use tilebound::plane::endpoint::Endpoint;
/// # use tilebound::plane::axis::{AxisVec, AxisMask};
/// # use tilebound::topology::vertex::Vertex;
/// type Sc = ConSc<16>;
/// // A bounding box of `(16.0 * 16.0)` units in size (equal to a single tile in eize).
/// let mut bounding_box = BoundBox::new((0.0, 0.0), (16.0, 16.0), AxisMask::NONE);
/// let mut state = State::new::<Sc, _>(&bounding_box);
///
/// // Assert the left and right of the bounding box is in tile index `0`.
/// assert_eq!(state.index::<_, Sc>(Endpoint::LEFT), 0);
/// assert_eq!(state.index::<_, Sc>(Endpoint::RIGHT), 0);
///
/// // Get the vertex on the right of the bounding box, and translate it by `1.0`.
/// let mut vertex = state.vertex::<_, Sc>(Endpoint::RIGHT);
/// vertex.translate::<Sc>(1.0);
///
/// // Update the state and check the right of the bounding box is now in tile index `1`.
/// state.set_vertex(vertex);
/// assert_eq!(state.index::<_, Sc>(Endpoint::RIGHT), 1);
///
/// // Commit changes back to the state and assert the new position.
/// state.apply::<Sc, _>(&mut bounding_box);
/// assert_eq!(bounding_box.pos(), AxisVec::new(1.0, 0.0));
/// ```
///
/// As demonstrated above, `State` is primarily used for updating a position with
/// [`State::set_vertex`], while attachments are set with[`State::attach`] and [`State::detach`].
/// Use [`State::pos`] or [`State::pos_vec`] to get the top/left position(s).
///
/// Note that `State` stores the last vertices (`last_verts`) for assertions on recent changes
/// (eg. [`State::index_changed`]). These variables are set at instantiation and updated with
/// [`State::update`].
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
    /// Create a new `State` from a [`BoundBoxView`] implementing type. See [`State`] and
    /// [`BoundBox`] documentation for examples.
    pub fn new<'a, Sc, P>(payload: &P) -> Self
    where
        Sc: Scale,
        R: Clone,
        P: BoundBoxView<R = R>,
    {
        let curr_edges = Self::bounding_box::<Sc>(payload.pos(), payload.len());
        let last_verts = curr_edges.map(|e| e.take_vertex());
        Self {
            curr_edges,
            curr_attmask: payload.attmask(),
            last_verts,
            res: payload.res().clone(),
        }
    }

    /// Return the `State` with the last position set. Handles orientating the current position away
    /// from the last position. This is generally only useful for debug assertions.
    pub fn with_last_pos<Sc: Scale>(mut self, last_pos: impl Into<AxisVec<f32>>) -> Self {
        let last_pos = last_pos.into();
        let ends = self.pos_vec::<Sc>().map_with(last_pos, |curr, last| {
            Endpoint::from_partial_cmp(curr, last).unwrap_or_default()
        });
        self.curr_edges.x_mut().set_inbound_end::<Sc>(ends.x);
        self.curr_edges.y_mut().set_inbound_end::<Sc>(ends.y);
        self.last_verts = last_pos.map_with(ends, |pos, end| Vertex::from_pos::<Sc>(pos, end));
        debug_assert_eq!(self.curr_edges.x.end(), self.last_verts.x.endpoint());
        debug_assert_eq!(self.curr_edges.y.end(), self.last_verts.y.endpoint());

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
            Edge::from_pos::<S>(pos.get::<AxisX>(), len.get::<AxisX>(), Endpoint::LOWER),
            Edge::from_pos::<S>(pos.get::<AxisY>(), len.get::<AxisY>(), Endpoint::LOWER),
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
    pub fn attach<A: Axis>(&mut self, end: Endpoint<A>) {
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

    /// Returns `true` if attached on the given [`Endpoint`].
    pub fn end_is_attached<A: Axis>(&self, end: Endpoint<A>) -> bool {
        self.curr_attmask.end_is_set::<A>(end)
    }

    /// Returns `true` if attached on either endpoint of the given axis `A`.
    pub fn axis_is_attached<A: Axis>(&self) -> bool {
        self.curr_attmask.any_on_axis::<A>()
    }

    /// Return the attached [`Endpoint<A>`] of axis `A`.
    pub fn attached_endpoint<A: Axis>(&self) -> Option<Endpoint<A>> {
        self.curr_attmask.first_end::<A>()
    }

    /// Return the displaced [`Endpoint<A>`] of axis `A` or `None` if no displacement has occured since
    /// the last call to [`State::update`]. If certain a new position has been set with a [`Vertex`]
    /// of an [`Endpoint<A>`] equal to displacement, then calling [`State::inbound_endpoint`] can be
    /// used to avoid assertions.
    pub fn displaced_endpoint<A: Axis, Sc: Scale>(&self) -> Option<Endpoint<A>> {
        let a = self.curr_edges.get::<A>().inbound_vertex();
        let b = self
            .last_verts
            .get::<A>()
            .into_endpoint::<Sc>(a.endpoint(), self.len::<A>());
        Endpoint::from_partial_cmp(a, b)
    }

    /// Return the inbound [`Endpoint`].
    pub fn inbound_endpoint<A: Axis>(&self) -> Endpoint<A> {
        self.curr_edges
            .get::<A>()
            .inbound_vertex()
            .cast()
            .endpoint()
    }

    /// Set the [`Vertex`] for the given axis `A`.
    pub fn set_vertex<A: Axis>(&mut self, vertex: Vertex<A>) {
        self.curr_edges.get_mut::<A>().set_vertex(vertex.cast());
    }

    /// Get the collider size.
    pub fn size(&self) -> AxisVec<f32> {
        AxisVec::new(self.curr_edges.x.len(), self.curr_edges.y.len())
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

    /// Get the [`Vertex`] for the given [`Endpoint`].
    pub fn vertex<A: Axis, Sc: Scale>(&self, end: Endpoint<A>) -> Vertex<A> {
        self.curr_edges.get::<A>().cast().vertex::<Sc>(end).cast()
    }

    /// Get the last [`Vertex`] for the given `end` of axis `A`.
    pub fn last_vertex<A: Axis, Sc: Scale>(&self, end: Endpoint<A>) -> Vertex<A> {
        self.last_verts
            .get::<A>()
            .cast()
            .into_endpoint::<Sc>(end, self.len::<A>())
            .cast()
    }

    /// Get the [`Vertex`] tile index for the given `end` of axis `A`.
    pub fn index<A: Axis, Sc: Scale>(&self, end: Endpoint<A>) -> i32 {
        self.vertex::<A, Sc>(end).index()
    }

    /// Get the last [`Vertex`] tile index for the given `end` of axis `A`.
    pub fn last_index<A: Axis, Sc: Scale>(&self, end: Endpoint<A>) -> i32 {
        self.last_verts
            .get::<A>()
            .into_edge(self.len::<A>())
            .cast()
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
    pub fn is_on_bound<A: Axis, Sc: Scale>(&self, end: Endpoint<A>) -> bool {
        self.vertex::<A, Sc>(end).is_on_bound::<Sc>()
    }

    /// Returns `true` if the given vertex [`Endpoint<A>`] intersects the given tile `index`.
    pub fn intersects_index<A: Axis, Sc: Scale>(&self, end: Endpoint<A>, index: i32) -> bool {
        self.index::<A, Sc>(end) == index
    }

    /// Returns `true` if the given vertex [`Endpoint<A>`] intersects the given [`Vertex`] tile index.
    pub fn intersects_vertex_index<A: Axis, Sc: Scale>(&self, vert: Vertex<A>) -> bool {
        vert.index() == self.index::<A, Sc>(vert.endpoint())
    }

    /// Returns `true` if the index of the given vertex [`Endpoint<A>`] has changed (relative to the
    /// last call to [`State::update`]).
    pub fn index_changed<A: Axis, Sc: Scale>(&self, end: Endpoint<A>) -> bool {
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
        F0: FnOnce(&Self, Endpoint<A>) -> DetachOp,
        F1: FnOnce(&Self, Endpoint<A>, &Ins) -> bool,
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

    /// Consumes the `State`, committing changes back to a [`BoundBoxView`] implementing type. See
    /// [`State`] and [`BoundBox`](super::bnd_box::BoundBox) documentation for examples.
    pub fn apply<Sc, P>(self, payload: &mut P)
    where
        Sc: Scale,
        R: Clone,
        P: BoundBoxView<R = R>,
    {
        *payload.pos_mut() = self.pos_vec::<Sc>();
        *payload.attmask_mut() = self.attmask();
        *payload.res_mut() = self.res().clone();
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

#[cfg(test)]
mod tests {
    use crate::ctx::bnd_box::BoundBox;
    use crate::plane::scale::ConSc;

    use super::*;

    type Sc = ConSc<16>;

    #[test]
    fn with_last_position() {
        let payload_0 = BoundBox::new((0.0, 0.0), (1.0, 1.0), AxisMask::NONE);
        let state_0: State<()> = State::new::<Sc, _>(&payload_0).with_last_pos::<Sc>((-10.0, 10.0));
        let payload_1 = BoundBox::new((0.0, 0.0), (1.0, 1.0), AxisMask::NONE);
        let state_1: State<()> = State::new::<Sc, _>(&payload_1).with_last_pos::<Sc>((10.0, -10.0));

        assert_eq!(
            state_0.curr_edges.x.inbound_vertex().cast::<AxisX>(),
            Vertex::from_pos::<Sc>(1.0, Endpoint::RIGHT)
        );
        assert_eq!(
            state_0.last_verts.x.cast::<AxisX>(),
            Vertex::from_pos::<Sc>(-10.0, Endpoint::RIGHT)
        );
        assert_eq!(
            state_0.curr_edges.y.inbound_vertex().cast::<AxisY>(),
            Vertex::from_pos::<Sc>(0.0, Endpoint::TOP)
        );
        assert_eq!(
            state_0.last_verts.y.cast::<AxisY>(),
            Vertex::from_pos::<Sc>(10.0, Endpoint::TOP)
        );
        assert_eq!(
            state_1.curr_edges.x.inbound_vertex().cast::<AxisX>(),
            Vertex::from_pos::<Sc>(0.0, Endpoint::LEFT)
        );
        assert_eq!(
            state_1.last_verts.x.cast::<AxisX>(),
            Vertex::from_pos::<Sc>(10.0, Endpoint::LEFT)
        );
        assert_eq!(
            state_1.curr_edges.y.inbound_vertex().cast::<AxisY>(),
            Vertex::from_pos::<Sc>(1.0, Endpoint::BOTTOM)
        );
        assert_eq!(
            state_1.last_verts.y.cast::<AxisY>(),
            Vertex::from_pos::<Sc>(-10.0, Endpoint::BOTTOM)
        );
    }
}
