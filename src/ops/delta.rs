//! [`Delta`] initialization helper functions.
use crate::plane::axis::Axis;
use crate::plane::endpoint::Endpoint;
use crate::plane::scale::Scale;
use crate::schema::state::State;
use crate::topology::delta::Delta;
use crate::topology::vertex::Vertex;

/// Initialize a [`Delta`] from an `f32` value, returning `None` if the delta is invalid (`0.0`),
/// or if the [`State`] is attached at the `delta` endpoint.
pub fn delta_by<A: Axis, S: Scale, R>(state: &State<R>, delta: f32) -> Option<Delta<A>> {
    let end = Endpoint::from_f32(delta)?;

    match state.end_is_attached::<A>(end) {
        true => None,
        false => {
            let mut target = state.edge::<A>();
            target.set_inbound_end::<S>(end);
            target.translate::<S>(delta);

            Some(Delta::new(target))
        }
    }
}

/// Initialize a [`Delta`] to an `f32` value, returning `None` if the delta is already intersected,
/// or if the [`State`] is attached at the `delta` endpoint.
pub fn delta_to<A: Axis, S: Scale, R>(state: &State<R>, to: f32) -> Option<Delta<A>> {
    let edge = state.edge::<A>();

    if !edge.range::<S>().contains(&to) {
        // If the edge does not contain the `target` value, comparing against either
        // vertex gives a correct endpoint.
        let end = Endpoint::from_partial_cmp_unchecked(to, edge.inbound_vertex().to_pos::<S>());
        match state.end_is_attached::<A>(end) {
            true => None,
            false => {
                let target = Vertex::from_pos::<S>(to, end).into_edge(state.len::<A>());

                Some(Delta::new(target))
            }
        }
    } else {
        None
    }
}
