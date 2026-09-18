use std::{marker::PhantomData, ops::ControlFlow};

use crate::collision::{Collision, EdgeRange};
use crate::handlers::{Attach, Slide, VertexHandler};
use crate::prelude::State;
use crate::tile::VertexTile;
use tilebound::ops::*;
use tilebound::plane::axis::AxisVec;
use tilebound::prelude::*;
use tilebound::schema::brk::Break;
use tilebound::schema::state::DetachOp;
use tilebound::topology::vertex::Vertex;

/// # Context
///
/// `Context` is a [`Scene`] wrapper exposing methods for displacement handlers (namely
/// [`Context::attach_handler`] and [`Context::slide_handler`]). Displacement operations are exposed
/// by calling one of the handler methods, returning a [`ContextHandler`] which exposes the
/// [`ContextHandler::sweep_by`] and [`ContextHandler::sweep_to`] displacement methods. Displacement
/// requires a mutable reference to a rectangle [`State`] that will be mutated with the result of
/// displacement.
///
/// # Usage
///
/// `Context` requires a [`Scene`] (either instantiated independently, or by calling [`Context::new`])
/// and a [`State`]. See the documentation for [`Scene`] and [`State`]
///
/// ```rust
/// # use std::assert_matches;
/// # use tilebound_halfslope::prelude::*;
/// # #[cfg(feature = "arraymap")]
/// # {
/// // A 2*2 map with a single tile in the top-right triangle tile (represented as a u8 for this example).
/// const MAP: ArrayMap<2, 2, 4, u8> = ArrayMap::new_unchecked([
///     00, 13,
///     00, 00
/// ]);
///
/// // Declare a constant tile size (a 16 unit square).
/// type Sc = ConSc<16>;
/// // Declare a `Context`, passing in the map and declaring any solid map bounds.
/// let ctx = Context::<Sc, _>::new(&MAP, AxisMask::NONE);
///
/// // Declare our rectangle variables.
/// let size = (8.0, 8.0); // An 8*8 unit square.
/// let mut pos = (0.0, 0.0); // Positions the top-left of the rectangle in the top-left of the map.
/// let mut attmask = AxisMask::NONE; // A AxisMask for representing collisions.
/// let mut trimask = VertexMask::NONE; // A VertexMask representing which vertices of the rectangle intersect a triangle tile.
///
/// // Create a state from our collider variables.
/// let mut state = State::new::<Sc>((pos, size, attmask, trimask));
///
/// // Displace 16.0 units to the right (one exact tile) using the `Slide` handler (slides on intersecting a triangle hypotenuse).
/// let brk = ctx.slide_handler().sweep_by::<AxisX>(&mut state, 16.0);
///
/// assert_matches!(brk, Break::ReachedTarget); // Assert we reached the target.
/// assert_eq!(state.pos_vec::<Sc>(), (16.0, 8.0).into()); // Assert we displaced downward on reaching the triangle tile.
/// assert_eq!(state.attmask(), AxisMask::NONE); // Assert we are not colliding (colliding prevents displacement).
///
/// // Displace 8.0 units up (half a tile) using the `Attach` handler (attaches on intersecting a triangle hypotenuse).
/// let brk = ctx.attach_handler().sweep_by::<AxisY>(&mut state, -8.0);
///
/// assert_matches!(brk, Break::Collision(Collision::TriangleHypotenuse)); // Assert this time we collided with the triangle hypotenuse.
/// assert_eq!(state.pos_vec::<Sc>(), (16.0, 8.0).into()); // Assert we didn't displace at all (we were already on the hypotenuse).
/// assert_eq!(state.attmask(), AxisMask::TOP); // Assert we are colliding at the top.
///
/// // Apply changes back to our variables.
/// state.apply::<Sc>((&mut pos, &mut attmask, &mut trimask));
/// # }
/// ```
pub struct Context<Sc, Map>(Scene<Sc, Map>);

impl<Sc, Map> Context<Sc, Map> {
    /// Create a new `Context` with the given `map` and `map_bounds`.
    pub fn new(map: Map, map_bounds: AxisMask) -> Self {
        Self(Scene::new(map, map_bounds))
    }

    /// Get a reference to the [`Scene`].
    pub fn scene(&self) -> &Scene<Sc, Map> {
        &self.0
    }

    /// Get a mutable reference to the [`Scene`].
    pub fn scene_mut(&mut self) -> &mut Scene<Sc, Map> {
        &mut self.0
    }

    /// Return the map bound [`AxisMask`].
    pub fn map_bounds(&self) -> AxisMask {
        self.0.map_bounds
    }

    /// Return a mutable reference to the map bound [`AxisMask`].
    pub fn map_bounds_mut(&mut self) -> &mut AxisMask {
        &mut self.0.map_bounds
    }

    /// Use the [`Attach`] vertex handler. See [`Attach`] and [`Slide`].
    pub fn attach_handler<'a>(&'a self) -> ContextHandler<'a, Sc, Map, Attach> {
        ContextHandler::new(&self.0)
    }

    /// Use the [`Slide`] vertex handler. See [`Attach`] and [`Slide`].
    pub fn slide_handler<'a>(&'a self) -> ContextHandler<'a, Sc, Map, Slide> {
        ContextHandler::new(&self.0)
    }
}

/// # ContextHander
///
/// `ContextHandler` implements vertex handlers ([`VertexHandler`]) with regards to the generic
/// marker `H`. While it can be instantiated alone, it may be more idiomatic to call
/// [`Context::attach_handler`] or [`Context::slide_handler`].
pub struct ContextHandler<'a, Sc, Map, H>(&'a Scene<Sc, Map>, PhantomData<H>);

impl<'a, Sc, Map, H> ContextHandler<'a, Sc, Map, H> {
    /// Create a new `ContextHandler` from the given [`Scene`] reference.
    pub fn new(scene: &'a Scene<Sc, Map>) -> Self {
        Self(scene, PhantomData)
    }
}

impl<'a, Sc: Scale, Map: TileMap, H> ContextHandler<'a, Sc, Map, H> {
    /// Attempt to displace the collider by the given `delta`, checking all intersecting tiles of
    /// the delta. Where colliding with a tilebound, returns a [`EdgeRange`] on the **transpose**
    /// axis (eg. returns a *y* axis *column* when displacing and colliding on the *x* axis).
    pub fn sweep_by<A: Axis>(&self, state: &mut State, delta: f32) -> Break<Collision<A::T>>
    where
        H: VertexHandler<A, Sc, Map>,
        Map: TileMapView<A, El: VertexTile>,
        Map: TileMapView<A::T, El: VertexTile>,
    {
        let Some(delta) = delta_by::<A, Sc, _>(state, delta) else {
            return Break::None;
        };
        state.detach::<A>();
        loop_outer_delta(delta, state, &self.0, Self::handle_vertex, Self::update)
    }

    /// Attempt to displace the nearest collider edge to the given position, checking all
    /// intersected tiles of the delta. Where colliding with a tilebound, returns a [`EdgeRange`] on
    /// the **transpose** axis (eg. returns a *y* axis *column* when displacing and colliding on the
    /// *x* axis).
    pub fn sweep_to<A: Axis>(&self, state: &mut State, pos: f32) -> Break<Collision<A::T>>
    where
        H: VertexHandler<A, Sc, Map>,
        Map: TileMapView<A, El: VertexTile>,
        Map: TileMapView<A::T, El: VertexTile>,
    {
        let Some(delta) = delta_to::<A, Sc, _>(state, pos) else {
            return Break::None;
        };
        state.detach::<A>();
        loop_outer_delta(delta, state, &self.0, Self::handle_vertex, Self::update)
    }

    /// Perform assertions on the [`State`] and [`Vertex`] target before passing to a displacement
    /// handler.
    pub fn handle_vertex<A: Axis>(
        tgt: Vertex<A>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Break<Collision<A::T>>>
    where
        H: VertexHandler<A, Sc, Map>,
        Map: TileMapView<A, El: VertexTile>,
        Map: TileMapView<A::T, El: VertexTile>,
    {
        match state.intersects_index::<A, Sc>(tgt.endpoint(), tgt.index()) {
            // Do not check collisions on already intersected tiles.
            true => Self::to_intersected_tile(tgt, state, &scene),
            // Check the tilemap view for collisions, or handle displacing over the map bounds.
            false => match inspect_intersected::<_, _, _, _, EdgeRange<A::T>>(tgt, state, scene) {
                Ok(edges) => match edges.inner_collision() {
                    true => {
                        state.attach::<A>(tgt.endpoint());
                        ControlFlow::Break(Collision::TileBound(edges))
                    }
                    false => H::to_next_tile(tgt, edges, state, scene),
                },
                Err(_) => Self::to_out_of_bounds(tgt, state, scene),
            },
        }
        .map_break(|c| Break::Collision(c))
    }

    fn to_intersected_tile<A: Axis>(
        tgt: Vertex<A>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>>
    where
        H: VertexHandler<A, Sc, Map>,
    {
        debug_assert!(!state.lt_vertex_index::<A, Sc>(tgt));
        debug_assert!(state.lt_vertex_offset::<A, Sc>(tgt));

        let adj_tris = state.res().isolate_edge::<A>(tgt.endpoint());

        if adj_tris.all_on_axis::<A::T>() {
            let t_range = state.index_range::<A::T, Sc>();
            H::handle_wedge(tgt, t_range, state, scene)
        } else if let Some(t_vert_sign) = adj_tris.any_first_end() {
            H::handle_tri(tgt, t_vert_sign, state, scene)
        } else {
            state.set_vertex(tgt);
            ControlFlow::Continue(())
        }
    }

    /// Assert the presence of a solid map bound and attach or continue displacement accordingly.
    fn to_out_of_bounds<A: Axis>(
        tgt: Vertex<A>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>> {
        match scene.map_bounds.end_is_set::<A>(tgt.endpoint()) {
            true => {
                state.attach::<A>(tgt.endpoint());
                Collision::MapBound.into()
            }
            false => {
                state.set_vertex(tgt);
                ControlFlow::Continue(())
            }
        }
    }

    fn update<A: Axis>(state: &mut State, scene: &Scene<Sc, Map>)
    where
        H: VertexHandler<A, Sc, Map>,
        Map: TileMapView<A, El: VertexTile>,
    {
        Self::detach_stale(state, scene);
        H::update(state, scene);
        state.update();
    }

    /// [`State::detach_stale`] logic for displacement on axis `A` (detaches the transpose axis).
    fn detach_stale<A: Axis>(state: &mut State, scene: &Scene<Sc, Map>) -> Option<EdgeRange<A>>
    where
        H: VertexHandler<A, Sc, Map>,
        Map: TileMapView<A, El: VertexTile>,
    {
        state.detach_stale::<A::T, Sc, _, _, _, _>(
            scene,
            |state, t_end| {
                let end = state.inbound_endpoint::<A>();
                // Left triangle hypotenuse.
                if state
                    .res()
                    .vertex_is_set(AxisVec::new_mapped::<A>(!end, t_end))
                {
                    DetachOp::Detach
                    // On a tile bound and index changed on the transpose axis or possibly leaving
                    // a triangle vertex.
                } else if state.is_on_bound::<A::T, Sc>(t_end)
                    && (state.index_changed::<A, Sc>(!end)
                        || state.last_vertex::<A, Sc>(!end).is_on_bound::<Sc>())
                {
                    DetachOp::CheckNextTiles
                } else {
                    DetachOp::Break
                }
            },
            |state, _, t_view: &EdgeRange<A>| {
                !t_view.inner_collision()
                    && ({
                        let end = state.inbound_endpoint::<A>();
                        let tris = t_view.trimask().isolate_axis::<A::T>();

                        // Only detach if:
                        // - not on a either tri vertex or,
                        // - on the next tri vertex but the collider is smaller than the gap (displaced).
                        !(tris.end_is_set::<A>(!end) && state.is_on_bound::<A, Sc>(!end)
                            || (tris.end_is_set::<A>(end)
                                && state.is_on_bound::<A, Sc>(end)
                                && state.is_not_padded::<A, Sc>()))
                    })
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use bitvec::prelude::*;

    use super::*;
    use crate::vertex_mask::VertexMask;

    type Sc = ConSc<16>;

    #[rustfmt::skip]
    const SOLID_TILE: ArrayMap<2, 2, 4, u8> = ArrayMap::new_unchecked([
        00, 00,
        00, 15,
    ]);

    #[rustfmt::skip]
    const TRI_TILE: ArrayMap<2, 2, 4, u8> = ArrayMap::new_unchecked([
        00, 00,
        00, 14,
    ]);

    #[rustfmt::skip]
    const WEDGE_TILES: ArrayMap<2, 2, 4, u8> = ArrayMap::new_unchecked([
        00, 00,
        11, 14,
    ]);

    #[test]
    fn dont_detach_on_solid_bound_nopad() {
        let mut state = State::new::<Sc>((
            (15.0, 0.0),
            (Sc::SCALE, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((16.0, 0.0));
        let scene = Scene::<Sc, _>::new(&SOLID_TILE, AxisMask::NONE);
        let update = ContextHandler::<Sc, _, Attach>::detach_stale::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::BOTTOM.into());
        assert_eq!(
            update,
            Some(EdgeRange::from_array(
                (0, 1),
                Endpoint::Lower,
                bitvec![0, 0, 1, 1]
            ))
        );
    }

    #[test]
    fn dont_detach_on_solid_bound_padded() {
        let mut state = State::new::<Sc>((
            (15.0, 0.0),
            (15.0, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((16.0, 0.0));
        let scene = Scene::<Sc, _>::new(&SOLID_TILE, AxisMask::NONE);
        let update = ContextHandler::<Sc, _, Attach>::detach_stale::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::BOTTOM.into());
        assert_eq!(update, None);
    }

    #[test]
    fn check_detach_on_leaving_tile() {
        let mut state = State::new::<Sc>((
            (16.0, 0.0),
            (Sc::SCALE, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((17.0, 0.0));
        let scene = Scene::<Sc, _>::new(&SOLID_TILE, AxisMask::NONE);
        let update = ContextHandler::<Sc, _, Attach>::detach_stale::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::BOTTOM.into());
        assert_eq!(
            update,
            Some(EdgeRange::from_array(
                (1, 1),
                Endpoint::Lower,
                bitvec![1, 1]
            ),)
        );
    }

    #[test]
    fn detach_on_leaving_tri_vertex_nopad() {
        let mut state = State::new::<Sc>((
            (15.0, 0.0),
            (Sc::SCALE, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((16.0, 0.0));
        let scene = Scene::<Sc, _>::new(&TRI_TILE, AxisMask::NONE);
        let update = ContextHandler::<Sc, _, Attach>::detach_stale::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::NONE.into());
        assert_eq!(
            update,
            Some(EdgeRange::from_array(
                (0, 1),
                Endpoint::Lower,
                bitvec![0, 0, 0, 1]
            ))
        );
    }

    #[test]
    fn detach_on_leaving_tri_vertex_padded() {
        let mut state = State::new::<Sc>((
            (16.0, 0.0),
            (15.0, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((17.0, 0.0));
        let scene = Scene::<Sc, _>::new(&TRI_TILE, AxisMask::NONE);
        let update = ContextHandler::<Sc, _, Attach>::detach_stale::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::NONE.into());
        assert_eq!(
            update,
            Some(EdgeRange::from_array(
                (1, 1),
                Endpoint::Lower,
                bitvec![0, 1]
            ))
        );
    }

    #[test]
    fn dont_detach_on_reaching_inner_tri_vertex() {
        let mut state = State::new::<Sc>((
            (16.0, 0.0),
            (Sc::SCALE, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((17.0, 0.0));
        let scene = Scene::<Sc, _>::new(&TRI_TILE, AxisMask::NONE);
        let update = ContextHandler::<Sc, _, Attach>::detach_stale::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::BOTTOM.into());
        assert_eq!(
            update,
            Some(EdgeRange::from_array(
                (1, 1),
                Endpoint::Lower,
                bitvec![0, 1]
            ))
        );
    }

    #[test]
    fn dont_detach_on_reaching_outer_tri_vertex() {
        let mut state = State::new::<Sc>((
            (0.0, 0.0),
            (Sc::SCALE * 2.0, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((16.0, 0.0));
        let scene = Scene::<Sc, _>::new(&WEDGE_TILES, AxisMask::NONE);
        ContextHandler::<Sc, _, Attach>::detach_stale::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::BOTTOM.into());
    }

    #[test]
    fn detach_on_reaching_outer_tri_vertex_after_gap() {
        let mut state = State::new::<Sc>((
            (0.0, 0.0),
            (Sc::SCALE - 1.0, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((16.0, 0.0));
        let scene = Scene::<Sc, _>::new(&WEDGE_TILES, AxisMask::NONE);
        ContextHandler::<Sc, _, Attach>::update::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::NONE.into());
    }

    #[test]
    fn detach_on_leaving_hypotenuse() {
        let mut state = State::new::<Sc>((
            (14.0, 1.0),
            (Sc::SCALE, Sc::SCALE),
            AxisMask::BOTTOM,
            VertexMask::NONE,
        ))
        .with_last_pos::<Sc>((15.0, 1.0));
        *state.res_mut() = VertexMask::BOTTOM_RIGHT_INCL.into();

        let scene = Scene::<Sc, _>::new(&TRI_TILE, AxisMask::NONE);
        ContextHandler::<Sc, _, Attach>::update::<AxisX>(&mut state, &scene);

        assert_eq!(state.attmask(), AxisMask::NONE.into());
    }
}
