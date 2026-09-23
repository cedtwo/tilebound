use std::ops::{ControlFlow, Range};

use tilebound::ctx::scene::Scene;
use tilebound::plane::axis::Axis;
use tilebound::plane::endpoint::Endpoint;
use tilebound::plane::scale::Scale;
use tilebound::topology::vertex::Vertex;
use tilebound::view::tilemap::TileMapView;

use crate::collision::{Collision, EdgeRange};
use crate::handlers::VertexHandler;
use crate::prelude::State;
use crate::state::StateExt;
use crate::tile::VertexTile;

/// # Attach
///
/// [`VertexHandler`] marker type for attaching on collision with a triangle hypotenuse. Ends
/// displacement, attaches and sets position onto the triangle hypotenuse on intersection.
pub enum Attach {}

impl<A, Sc, Map> VertexHandler<A, Sc, Map> for Attach
where
    A: Axis,
    Sc: Scale,
    Map: TileMapView<A>,
    Map::El: VertexTile,
{
    fn to_next_tile(
        tgt: Vertex<A>,
        insp: EdgeRange<A::T>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>> {
        debug_assert!(state.lt_vertex_index::<A, Sc>(tgt));

        let adj_tris = insp.trimask().isolate_axis::<A>();
        let t_edge = state.edge::<A::T>();

        if adj_tris.all_on_axis::<A::T>() {
            // Attach to vertex/vertices.
            if t_edge.is_on_bound::<Sc>(Endpoint::LOWER) {
                // Attached to either the negative, or both wedge vertices.
                state.attach::<A>(tgt.endpoint());
                Collision::TileBound(insp).into()
            } else if t_edge.is_on_bound::<Sc>(Endpoint::UPPER) {
                // Attached to either the positive, or both wedge vertices.
                state.attach::<A>(tgt.endpoint());
                Collision::TileBound(insp).into()
            } else {
                let t_wedge_range = state.index_range::<A::T, Sc>();
                state.set_tris::<A>(tgt.endpoint(), insp.trimask());
                Self::handle_wedge(tgt, t_wedge_range, state, scene)
            }
        } else if let Some(t_sign) = adj_tris.first_end::<A::T>() {
            if t_edge.is_on_bound::<Sc>(t_sign) {
                // Attached to a single triangle vertex.
                state.attach::<A>(tgt.endpoint());
                Collision::TileBound(insp).into()
            } else {
                state.set_tris::<A>(tgt.endpoint(), insp.trimask());
                Self::handle_tri(tgt, t_sign, state, scene)
            }
        } else {
            state.set_vertex::<A>(tgt);
            ControlFlow::Continue(())
        }
    }

    fn handle_tri(
        tgt: Vertex<A>,
        t_end: Endpoint<A::T>,
        state: &mut State,
        _scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>> {
        let t_vert = state.vertex::<A::T, Sc>(t_end);
        let hyp = t_vert.transpose::<Sc>(tgt.index(), tgt.endpoint());

        if let Some(lmt) = tgt.endpoint().partial_gt(tgt, &hyp).then_some(hyp) {
            state.set_vertex::<A>(lmt);

            match tgt.endpoint().ordering(tgt.index(), lmt.index()).is_gt() {
                // Collided prior to entering the tile (at the vertex).
                true => {
                    state.attach::<A>(tgt.endpoint());
                    state.clear_tris::<A>(tgt.endpoint());
                }
                // Collided with the triangle at a point within the tile.
                false => {
                    state.attach::<A>(tgt.endpoint());
                }
            }
            Collision::TriangleHypotenuse.into()
        } else {
            state.set_vertex::<A>(tgt);
            ControlFlow::Continue(())
        }
    }

    fn handle_wedge(
        tgt: Vertex<A>,
        _t_range: Range<i32>,
        state: &mut State,
        _scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>> {
        let t_edge = state.edge::<A::T>();
        // The vertex nearest the hypotenuse on the transpose axis.
        let t_near = t_edge
            .nearest_bound_vertex::<Sc>()
            .unwrap_or_else(|| t_edge.inbound_vertex());
        let lmt = t_near.transpose::<Sc>(tgt.index(), tgt.endpoint());

        match lmt.index() == tgt.index() {
            // Displacement within the tile.
            true => {
                if tgt.endpoint().partial_gt(tgt, &lmt) {
                    state.attach::<A>(tgt.endpoint());
                    state.set_vertex::<A>(lmt);

                    Collision::TriangleHypotenuse.into()
                } else {
                    // Displacement on the main axis does not exceed a triangle.
                    state.set_vertex::<A>(tgt);
                    ControlFlow::Continue(())
                }
            }
            // Collided with a tri vertex.
            false => {
                state.attach::<A>(tgt.endpoint());
                state.clear_tris::<A>(lmt.endpoint());
                state.set_vertex::<A>(lmt);

                Collision::TriangleHypotenuse.into()
            }
        }
    }

    fn update(state: &mut State, _scene: &Scene<Sc, Map>) {
        state.clear_disjoint_tris::<A, Sc>();
    }
}

#[cfg(all(test, feature = "arraymap"))]
mod tests {

    use bitvec::prelude::*;

    use super::*;
    use crate::prelude::State;
    use crate::vertex_mask::VertexMask;
    use tilebound::prelude::*;

    type Sc = ConSc<16>;

    const EMPTY: tilebound::view::tilemap::ArrayMap<0, 0, 0, u8> =
        tilebound::view::tilemap::ArrayMap::new_unchecked([]);

    mod continue_ {

        use super::*;

        #[test]
        fn before_tri_hypotenuse() {
            let payload = BoundBox::new_with_res(
                (16.0, 2.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 2.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into());
            assert_eq!(r, ControlFlow::Continue(()));
        }

        #[test]
        fn before_wedge_hypotenuse() {
            let payload = BoundBox::new_with_res(
                (16.0, 2.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(15.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (15.0, 2.0).into());
            assert_eq!(state.attmask(), AxisMask::NONE.into());
            assert_eq!(*state.res(), VertexMask::LEFT.into());
            assert_eq!(r, ControlFlow::Continue(()));
        }
    }

    mod break_ {

        use super::*;

        #[test]
        fn on_tri_vertex() {
            let payload = BoundBox::new_with_res(
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (16.0, 0.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::NONE.into());
            assert_eq!(
                r,
                Collision::TileBound(EdgeRange::from_array(
                    (0, 0),
                    Endpoint::UPPER,
                    bitvec![1, 0]
                ))
                .into()
            )
        }

        #[test]
        fn on_tri_hypotenuse() {
            let payload = BoundBox::new_with_res(
                (16.0, 2.0),
                (Sc::SCALE, Sc::SCALE),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extreme(0, 0..1, Endpoint::TOP, Endpoint::RIGHT);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (14.0, 2.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::TOP_LEFT_INCL.into());
            assert_eq!(r, Collision::TriangleHypotenuse.into())
        }

        #[test]
        fn on_lower_wedge_vertex() {
            let payload = BoundBox::new_with_res(
                (16.0, 0.0),
                (Sc::SCALE, 47.0),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (16.0, 0.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::NONE.into());
            assert_eq!(
                r,
                Collision::TileBound(EdgeRange::from_array(
                    (0, 0),
                    Endpoint::UPPER,
                    bitvec![1, 0, 0, 0, 0, 1]
                ),)
                .into()
            )
        }

        #[test]
        fn on_upper_wedge_vertex() {
            let payload = BoundBox::new_with_res(
                (16.0, 1.0),
                (Sc::SCALE, 47.0),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (16.0, 1.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::NONE.into());
            assert_eq!(
                r,
                Collision::TileBound(EdgeRange::from_array(
                    (0, 0),
                    Endpoint::UPPER,
                    bitvec![1, 0, 0, 0, 0, 1]
                ))
                .into()
            )
        }

        #[test]
        fn on_wedge_vertices() {
            let payload = BoundBox::new_with_res(
                (16.0, 0.0),
                (Sc::SCALE, Sc::SCALE * 3.0),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (16.0, 0.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::NONE.into());
            assert_eq!(
                r,
                Collision::TileBound(EdgeRange::from_array(
                    (0, 0),
                    Endpoint::UPPER,
                    bitvec![1, 0, 0, 0, 0, 1]
                ))
                .into()
            );
        }

        #[test]
        fn on_lower_wedge_hypotenuse() {
            let payload = BoundBox::new_with_res(
                (16.0, 4.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (12.0, 4.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::LEFT.into());
            assert_eq!(r, Collision::TriangleHypotenuse.into());
        }

        #[test]
        fn on_upper_wedge_hypotenuse() {
            let payload = BoundBox::new_with_res(
                (16.0, 12.0),
                (Sc::SCALE, Sc::SCALE * 2.0),
                AxisMask::NONE,
                VertexMask::NONE,
            );
            let mut state = State::new::<Sc, _>(&payload);
            let scene = Scene::<Sc, _>::new(&EMPTY, AxisMask::NONE);

            let edges = EdgeRange::<AxisY>::extremes(0, 0..3, Endpoint::UPPER);
            let tgt = Vertex::<AxisX>::from_pos::<Sc>(0.0, Endpoint::LOWER);
            let r = Attach::to_next_tile(tgt, edges, &mut state, &scene);

            assert_eq!(state.pos_vec::<Sc>(), (12.0, 12.0).into());
            assert_eq!(state.attmask(), AxisMask::LEFT.into());
            assert_eq!(*state.res(), VertexMask::LEFT.into());
            assert_eq!(r, Collision::TriangleHypotenuse.into());
        }
    }
}
