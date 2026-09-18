//! # Tilebound
//!
//! A framework for resolving tilemap collisions.
//!
//! ## Summary
//!
//! `tilebound` is a library of types, representations and operations for handling and resolving
//! bounding-box displacement and collision on a 2d array. `tilebound` includes:
//! - Generic axis type enforcement and variable access and mutation (See [`Axis`](crate::prelude::Axis)
//! and [`AxisVec`](crate::prelude::AxisVec)),
//! - Delta endpoint spacing and orientation (See [`Delta`](crate::schema::delta::Delta) and
//! [`Endpoint`](crate::prelude::Endpoint)),
//! - Exact tile bound alignment with no [`f32::EPSILON`], [`f32::next_down`]/[`f32::next_up`] or
//! other offsetting (See [`Vertex`](crate::topology::vertex::Vertex) and [`Edge`](crate::topology::edge::Edge)),
//! - Axis generic tilemap implementations build on `[T]`, [`Vec`] and `BitVec`, and support for
//! `ndarray` and `mdarray` arrays (See [`TileMap`](crate::view::tilemap::TileMap) and
//! [`TileMapView`],
//! - Various helper methods and operations for initializing a delta, sweeping over a delta, checking tiles on-demand
//! and updating stale collisions (See [`crate::ops`]).
//!
//! ## Features
//!
//! `tilebound` exposes various [`TileMap`](crate::view::tilemap::TileMap) and [`TileMapView`](crate::view::tilemap::TileMapView)
//! implementations as optional features.
//!
//! Feature | Description | Exposed Type(s)
//! ---|---|---
//! `arraymap` | Exports tilemap implementations build on `std` types. | [`ArrayMap`](crate::prelude::ArrayMap), [`VecMap`](crate::prelude::VecMap)
//! `bitmap` | Exports a `bitvec` backed tilemap (where each tile represents one bit). | [`BitMap`](crate::prelude::BitMap)
//! `packmap` | Exports an *experimental* bitfield backed tilemap (where each tile represents `2` or `4` bits). | [`packmap`](crate::prelude::PackMap)
pub mod plane;
pub mod schema;
pub mod topology;
pub mod view;

pub mod ops;

pub mod prelude {
    //! Core types and traits.
    pub use crate::plane::axis::{Axis, AxisMask, AxisVec, AxisX, AxisY, DynAxis};
    pub use crate::plane::endpoint::Endpoint;
    pub use crate::plane::scale::{ConSc, Scale};

    pub use crate::schema::brk::Break;
    pub use crate::schema::scene::Scene;
    pub use crate::schema::state::State;

    pub use crate::view::tilemap::{TileMap, TileMapView};

    #[cfg(feature = "bitmap")]
    pub use crate::view::tilemap::BitMap;
    #[cfg(feature = "packmap")]
    pub use crate::view::tilemap::PackMap;
    #[cfg(feature = "arraymap")]
    pub use crate::view::tilemap::{ArrayMap, VecMap};

    #[cfg(feature = "bitmap")]
    pub use bitvec;
    #[cfg(feature = "packmap")]
    pub use subbyte_index;
}
