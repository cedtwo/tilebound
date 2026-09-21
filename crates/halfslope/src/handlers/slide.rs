use std::ops::{ControlFlow, Range};

use tilebound::ctx::scene::Scene;
use tilebound::plane::axis::Axis;
use tilebound::plane::endpoint::{Endpoint, EndpointRange};
use tilebound::plane::scale::Scale;
use tilebound::topology::vertex::Vertex;
use tilebound::view::index::SliceIndex;
use tilebound::view::inspect::TileMapInspect;
use tilebound::view::tilemap::TileMapView;

use crate::collision::{Collision, EdgeRange};
use crate::handlers::VertexHandler;
use crate::prelude::State;
use crate::state::StateExt;
use crate::tile::VertexTile;

/// # Slide
///
/// [`VertexHandler`] marker type for sliding. Continues displacement, projecting displacement to
/// both axes when intersecting a triangle hypotenuse.
pub enum Slide {}

impl<A, Sc, Map> VertexHandler<A, Sc, Map> for Slide
where
    A: Axis,
    Sc: Scale,
    Map: TileMapView<A> + TileMapView<A::T>,
    Map::El: VertexTile + Copy,
{
    fn to_next_tile(
        tgt: Vertex<A>,
        coll: EdgeRange<A::T>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>> {
        debug_assert!(state.lt_vertex_index::<A, Sc>(tgt));

        let adj_tris = coll.trimask().isolate_axis::<A>();
        let t_edge = state.edge::<A::T>();

        if adj_tris.count_ones() == 2 {
            // Attach to wedge vertices.
            if t_edge.is_on_bound::<Sc>(Endpoint::LOWER) && t_edge.is_not_padded::<Sc>() {
                state.attach::<A>(tgt.endpoint());
                Collision::TileBound(coll).into()
            } else {
                let t_wedge_len = state.index_range::<A::T, Sc>();
                state.set_tris::<A>(tgt.endpoint(), coll.trimask());
                Self::handle_wedge(tgt, t_wedge_len, state, scene)
                // VertexHandler::<A, Slide>::handle_wedge(self, a, state, tgt, t_wedge_len)
            }
        } else if let Some(t_sign) = adj_tris.first_end::<A::T>() {
            state.set_tris::<A>(tgt.endpoint(), coll.trimask());
            Self::handle_tri(tgt, t_sign, state, scene)
            // VertexHandler::<A, Slide>::handle_tri(self, a, state, tgt, t_sign)
        } else {
            state.set_vertex::<A>(tgt);
            ControlFlow::Continue(())
        }
    }

    fn handle_tri(
        tgt: Vertex<A>,
        t_tri_end: Endpoint<A::T>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>> {
        let t_vert = state.vertex::<A::T, Sc>(t_tri_end);
        let hyp = t_vert.transpose::<Sc>(tgt.index(), tgt.endpoint());

        if tgt.endpoint().partial_gt(tgt, &hyp) {
            // Displacement exceeds the triangle hypotenuse.
            let mut t_tgt = tgt
                .transpose::<Sc>(state.index::<A::T, Sc>(t_tri_end), t_tri_end)
                .into_adjacent::<Sc>(state.len::<A::T>());

            if t_vert.into_adjacent::<Sc>(state.len::<A::T>()).index() == t_tgt.index() {
                // Transpose axis displacement does not enter a new tile.
                state.set_vertex(tgt);
                state.set_vertex(t_tgt);
                ControlFlow::Continue(())
            } else {
                // Transpose axis displacement enters a new tile.
                let range = Range::new_mapped_incl(
                    tgt.index(),
                    state.index::<A, Sc>(!tgt.endpoint()),
                    tgt.endpoint(),
                );

                match SliceIndex::<A>::try_new(t_tgt.index(), range, t_tri_end, scene.map.size()) {
                    Ok(t_index) => {
                        let t_edges = EdgeRange::inspect(&scene.map, &t_index);
                        match t_edges.inner_collision() {
                            false => {
                                state.set_tris::<A::T>(t_tgt.endpoint(), t_edges.trimask());
                                let axis_tris = state.isolate_edge::<A>(tgt.endpoint());
                                // .isolate(a, tgt.endpoint());

                                if axis_tris.all_on_axis::<A::T>() {
                                    let mut t_range = state.index_range::<A::T, Sc>();
                                    // Extend the range into the new tile.
                                    t_range.incr(!t_tri_end);

                                    Self::handle_wedge(tgt, t_range, state, scene)
                                } else {
                                    state.set_vertex(tgt);
                                    state.set_vertex(t_tgt);
                                    ControlFlow::Continue(())
                                }
                            }
                            true => {
                                t_tgt.decr_bound::<Sc>();
                                state.set_vertex(t_tgt);

                                let lmt = state
                                    .vertex::<A::T, Sc>(!t_tgt.endpoint())
                                    .transpose::<Sc>(tgt.index(), tgt.endpoint());

                                // Check if we still enter the main axis tile.
                                match tgt.index() == lmt.index() {
                                    // Displacement within the tile.
                                    true => {
                                        debug_assert!(!state.is_not_padded::<A::T, Sc>());

                                        state.attach::<A>(tgt.endpoint());
                                        state.attach::<A::T>(t_tgt.endpoint());
                                        state.set_vertex(lmt);
                                    }
                                    // Transpose axis displacement prevents main axis displacement.
                                    false => {
                                        debug_assert!(state.is_not_padded::<A::T, Sc>());
                                        debug_assert_eq!(
                                            lmt.index(),
                                            tgt.endpoint().decr(tgt.index())
                                        );

                                        state.attach::<A>(tgt.endpoint());
                                        state.clear_tris::<A>(tgt.endpoint());
                                        state.set_vertex(lmt);

                                        // Check if colliding with transpose vertices outside of the
                                        // triangle.
                                        if t_edges.truncated_inner_collision(tgt.endpoint(), 1) {
                                            state.attach::<A::T>(t_tgt.endpoint());
                                        }
                                    }
                                }
                                Collision::TriangleApex.into()
                            }
                        }
                    }
                    Err(_) => match scene.map_bounds.end_is_set::<A::T>(t_tri_end) {
                        true => {
                            t_tgt.decr_bound::<Sc>();
                            state.set_vertex(t_tgt);

                            let lmt = state
                                .vertex::<A::T, Sc>(!t_tgt.endpoint())
                                .transpose::<Sc>(tgt.index(), tgt.endpoint());

                            match tgt.index() == lmt.index() {
                                // Displacement within the tile.
                                true => {
                                    debug_assert!(!state.is_not_padded::<A::T, Sc>());

                                    state.attach::<A>(tgt.endpoint());
                                    state.attach::<A::T>(t_tgt.endpoint());
                                    state.set_vertex(lmt);
                                }
                                // Transpose axis displacement prevents main axis displacement.
                                false => {
                                    debug_assert!(state.is_not_padded::<A::T, Sc>());
                                    debug_assert_eq!(lmt.index(), tgt.endpoint().decr(tgt.index()));

                                    state.attach::<A>(tgt.endpoint());
                                    state.attach::<A::T>(t_tgt.endpoint());
                                    state.set_vertex(lmt);
                                    // Revert changes to the `TriMask`.
                                    state.clear_tris::<A>(tgt.endpoint());
                                }
                            }
                            Collision::TriangleApex.into()
                        }
                        false => {
                            state.set_vertex(tgt);
                            state.set_vertex(t_tgt);
                            ControlFlow::Continue(())
                        }
                    },
                }
            }
        } else {
            // Displacement does not exceed the triangle hypotenuse.
            state.set_vertex(tgt);
            ControlFlow::Continue(())
        }
    }

    fn handle_wedge(
        tgt: Vertex<A>,
        t_range: Range<i32>,
        state: &mut State,
        _scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>> {
        let wedge_len = t_range.len() as u32;
        let wedge_bleed = wedge_len - 2;

        let t_edge_reduced = state.len::<A::T>() - (wedge_bleed as i32 * Sc::SCALE_INT) as f32;

        let mut lmt = tgt.clone();
        lmt.set_endpoint_offset::<Sc>(t_edge_reduced / 2.0, tgt.endpoint().invert());

        // Vertex exceeds the wedge limit.
        if tgt.endpoint().partial_gt(tgt, &lmt) {
            let t_lmt = lmt.transpose::<Sc>(t_range.start, Endpoint::LOWER);

            state.attach::<A>(tgt.endpoint());
            state.set_vertex(lmt);
            state.set_vertex(t_lmt);

            Collision::WedgeApex.into()

            // Vertex does not exceeds the wedge limit.
        } else {
            let mut t_edge = state.edge::<A::T>();
            let t_edge_range = t_edge.index_range::<Sc>();

            // Get the nearest triangle hypotenuse.
            if let Some(t_end) = t_edge_range.endpoint_diff(&t_range).map(|end| end) {
                // The collider occupies one triangle (prior to displacing).
                // Only reachable when displacing the transpose axis from `handle_tri`.
                t_edge.set_inbound_end::<Sc>(t_end);
                let t_tgt = t_edge.take_vertex();
                let t_lmt = tgt.transpose::<Sc>(t_tgt.index(), t_tgt.endpoint());

                state.set_vertex::<A::T>(t_lmt);
                // Assert that the collider now occupies both extremes of the wedge.
                debug_assert_eq!(state.edge::<A::T>().index_range::<Sc>(), t_range);
            } else {
                // The collider occupies both triangles.
                let t_tgt = t_edge
                    .nearest_bound_vertex::<Sc>()
                    .unwrap_or_else(|| t_edge.inbound_vertex());
                let t_vert = state.vertex::<A::T, Sc>(t_tgt.endpoint());
                let t_vert_lmt = tgt.transpose::<Sc>(t_tgt.index(), t_tgt.endpoint());

                if t_vert.endpoint().partial_gt(t_vert, &t_vert_lmt) {
                    state.set_vertex(t_vert_lmt);
                }
            }

            state.set_vertex(tgt);
            ControlFlow::Continue(())
        }
    }

    #[inline]
    fn update(state: &mut State, _scene: &Scene<Sc, Map>) {
        state.clear_disjoint_tris::<A, Sc>();
        state.clear_disjoint_tris::<A::T, Sc>();
    }
}

#[cfg(all(test, feature = "arraymap"))]
mod test {

    use std::ops::ControlFlow;

    use bitvec::prelude::*;

    use super::*;
    use crate::collision::Collision;
    use crate::collision::EdgeRange;
    use crate::prelude::State;
    use crate::vertex_mask::VertexMask;
    use tilebound::prelude::*;
    use tilebound::topology::vertex::Vertex;

    type Sc = ConSc<16>;

    const EMPTY: tilebound::view::tilemap::ArrayMap<0, 0, 0, u8> =
        tilebound::view::tilemap::ArrayMap::new_unchecked([]);

    mod continue_ {

        use super::*;

        #[test]
        fn on_tri_vertex_nopad() {
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 1.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into()); // Not updated yet.
            assert_eq!(result, ControlFlow::Continue(()));
        }

        #[test]
        fn on_tri_vertex_pad() {
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE / 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 1.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into()); // Not updated yet.
            assert_eq!(result, ControlFlow::Continue(()));
        }

        #[test]
        fn on_tri_hypotenuse() {
            let mut state = State::new::<Sc>((
                (16.0, 1.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (1.0, 15.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into()); // Not updated yet.
            assert_eq!(result, ControlFlow::Continue(()));
        }

        #[test]
        fn between_diagonal_tris() {
            #[rustfmt::skip]
            const MAP: ArrayMap<2, 2, 4, u8> = ArrayMap::new_unchecked([
                07, 00,
                00, 14,
            ]);
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&MAP, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(1.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (1.0, 15.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(
                *state.res(),
                (VertexMask::TOP_LEFT_INCL | VertexMask::BOTTOM_RIGHT_INCL).into()
            );
            assert_eq!(result, ControlFlow::Continue(()));
        }

        #[test]
        fn before_tri_hypotenuse() {
            let mut state = State::new::<Sc>((
                (16.0, 2.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..2, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 2.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into());
            assert_eq!(result, ControlFlow::Continue(()));
        }

        #[test]
        fn before_wedge_hypotenuse() {
            let mut state = State::new::<Sc>((
                (16.0, 2.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..2, Endpoint::UPPER);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 2.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::LEFT.into());
            assert_eq!(result, ControlFlow::Continue(()));
        }

        #[test]
        fn on_sliding_out_of_bounds() {
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE * 3.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..3, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (0.0, 16.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into());
            assert_eq!(result, ControlFlow::Continue(()));
        }

        #[test]
        fn on_wedge_neg_hypotenuse() {
            let mut state = State::new::<Sc>((
                (16.0, 2.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..2, Endpoint::UPPER);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(10.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (10.0, 6.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::LEFT.into());
            assert_eq!(result, ControlFlow::Continue(()));
        }

        #[test]
        fn on_wedge_pos_hypotenuse() {
            let mut state = State::new::<Sc>((
                (16.0, 14.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(10.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (10.0, 10.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::LEFT.into());
            assert_eq!(result, ControlFlow::Continue(()));
        }
    }

    mod break_ {

        use super::*;

        #[test]
        fn on_triangle_apex_in_front() {
            #[rustfmt::skip]
            const MAP: ArrayMap<3, 3, 9, u8> = ArrayMap::new_unchecked([
                07, 00, 00,
                00, 00, 00,
                15, 00, 00,
            ]);
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, 31.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&MAP, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 1.0).into());
            assert_eq!(state.attmask(), AxisMask::BOTTOM_LEFT.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into());
            assert_eq!(result, Collision::TriangleApex.into());
        }

        #[test]
        fn on_triangle_apex_behind() {
            #[rustfmt::skip]
            const MAP: ArrayMap<3, 3, 9, u8> = ArrayMap::new_unchecked([
                07, 00, 00,
                00, 00, 00,
                00, 15, 00,
            ]);
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, 31.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&MAP, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 1.0).into());
            assert_eq!(state.attmask(), AxisMask::BOTTOM_LEFT.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into());
            assert_eq!(result, Collision::TriangleApex.into());
        }

        #[test]
        fn on_triangle_apex_eq_collider_size() {
            #[rustfmt::skip]
            const MAP: ArrayMap<3, 3, 9, u8> = ArrayMap::new_unchecked([
                07, 00, 00,
                00, 00, 00,
                15, 00, 00,
            ]);
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&MAP, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..2, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (16.0, 0.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::NONE.into());
            assert_eq!(result, Collision::TriangleApex.into());
        }

        #[test]
        fn on_map_bound_triangle_apex() {
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, 47.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::ALL);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..3, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 1.0).into());
            assert_eq!(state.attmask(), AxisMask::BOTTOM_LEFT.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into());
            assert_eq!(result, Collision::TriangleApex.into());
        }

        #[test]
        fn on_map_bound_triangle_apex_eq_collider_size() {
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE * 3.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::ALL);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..3, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (16.0, 0.0).into());
            assert_eq!(state.attmask(), AxisMask::BOTTOM_LEFT.into());
            assert_eq!(*state.res(), VertexMask::NONE.into());
            assert_eq!(result, Collision::TriangleApex.into());
        }

        #[test]
        fn on_wedge_eq_collider_size() {
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE * 3.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (16.0, 0.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::NONE.into());
            assert_eq!(
                result,
                Collision::TileBound(EdgeRange::from_array(
                    (0, 0),
                    Endpoint::UPPER,
                    bitvec![1, 0, 0, 0, 0, 1]
                ))
                .into()
            );
        }

        #[test]
        fn on_wedge_apex() {
            let mut state = State::new::<Sc>((
                (16.0, 2.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (8.0, 8.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::LEFT.into());
            assert_eq!(result, Collision::WedgeApex.into());
        }

        #[test]
        fn on_sliding_into_wedge_apex() {
            #[rustfmt::skip]
            const MAP: ArrayMap<3, 3, 9, u8> = ArrayMap::new_unchecked([
                07, 00, 00,
                00, 00, 00,
                11, 00, 00,
            ]);
            let mut state = State::new::<Sc>((
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            ));
            let scene = Scene::<Sc, _>::new(&MAP, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..2, Endpoint::TOP, Endpoint::RIGHT);
            let vert_tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let result = Slide::to_next_tile(vert_tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (8.0, 8.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::LEFT.into());
            assert_eq!(result, Collision::WedgeApex.into());
        }
    }
}
