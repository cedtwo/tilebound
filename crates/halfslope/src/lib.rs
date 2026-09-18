//! # Tilebound-Halfslope
//!
//! A tilemap collision libary supporting half-tile slopes.
//!
//! ## Summary
//!
//! `tilebound_halfslope` is an implementation of [`tilebound`] supporting half-tile 45 degree
//! slopes, and sliding and attaching behaviour. It offers the following features:
//! - Half tile slopes of any direction,
//! - One-way tiles of any direction,
//! - The choice to either slide or attach (stop displacement) on colliding with a hypotenuse,
//! - Axis seperate *sweep* displacement that only checks tiles on-demand (when moving into a new
//! tile, or off a ledge),
//! - Attach/detach behaviour (for asserting collisions) when moving on/off surfaces,
//! - Tile range inspection on collision (eg. Check where the first halfslope tile is when colliding
//! with a ledge),
//! - Perfect tile-bound alignment (no fractional displacement for an edge on a tile bound),
//! - Support tiles as small as half a byte in size.
//!
//! A large number of these features are provided by the implementing library, [`tilebound`].
//!
//! ## features
//!
//! `tilebound_halfslope` supports the following features:
//!
//! Feature | Description | Exposed Type(s)
//! ---|---|---
//! `arraymap` | Exports tilemap implementations built on `std` types. | [`ArrayMap`](crate::prelude::ArrayMap), [`VecMap`](crate::prelude::VecMap)
//!
//! ## Example
//!
//! A minimal example is provided using `macroquad` as a backend. `bevy` uses a different axis
//! orientation so variables will either need to be manually oriented, or another rendering backend
//! (such as `bevy_framebuffer`) will be needed.
//!
//! ```bash
//! cargo run --example minimal --features="arraymap"
//! ```
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
