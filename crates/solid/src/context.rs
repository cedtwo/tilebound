use std::ops::ControlFlow;

use crate::collision::{Collision, TileRange};
use crate::tile::SolidTile;
use tilebound::ctx::brk::Break;
use tilebound::ctx::state::DetachOp;
use tilebound::ops::*;
use tilebound::prelude::*;
use tilebound::topology::delta::Delta;
use tilebound::topology::vertex::Vertex;

/// # Context
///
/// `Context` is a [`Scene`] wrapper exposing methods for displacement (namely [`Context::sweep_by`]
/// and [`Context::sweep_to`]). Displacement requires a mutable reference to a rectangle [`State`]
/// that will be mutated with the result of displacement. See crate level documentation for
/// examples.
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
    /// Attempt to displace the given [`BoundBox`] by the given `delta`, checking all
    /// intersecting tiles of the delta. Returns a [`TileRange`] on collision with a tile.
    pub fn sweep_by<'a, A: Axis, P: BoundBoxView<R = ()>>(
        &self,
        bbox: &mut BoundBox<'a, P>,
        delta: f32,
    ) -> Break<Collision<A::T>>
    where
        Map: TileMapView<A, El: SolidTile>,
        Map: TileMapView<A::T, El: SolidTile>,
    {
        let mut state = State::new::<Sc, _>(bbox);
        let Some(delta) = delta_by::<A, Sc, _>(&state, delta) else {
            return Break::None;
        };
        state.detach::<A>();
        Self::loop_delta(bbox, delta, &mut state, &self.0)
    }

    /// Attempt to displace the nearest [`BoundBox`] edge to the given position, checking all
    /// intersected tiles along the delta. Returns a [`TileRange`] on collision with a tile.
    pub fn sweep_to<'a, A: Axis, P: BoundBoxView<R = ()>>(
        &self,
        bbox: &mut BoundBox<'a, P>,
        pos: f32,
    ) -> Break<Collision<A::T>>
    where
        Map: TileMapView<A, El: SolidTile>,
        Map: TileMapView<A::T, El: SolidTile>,
    {
        let mut state = State::new::<Sc, _>(bbox);
        let Some(delta) = delta_to::<A, Sc, _>(&state, pos) else {
            return Break::None;
        };
        state.detach::<A>();
        Self::loop_delta(bbox, delta, &mut state, &self.0)
    }

    /// Loops over the delta, commiting changes back to the [`BoundBox`] and returning the result.
    pub fn loop_delta<'a, A: Axis, P: BoundBoxView<R = ()>>(
        bbox: &mut BoundBox<'a, P>,
        delta: Delta<A>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> Break<Collision<A::T>>
    where
        Map: TileMapView<A, El: SolidTile>,
        Map: TileMapView<A::T, El: SolidTile>,
    {
        let brk = loop_outer_delta(delta, state, scene, Self::handle_vertex, Self::update::<A>);
        state.apply::<Sc, _>(bbox);
        brk
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
            let payload = BoundBox::new((0.0, 0.0), (16.0, 16.0), AxisMask::NONE);
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(ArrayMap::<2, 2, 4, _>([false; 4]), AxisMask::ALL);

            let tgt = Vertex::<AxisX>::from_pos::<Sc>(-1.0, Endpoint::LOWER);
            let r = Context::handle_vertex::<AxisX>(tgt, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (0.0, 0.0).into());
            assert_eq!(r, Collision::MapBound.into());
        }

        #[test]
        fn no_map_bound() {
            let payload = BoundBox::new((0.0, 0.0), (16.0, 16.0), AxisMask::NONE);
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(ArrayMap::<2, 2, 4, _>([false; 4]), AxisMask::NONE);

            let tgt = Vertex::<AxisX>::from_pos::<Sc>(-1.0, Endpoint::LOWER);
            let r = Context::handle_vertex::<AxisX>(tgt, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (-1.0, 0.0).into());
            assert_eq!(r, ControlFlow::Continue(()));
        }

        #[test]
        fn intersect() {
            let payload = BoundBox::new((0.0, 0.0), (15.0, 16.0), AxisMask::NONE);
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(ArrayMap::<2, 2, 4, _>([false; 4]), AxisMask::ALL);

            let tgt = Vertex::<AxisX>::from_pos::<Sc>(16.0, Endpoint::UPPER);
            let r = Context::handle_vertex::<AxisX>(tgt, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (1.0, 0.0).into());
            assert_eq!(r, ControlFlow::Continue(()));
        }

        #[test]
        fn next_tile() {
            let payload = BoundBox::new((0.0, 0.0), (16.0, 16.0), AxisMask::NONE);
            let mut state = State::new::<Sc, _>(&payload);
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
            let payload = BoundBox::new((0.0, 0.0), (16.0, 16.0), AxisMask::NONE);
            let mut state = State::new::<Sc, _>(&payload);
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
            let payload = BoundBox::new((16.0, 16.0), (Sc::SCALE, Sc::SCALE), AxisMask::BOTTOM);
            let mut state = State::new::<Sc, _>(&payload).with_last_pos::<Sc>((0.0, 16.0));
            let scene = Scene::new(&MAP, AxisMask::NONE);
            Context::<Sc, _>::update::<AxisX>(&mut state, &scene);

            assert_eq!(state.attmask(), AxisMask::NONE.into());
        }

        #[test]
        fn dont_detach_transpose_on_same_edge() {
            let payload = BoundBox::new((15.0, 16.0), (Sc::SCALE, Sc::SCALE), AxisMask::BOTTOM);
            let mut state = State::new::<Sc, _>(&payload).with_last_pos::<Sc>((0.0, 16.0));
            let scene = Scene::new(&MAP, AxisMask::NONE);
            Context::<Sc, _>::update::<AxisX>(&mut state, &scene);

            assert_eq!(state.attmask(), AxisMask::BOTTOM.into());
        }

        #[test]
        fn dont_detach_transpose_on_next_edge() {
            let payload = BoundBox::new((16.0, 16.0), (Sc::SCALE, Sc::SCALE), AxisMask::TOP);
            let mut state = State::new::<Sc, _>(&payload).with_last_pos::<Sc>((0.0, 16.0));

            let scene = Scene::new(&MAP, AxisMask::NONE);
            Context::<Sc, _>::update::<AxisX>(&mut state, &scene);

            assert_eq!(state.attmask(), AxisMask::TOP.into());
        }
    }
}
