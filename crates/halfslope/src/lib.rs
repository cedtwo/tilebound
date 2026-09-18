mod context;
mod handlers;
mod state;

mod collision;
mod tile;

mod vertex_mask;

pub mod prelude {
    //! Core types and traits.
    pub use tilebound::prelude::*;

    pub type State = tilebound::prelude::State<VertexMask>;

    pub use crate::collision::{Collision, EdgeRange};
    pub use crate::context::Context;
    pub use crate::state::StateExt;
    pub use crate::tile::{VertexEdge, VertexPattern, VertexTile};
    pub use crate::vertex_mask::VertexMask;
}
