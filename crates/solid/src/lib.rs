//! # Tilebound-Solid
//!
//! A small boolean/bit tilemap collision libary.
//!
//! ## Summary
//!
//! `tilebound_solid` is a simle implementation of [`tilebound`] supporting arrays of simple
//! solid/empty tiles. It offers the following features:
//! - Axis seperate *sweep* displacement that only checks tiles on-demand (when moving into a new
//! tile),
//! - Attach/detach behaviour (for asserting collisions) when moving on/off surfaces,
//! - Tile range inspection on collision (eg. Check where the first solid tile is when colliding
//! with a ledge),
//! - Perfect tile-bound alignment (no fractional displacement for an edge on a tile bound),
//! - Support for `bool`, `u8` (0 or 1) and `bitvec` backed tile arrays.
//!
//! A large number of these features are provided by the implementing library, [`tilebound`].
//!
//! ## Example
//!
//! The following demonstrates `tilebound_solid` usage.
//!
//! ```rust
//! # use std::assert_matches;
//! # use tilebound_solid::prelude::*;
//! # #[cfg(feature = "arraymap")]
//! # {
//! // A 2*2 map with a single solid tile in the top-right.
//! const MAP: ArrayMap<2, 2, 4, u8> = ArrayMap::new_unchecked([
//!     0, 1,
//!     0, 0
//! ]);
//!
//! // Declare a constant tile size (a 16 unit square).
//! type Sc = ConSc<16>;
//! // Declare a `Context`, passing in the map and declaring any solid map bounds.
//! let ctx = Context::<Sc, _>::new(&MAP, AxisMask::NONE);
//!
//! // Create a mutable bounding box. Here we define a box at the top left of `(8.0 * 8.0)` units in size.
//! let mut rect = BoundBox::new((0.0, 0.0), (8.0, 8.0), AxisMask::NONE);
//!
//! let brk = ctx.sweep_by::<AxisX, _>(&mut rect, 16.0); // Displace 16.0 units to the right (one exact tile).
//!
//! assert_matches!(brk, Break::Collision(_)); // Assert we collided.
//! assert_eq!(rect.pos(), (8.0, 0.0).into()); // Assert we displaced only 8 units to the right.
//! assert_eq!(rect.attmask(), AxisMask::RIGHT); // Assert we are colliding on the right.
//!
//! let brk = ctx.sweep_by::<AxisY, _>(&mut rect, 16.0); // Displace 16.0 units down (one exact tile).
//!
//! assert_matches!(brk, Break::ReachedTarget); // Assert we reached the target.
//! assert_eq!(rect.pos(), (8.0, 16.0).into()); // Assert we displaced all 16 units down.
//! assert_eq!(rect.attmask(), AxisMask::NONE); // Assert we are no longer colliding on the right.
//! # }
//!```
//!
//! ## features
//!
//! `tilebound_solid` supports the following features:
//!
//! Feature | Description | Exposed Type(s)
//! ---|---|---
//! `arraymap` | Exports tilemap implementations built on `std` types. | [`ArrayMap`](crate::prelude::ArrayMap), [`VecMap`](crate::prelude::VecMap)
//! `bitmap` | Exports a `bitvec` backed tilemap (where each tile represents one bit). | [`BitMap`](crate::prelude::BitMap)
//!
//! ## Example
//!
//! Two examples are provided using `macroquad` as a backend. `bevy` uses a different axis
//! orientation so variables will either need to be manually oriented, or another rendering backend
//! (eg. `bevy_framebuffer`) will be needed.
//!
//! Example | Description | Command
//! ---|---|---
//! `minimal` | A minimal example with input and rendering. | `cargo run --example minimal --features="bitmap"`
//! `platformer` | A minimal example with gravity and jumping. | `cargo run --example platformer --features="bitmap"`
mod context;

mod collision;
mod tile;

pub mod prelude {
    //! Core types and traits.
    pub use tilebound::prelude::*;

    pub type State = tilebound::prelude::State<()>;

    // pub use crate::bnd_box::BoundBox;
    pub use crate::collision::{Collision, TileRange};
    pub use crate::context::Context;
    pub use crate::tile::SolidTile;
}
