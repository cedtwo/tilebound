#[cfg(feature = "bitmap")]
use tilebound::prelude::bitvec::ptr::{BitRef, Const};
#[cfg(feature = "bitmap")]
use tilebound::prelude::bitvec::store::BitStore;

/// # SolidTile
///
/// `SolidTile` is implemented on [`TileMap`](crate::prelude::TileMap) elements that can represent
/// a collision (or lack thereof).
pub trait SolidTile {
    /// Returns `true` if a tile is solid.
    fn is_solid(&self) -> bool;
}

impl SolidTile for bool {
    #[inline]
    fn is_solid(&self) -> bool {
        *self
    }
}

impl SolidTile for u8 {
    #[inline]
    fn is_solid(&self) -> bool {
        *self == 1
    }
}

impl<'a, T: SolidTile + Copy> SolidTile for &'a T {
    fn is_solid(&self) -> bool {
        (*self).is_solid()
    }
}

#[cfg(feature = "bitmap")]
impl<'a, Store: BitStore> SolidTile for BitRef<'a, Const, Store> {
    #[inline]
    fn is_solid(&self) -> bool {
        **self
    }
}
