use std::ops::{ControlFlow, Range};

use tilebound::ctx::scene::Scene;
use tilebound::plane::axis::Axis;
use tilebound::plane::endpoint::Endpoint;
use tilebound::topology::vertex::Vertex;

use crate::collision::{Collision, EdgeRange};
use crate::prelude::State;

/// # VertexHandler
///
/// Logic for handling vertex displacement, especially in relation to the triangle hypotenuse.
pub trait VertexHandler<A: Axis, Sc, Map> {
    fn to_next_tile(
        tgt: Vertex<A>,
        inspect: EdgeRange<A::T>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>>;

    /// Defines how to handle a collider edge spanning tiles with a single triangle at an extreme.
    /// Receives the [`Endpoint`] of the triangle on the transpose axis.
    fn handle_tri(
        tgt: Vertex<A>,
        t_end: Endpoint<A::T>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>>;

    /// Defines how to handle a collider edge spanning tiles with triangles at both extremes.
    /// Receives the wedge `Range` on the transpose axis.
    fn handle_wedge(
        tgt: Vertex<A>,
        t_range: Range<i32>,
        state: &mut State,
        scene: &Scene<Sc, Map>,
    ) -> ControlFlow<Collision<A::T>>;

    /// Update triangles in the [`State`].
    fn update(state: &mut State, scene: &Scene<Sc, Map>);
}
