// #[macro_use]
// mod dynamic;
mod tilemap;

#[cfg(feature = "arraymap")]
mod arraymap;
#[cfg(feature = "bitmap")]
mod bitmap;
#[cfg(feature = "packmap")]
mod packmap;

#[cfg(feature = "mdarray")]
mod impl_mdarray;
#[cfg(feature = "ndarray")]
mod impl_ndarray;

#[cfg(feature = "arraymap")]
pub use arraymap::*;
#[cfg(feature = "bitmap")]
pub use bitmap::BitMap;
#[cfg(feature = "packmap")]
pub use packmap::PackMap;

pub use tilemap::{TileMap, TileMapView};
