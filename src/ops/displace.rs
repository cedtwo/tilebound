//! Displacement helper functions.
use std::ops::ControlFlow;

use crate::ctx::brk::Break;
use crate::ctx::scene::Scene;
use crate::ctx::state::State;
use crate::plane::axis::Axis;
use crate::plane::endpoint::EndpointRange;
use crate::plane::scale::Scale;
use crate::topology::delta::Delta;
use crate::topology::vertex::Vertex;
use crate::view::tilemap::TileMap;

/// Handle out-of-bounds displacement toward, or parallel to the map. Expects a [`Delta::target`]
/// [`Vertex`].
///
/// Displaces immediately to the `tgt` or the furthermost point prior to entering the tilemap. This
/// can be called once at the start of displacement to avoid looping over a [`Delta`] while out of
/// bounds. Returns `true` if displaced and does nothing if in-bounds.
pub fn skip_to_map<A: Axis, R, Sc: Scale, Map: TileMap>(
    tgt: Vertex<A>,
    state: &mut State<R>,
    scene: &Scene<Sc, Map>,
) -> bool {
    let end = tgt.endpoint();
    let vert_idx = state.index::<A, Sc>(end);
    let bnd_idx = scene.map.bound_index::<A::T>(!end) as i32;

    if !state
        .index_range::<A::T, Sc>()
        .intersects(&(0..scene.map.axis_len::<A>() as i32))
    {
        // Out-of-bounds displacement **parallell** to the map.
        state.set_vertex::<A>(tgt);
        true
    } else if end.gt(bnd_idx, vert_idx) {
        // Out-of-bounds displacing **toward** the map.
        let lmt = Vertex::from_bound::<Sc>(end.decr(bnd_idx), end);
        state.set_vertex::<A>(end.partial_min(lmt, tgt));
        true
    } else {
        false
    }
}

/// Handle out-of-bounds displacement away from the map. Expects a [`Delta::target`] [`Vertex`].
///
/// Displaces immediately to the `tgt` if moving away from the tilemap. This can be called once at
/// the start of displacement to avoid looping over a [`Delta`] while out of bounds. Returns `true`
/// if displaced and does nothing if in-bounds.
///
/// Note that unlike [`skip_to_map`], the back of the collider may still be within the map bounds
/// and may require additional operations on displacement (eg. detaching the transpose axis or
/// updating stale state).
pub fn skip_from_map<A: Axis, R, Sc: Scale, Map: TileMap>(
    tgt: Vertex<A>,
    state: &mut State<R>,
    scene: &Scene<Sc, Map>,
) -> bool {
    let end = tgt.endpoint();
    let vert_idx = state.index::<A, Sc>(end);
    let bnd_idx = scene.map.bound_index::<A::T>(end) as i32;

    if end.gt(vert_idx, bnd_idx) {
        // Out-of-bounds displacing **away** from the map.
        state.set_vertex::<A>(tgt);
        true
    } else {
        false
    }
}

/// Loops over the delta passing vertices to the `handler` callback, calling the `update` callback
/// on any (potential) displacement. Out-of-bound displacement skips straight to the target vertex,
/// also calling `update`.
///
/// This function passing the next furthest vertex of the current [`State`] to `handler` on each
/// iteration. Iteration only ends when `handler` returns [`ControlFlow::Break`], or the `State` is
/// set to [`Delta::target`] (the furthest vertex returned from the iterator). See [`Delta`],
/// [`skip_from_map`] and [`skip_to_map`].
pub fn loop_outer_delta<A: Axis, R, Sc: Scale, Map: TileMap, F0, F1, Brk>(
    delta: Delta<A>,
    state: &mut State<R>,
    scene: &Scene<Sc, Map>,
    mut handler: F0,
    mut update: F1,
) -> Break<Brk>
where
    F0: FnMut(Vertex<A>, &mut State<R>, &Scene<Sc, Map>) -> ControlFlow<Break<Brk>>,
    F1: FnMut(&mut State<R>, &Scene<Sc, Map>),
{
    if skip_to_map(delta.target(), state, scene) {
        update(state, scene);
    }
    if skip_from_map(delta.target(), state, scene) {
        update(state, scene);
        return Break::ReachedTarget;
    }

    while let Some(tgt) = delta.next_inbound_or_target::<Sc>(state.vertex::<A, Sc>(delta.end())) {
        let cf = handler(tgt, state, scene);
        update(state, scene);

        if let ControlFlow::Break(brk) = cf {
            return brk;
        }

        if skip_from_map(delta.target(), state, scene) {
            update(state, scene);
            return Break::ReachedTarget;
        }
    }

    Break::ReachedTarget
}

#[cfg(all(test, feature = "arraymap"))]
mod tests {

    use super::*;
    use crate::plane::axis::{AxisMask, AxisX};
    use crate::plane::endpoint::Endpoint;
    use crate::plane::scale::ConSc;
    use crate::view::tilemap::ArrayMap;

    type Sc = ConSc<16>;

    mod skip_to_map {

        use super::*;

        #[test]
        fn inb_to_inb() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));

            let mut tgt = state.vertex::<AxisX, Sc>(Endpoint::Upper);
            tgt.translate::<Sc>(16.0);
            let mutated = skip_to_map::<AxisX, _, Sc, _>(tgt, &mut state, &scene);

            assert!(!mutated);
            assert_eq!(state.pos::<AxisX, Sc>(), 0.0);
        }

        #[test]
        fn inb_to_outb() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let mut tgt_l = state.vertex::<AxisX, Sc>(Endpoint::Lower);
            tgt_l.translate::<Sc>(-32.0);
            let mut tgt_r = state.vertex::<AxisX, Sc>(Endpoint::Upper);
            tgt_r.translate::<Sc>(64.0);

            let mutated = skip_to_map::<AxisX, _, Sc, _>(tgt_l, &mut state, &scene);
            assert!(!mutated);
            assert_eq!(state.pos::<AxisX, Sc>(), 0.0);
            let mutated = skip_to_map::<AxisX, _, Sc, _>(tgt_r, &mut state, &scene);
            assert!(!mutated);
            assert_eq!(state.pos::<AxisX, Sc>(), 0.0);
        }

        #[test]
        fn outb_to_inb() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state_l = State::new::<Sc>(((-32.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let mut state_r = State::new::<Sc>(((48.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let tgt_l = Vertex::from_pos::<Sc>(0.0, Endpoint::Lower);
            let tgt_r = Vertex::from_pos::<Sc>(0.0, Endpoint::Upper);

            let mutated = skip_to_map::<AxisX, _, Sc, _>(tgt_r, &mut state_l, &scene);
            assert!(mutated);
            assert_eq!(state_l.pos::<AxisX, Sc>(), -16.0);
            let mutated = skip_to_map::<AxisX, _, Sc, _>(tgt_l, &mut state_r, &scene);
            assert!(mutated);
            assert_eq!(state_r.pos::<AxisX, Sc>(), 32.0);
        }

        #[test]
        fn outb_to_outb() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state_l = State::new::<Sc>(((-32.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let mut state_r = State::new::<Sc>(((48.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let tgt_l = Vertex::from_pos::<Sc>(-64.0, Endpoint::Lower);
            let tgt_r = Vertex::from_pos::<Sc>(96.0, Endpoint::Upper);

            let mutated = skip_to_map::<AxisX, _, Sc, _>(tgt_l, &mut state_l, &scene);
            assert!(!mutated);
            assert_eq!(state_l.pos::<AxisX, Sc>(), -32.0);
            let mutated = skip_to_map::<AxisX, _, Sc, _>(tgt_r, &mut state_r, &scene);
            assert!(!mutated);
            assert_eq!(state_r.pos::<AxisX, Sc>(), 48.0);
        }

        #[test]
        fn parallel_to_map() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state = State::new::<Sc>(((-16.0, -16.0), (16.0, 16.0), AxisMask::NONE));
            let tgt = Vertex::from_pos::<Sc>(48.0, Endpoint::Upper);

            let mutated = skip_to_map::<AxisX, _, Sc, _>(tgt, &mut state, &scene);
            assert!(mutated);
            assert_eq!(state.pos::<AxisX, Sc>(), 32.0);
        }
    }

    mod skip_from_map {

        use super::*;

        #[test]
        fn inb_to_inb() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let tgt = Vertex::from_pos::<Sc>(32.0, Endpoint::Upper);

            let mutated = skip_from_map::<AxisX, _, Sc, _>(tgt, &mut state, &scene);
            assert!(!mutated);
            assert_eq!(state.pos::<AxisX, Sc>(), 0.0);
        }

        #[test]
        fn inb_to_outb() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state = State::new::<Sc>(((0.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let tgt_l = Vertex::from_pos::<Sc>(-32.0, Endpoint::Lower);
            let tgt_r = Vertex::from_pos::<Sc>(80.0, Endpoint::Upper);

            let mutated = skip_from_map::<AxisX, _, Sc, _>(tgt_l, &mut state, &scene);
            assert!(!mutated);
            assert_eq!(state.pos::<AxisX, Sc>(), 0.0);
            let mutated = skip_from_map::<AxisX, _, Sc, _>(tgt_r, &mut state, &scene);
            assert!(!mutated);
            assert_eq!(state.pos::<AxisX, Sc>(), 0.0);
        }

        #[test]
        fn outb_to_inb() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state_l = State::new::<Sc>(((-32.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let mut state_r = State::new::<Sc>(((48.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let tgt_l = Vertex::from_pos::<Sc>(0.0, Endpoint::Lower);
            let tgt_r = Vertex::from_pos::<Sc>(0.0, Endpoint::Upper);

            let mutated = skip_from_map::<AxisX, _, Sc, _>(tgt_r, &mut state_l, &scene);
            assert!(!mutated);
            assert_eq!(state_l.pos::<AxisX, Sc>(), -32.0);
            let mutated = skip_from_map::<AxisX, _, Sc, _>(tgt_l, &mut state_r, &scene);
            assert!(!mutated);
            assert_eq!(state_r.pos::<AxisX, Sc>(), 48.0);
        }

        #[test]
        fn outb_to_outb() {
            let scene = Scene::<Sc, _>::new(ArrayMap::<3, 2, 6, _>([false; 6]), AxisMask::ALL);
            let mut state_l = State::new::<Sc>(((-32.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let mut state_r = State::new::<Sc>(((48.0, 0.0), (16.0, 16.0), AxisMask::NONE));
            let tgt_l = Vertex::from_pos::<Sc>(-64.0, Endpoint::Lower);
            let tgt_r = Vertex::from_pos::<Sc>(96.0, Endpoint::Upper);

            let mutated = skip_from_map::<AxisX, _, Sc, _>(tgt_l, &mut state_l, &scene);
            assert!(mutated);
            assert_eq!(state_l.pos::<AxisX, Sc>(), -64.0);
            let mutated = skip_from_map::<AxisX, _, Sc, _>(tgt_r, &mut state_r, &scene);
            assert!(mutated);
            assert_eq!(state_r.pos::<AxisX, Sc>(), 80.0);
        }
    }
}
