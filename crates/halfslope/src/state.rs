use tilebound::plane::axis::{Axis, AxisMask};
use tilebound::plane::endpoint::Endpoint;
use tilebound::plane::scale::Scale;

use crate::prelude::State;
use crate::vertex_mask::VertexMask;

/// # StateExt
///
/// [`State`] operations, especially in relation to [`VertexMask`] resource handling.
pub trait StateExt {
    /// Set the tris of a given axis and `end`. Expects a `VertexMask` of **only** the bits of the
    /// given ended axis set.
    fn set_tris<A: Axis>(&mut self, end: Endpoint, mask: VertexMask);

    /// Clear the tris of the identified edge. See [`VertexMask::clear_edge`].
    fn clear_tris<A: Axis>(&mut self, end: Endpoint);

    /// Returns an [`AxisMask`] retaining only the endpoints of the given edge. See
    /// [`VertexMask::isolate_edge`].
    fn isolate_edge<A: Axis>(&mut self, end: Endpoint) -> AxisMask;

    /// Attempt to clear `VertexMask` tris that are no longer intersected.
    fn clear_disjoint_tris<A: Axis, Sc: Scale>(&mut self);
}

impl StateExt for State {
    fn set_tris<A: Axis>(&mut self, end: Endpoint, mask: VertexMask) {
        debug_assert!(mask.isolate_edge::<A>(!end).none());
        let state_mask = self.res_mut();
        state_mask.clear_edge::<A>(end);
        *state_mask |= mask;
    }

    fn clear_tris<A: Axis>(&mut self, end: Endpoint) {
        self.res_mut().clear_edge::<A>(end);
    }

    fn isolate_edge<A: Axis>(&mut self, end: Endpoint) -> AxisMask {
        self.res_mut().isolate_edge::<A>(end)
    }

    fn clear_disjoint_tris<A: Axis, Sc: Scale>(&mut self) {
        let end = self.inbound_endpoint::<A>();
        let idx_curr = self.index::<A, Sc>(!end);
        let idx_last = self.last_index::<A, Sc>(!end);

        if end.gt(idx_curr, idx_last) {
            self.res_mut().clear_edge::<A>(!end);
        }
    }
}
