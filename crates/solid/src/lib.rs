mod context;

mod collision;
mod tile;

pub mod prelude {
    //! Core types and traits.
    pub use tilebound::prelude::*;

    pub type State = tilebound::prelude::State<()>;

    pub use crate::collision::{Collision, TileRange};
    pub use crate::context::Context;
    pub use crate::tile::SolidTile;
}
