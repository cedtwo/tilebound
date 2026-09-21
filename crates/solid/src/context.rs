use std::ops::ControlFlow;

use crate::collision::{Collision, TileRange};
use crate::tile::SolidTile;
use tilebound::ctx::brk::Break;
use tilebound::ctx::state::DetachOp;
use tilebound::ops::*;
use tilebound::prelude::*;
use tilebound::topology::vertex::Vertex;

/// # Context
///
/// `Context` is a [`Scene`] wrapper exposing methods for displacement (namely [`Context::sweep_by`]
/// and [`Context::sweep_to`]). Displacement requires a mutable reference to a rectangle [`State`]
/// that will be mutated with the result of displacement.
///
/// # Usage
///
/// `Context` requires a [`Scene`] (either instantiated independently, or by calling [`Context::new`])
/// and a [`State`]. See the documentation for [`Scene`] and [`State`]
///
/// ```rust
/// # use std::assert_matches;
/// # use tilebound_solid::prelude::*;
/// # #[cfg(feature = "arraymap")]
/// # {
/// // A 2*2 map with a single solid tile in the top-right.
/// const MAP: ArrayMap<2, 2, 4, u8> = ArrayMap::new_unchecked([
///     0, 1,
///     0, 0
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
/// let mut attmask = AxisMask::NONE; // An AxisMask for representing collisions.
///
/// // Create a state from our collider variables.
/// let mut state = State::new::<Sc>((pos, size, attmask));
///
/// let brk = ctx.sweep_by::<AxisX>(&mut state, 16.0); // Displace 16.0 units to the right (one exact tile).
///
/// assert_matches!(brk, Break::Collision(_)); // Assert we collided.
/// assert_eq!(state.pos_vec::<Sc>(), (8.0, 0.0).into()); // Assert we displaced only 8 units to the right.
/// assert_eq!(state.attmask(), AxisMask::RIGHT); // Assert we are colliding on the right.
///
/// let brk = ctx.sweep_by::<AxisY>(&mut state, 16.0); // Displace 16.0 units down (one exact tile).
///
/// assert_matches!(brk, Break::ReachedTarget); // Assert we reached the target.
/// assert_eq!(state.pos_vec::<Sc>(), (8.0, 16.0).into()); // Assert we displaced all 16 units down.
/// assert_eq!(state.attmask(), AxisMask::NONE); // Assert we are no longer colliding on the right.
///
/// // Apply changes back to our variables.
/// state.apply::<Sc>((&mut pos, &mut attmask));
/// # }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Context<Sc, Map>(Scene<Sc, Map>);

impl<Sc, Map> Context<Sc, Map> {
    /// Create a new `Context` with the given `map` and `map_bounds`.
    pub fn new(map: Map, map_bounds: AxisMask) -> Self {
        Self(Scene::new(map, map_bounds))
    }

    /// Create a new `Contest` from the given [`Scene`].
    pub fn from_scene(scene: Scene<Sc, Map>) -> Self {
        Self(scene)
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

    /// Consume the `Context` returning the inner [`Scene`].
    pub fn take(self) -> Scene<Sc, Map> {
        self.0
    }
}

impl<Sc: Scale, Map: TileMap> Context<Sc, Map> {
    /// Attempt to displace the collider by the given `delta`, checking all intersecting tiles of
    /// the delta. Where colliding with a tilebound, returns a [`TileRange`] on the **transpose**
    /// axis (eg. returns a *y* axis *column* when displacing and colliding on the *x* axis).
    pub fn sweep_by<A: Axis>(&self, state: &mut State, delta: f32) -> Break<Collision<A::T>>
    where
        Map: TileMapView<A, El: SolidTile>,
        Map: TileMapView<A::T, El: SolidTile>,
    {
        let Some(delta) = delta_by::<A, Sc, _>(state, delta) else {
            return Break::None;
        };
        state.detach::<A>();
        loop_outer_delta(
            delta,
            state,
            &self.0,
            Self::handle_vertex,
            Self::update::<A>,
        )
    }

    /// Attempt to displace the nearest collider edge to the given position, checking all
    /// intersected tiles along the delta. Where colliding with a tilebound, returns a [`TileRange`]
    /// on the **transpose** axis (eg. returns a *y* axis *column* when displacing and colliding
    /// on the *x* axis).
    pub fn sweep_to<A: Axis>(&self, state: &mut State, pos: f32) -> Break<Collision<A::T>>
    where
        Map: TileMapView<A, El: SolidTile>,
        Map: TileMapView<A::T, El: SolidTile>,
    {
        let Some(delta) = delta_to::<A, Sc, _>(state, pos) else {
            return Break::None;
        };
        state.detach::<A>();
        loop_outer_delta(
            delta,
            state,
            &self.0,
            Self::handle_vertex,
            Self::update::<A>,
        )
    }

    /// Perform assertions on the [`State`] and [`Vertex`] target before passing to a displacement
    /// handler.
    fn handle_vertex<A: Axis>(
        tgt: Vertex<A>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Break<Collision<A::T>>>
    where
        Map: TileMapView<A, El: SolidTile>,
        Map: TileMapView<A::T, El: SolidTile>,
    {
        match state.intersects_index::<A, Sc>(tgt.endpoint(), tgt.index()) {
            // Do not check collisions on already intersected tiles.
            true => Self::to_intersected_tile(tgt, state),
            // Check the tilemap view for collisions, or handle displacing over the map bounds.
            false => match scene.inspect_intersected::<_, _, TileRange<A::T>>(tgt, state) {
                Ok(tiles) => Self::to_next_tile(tgt, state, tiles),
                Err(_) => Self::to_out_of_bounds(tgt, state, scene),
            },
        }
    }

    /// Update the collider state after displacement. Checks if the transpose axis need be detached
    /// after moving off a ledge, and update the last [`State`] position for future assertions.
    fn update<A: Axis>(state: &mut State, scene: &Scene<Sc, Map>)
    where
        Map: TileMapView<A, El: SolidTile>,
    {
        state.detach_stale::<A::T, Sc, Map, TileRange<A>, _, _>(
            scene,
            // Check if still attached on the transpose axis only if displacement on the main
            // axis caused the back of the collider to leave a tile.
            |state, _t_end| match state.index_changed::<A, Sc>(!state.inbound_endpoint::<A>()) {
                true => DetachOp::CheckNextTiles,
                false => DetachOp::Break,
            },
            |_, _t_end, view| !view.collided(),
        );
        state.update();
    }

    /// Assert the presence of solid tiles and attach or continue displacement accordingly.
    fn to_next_tile<A: Axis>(
        tgt: Vertex<A>,
        state: &mut State,
        tiles: TileRange<A::T>,
    ) -> ControlFlow<Break<Collision<A::T>>> {
        match tiles.collided() {
            true => {
                state.attach::<A>(tgt.endpoint());
                Collision::TileBound(tiles).into()
            }
            false => {
                state.set_vertex(tgt);
                ControlFlow::Continue(())
            }
        }
    }

    /// Move the the target [`Vertex`] in the already intersected tiles and continue.
    fn to_intersected_tile<A: Axis>(
        tgt: Vertex<A>,
        state: &mut State,
    ) -> ControlFlow<Break<Collision<A::T>>> {
        state.set_vertex(tgt);
        ControlFlow::Continue(())
    }

    /// Assert the presence of a solid map bound and attach or continue displacement accordingly.
    fn to_out_of_bounds<A: Axis>(
        tgt: Vertex<A>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Break<Collision<A::T>>> {
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
}

#[cfg(all(test, feature = "arraymap"))]
mod tests {

    use super::*;
    use bitvec::prelude::*;

    type Sc = ConSc<16>;

    #[rustfmt::skip]
    const MAP: ArrayMap<3, 3, { 3 * 3 }, u8> = ArrayMap::new_unchecked([
        1, 1, 1,
        0, 0, 0,
        1, 0, 1,
    ]);

    mod handle {

        use super::*;

        #[test]
        fn map_bound() {
            let mut state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let scene = Scene::<Sc, _>::new(ArrayMap::<2, 2, 4, _>([false; 4]), AxisMask::ALL);

            let tgt = Vertex::<AxisX>::from_pos::<Sc>(-1.0, Endpoint::LOWER);
            let r = Context::handle_vertex::<AxisX>(tgt, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (0.0, 0.0).into());
            assert_eq!(r, Collision::MapBound.into());
        }

        #[test]
        fn no_map_bound() {
            let mut state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let scene = Scene::<Sc, _>::new(ArrayMap::<2, 2, 4, _>([false; 4]), AxisMask::NONE);

            let tgt = Vertex::<AxisX>::from_pos::<Sc>(-1.0, Endpoint::LOWER);
            let r = Context::handle_vertex::<AxisX>(tgt, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (-1.0, 0.0).into());
            assert_eq!(r, ControlFlow::Continue(()));
        }

        #[test]
        fn intersect() {
            let mut state = State::new::<Sc>(((0.0, 0.0), (15.0, 16.0), AxisMask::NONE));
            let scene = Scene::<Sc, _>::new(ArrayMap::<2, 2, 4, _>([false; 4]), AxisMask::ALL);

            let tgt = Vertex::<AxisX>::from_pos::<Sc>(16.0, Endpoint::UPPER);
            let r = Context::handle_vertex::<AxisX>(tgt, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (1.0, 0.0).into());
            assert_eq!(r, ControlFlow::Continue(()));
        }

        #[test]
        fn next_tile() {
            let mut state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let scene = Scene::<Sc, _>::new(ArrayMap::<2, 2, 4, _>([false; 4]), AxisMask::ALL);

            let tgt = Vertex::<AxisX>::from_pos::<Sc>(17.0, Endpoint::UPPER);
            let r = Context::handle_vertex::<AxisX>(tgt, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (1.0, 0.0).into());
            assert_eq!(r, ControlFlow::Continue(()));
        }

        #[test]
        fn tile_collision() {
            #[rustfmt::skip]
            let map = ArrayMap::<2, 2, 4, _>([
                false, true,
                false, true
            ]);
            let mut state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let scene = Scene::<Sc, _>::new(map, AxisMask::ALL);

            let tgt = Vertex::<AxisX>::from_pos::<Sc>(17.0, Endpoint::UPPER);
            let r = Context::handle_vertex::<AxisX>(tgt, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (0.0, 0.0).into());
            assert_eq!(
                r,
                Collision::TileBound(TileRange::<AxisY>::from_array((1, 0), bitvec![1]).into())
                    .into()
            );
        }
    }

    mod update {

        use super::*;

        #[test]
        fn detach_transpose_on_leaving_edge() {
            let mut state = State::new::<Sc>((
                (16.0, 16.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::BOTTOM.into(),
            ))
            .with_last_pos::<Sc>((0.0, 16.0));
            let scene = Scene::new(&MAP, AxisMask::NONE);
            Context::<Sc, _>::update::<AxisX>(&mut state, &scene);

            assert_eq!(state.attmask(), AxisMask::NONE.into());
        }

        #[test]
        fn dont_detach_transpose_on_same_edge() {
            let mut state = State::new::<Sc>((
                (15.0, 16.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::BOTTOM.into(),
            ))
            .with_last_pos::<Sc>((0.0, 16.0));
            let scene = Scene::new(&MAP, AxisMask::NONE);
            Context::<Sc, _>::update::<AxisX>(&mut state, &scene);

            assert_eq!(state.attmask(), AxisMask::BOTTOM.into());
        }

        #[test]
        fn dont_detach_transpose_on_next_edge() {
            let mut state =
                State::new::<Sc>(((16.0, 16.0), (Sc::SCALE, Sc::SCALE), AxisMask::TOP.into()))
                    .with_last_pos::<Sc>((0.0, 16.0));

            let scene = Scene::new(&MAP, AxisMask::NONE);
            Context::<Sc, _>::update::<AxisX>(&mut state, &scene);

            assert_eq!(state.attmask(), AxisMask::TOP.into());
        }
    }
}
