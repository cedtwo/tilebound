use std::error::Error;
use std::fmt::Display;

use crate::vertex_mask::VertexMask;

/// # VertexTile
///
/// `VertexTile` is implemented on [`TileMap`](crate::prelude::TileMap) elements that can be
/// represented as a collision [`VertexMask`].
pub trait VertexTile {
    fn mask(&self) -> VertexMask;
}

impl VertexTile for u8 {
    fn mask(&self) -> VertexMask {
        VertexMask::from_bits(*self)
    }
}

impl VertexTile for VertexPattern {
    fn mask(&self) -> VertexMask {
        self.into_mask()
    }
}

impl VertexTile for VertexMask {
    fn mask(&self) -> VertexMask {
        *self
    }
}

/// # VertexPattern
///
/// `VertexPattern` is an `enum` representation of all valid [`VertexMask`] variants. It offers no
/// functionality over [`VertexMask`] and is only used for convenience. See [`VertexMask`].
#[repr(u8)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum VertexPattern {
    /// No collision.
    #[default]
    None = VertexMask::NONE.into_raw(), // 0
    /// Collides with the top, left, bottom and right edge.
    Full = VertexMask::ALL.into_raw(), // 15

    /// Collides with the top edge.
    TopOneway = VertexMask::TOP.into_raw(), // 5
    /// Collides with the left edge.
    LeftOneway = VertexMask::LEFT.into_raw(), // 3
    /// Collides with the bottom edge.
    BottomOneway = VertexMask::BOTTOM.into_raw(), // 10
    /// Collides with the right edge.
    RightOneway = VertexMask::RIGHT.into_raw(), // 12

    /// Collides with the top, left and hypotenuse edges.
    TopLeftTri = VertexMask::BOTTOM_RIGHT_EXCL.into_raw(), // 7
    /// Collides with the bottom, left and hypotenuse edges.
    BottomLeftTri = VertexMask::TOP_RIGHT_EXCL.into_raw(), // 11
    /// Collides with the top, right and hypotenuse edges.
    TopRightTri = VertexMask::BOTTOM_LEFT_EXCL.into_raw(), // 13
    /// Collides with the bottom, right and hypotenuse edges.
    BottomRightTri = VertexMask::TOP_LEFT_EXCL.into_raw(), // 14
}

impl VertexPattern {
    /// Represent the `VertexTile` as a `VertexMask`.
    #[inline]
    pub fn into_mask(self) -> VertexMask {
        VertexMask::from_bits(self as u8)
    }
}

impl TryFrom<u8> for VertexPattern {
    type Error = VertexTileConversionError;

    fn try_from(mask: u8) -> Result<Self, Self::Error> {
        match VertexMask::from_bits(mask) {
            VertexMask::NONE => Ok(VertexPattern::None),
            VertexMask::ALL => Ok(VertexPattern::Full),
            VertexMask::TOP => Ok(VertexPattern::TopOneway),
            VertexMask::LEFT => Ok(VertexPattern::LeftOneway),
            VertexMask::BOTTOM => Ok(VertexPattern::BottomOneway),
            VertexMask::RIGHT => Ok(VertexPattern::RightOneway),
            VertexMask::BOTTOM_RIGHT_EXCL => Ok(VertexPattern::TopLeftTri),
            VertexMask::TOP_RIGHT_EXCL => Ok(VertexPattern::BottomLeftTri),
            VertexMask::BOTTOM_LEFT_EXCL => Ok(VertexPattern::TopRightTri),
            VertexMask::TOP_LEFT_EXCL => Ok(VertexPattern::BottomRightTri),
            mask => Err(VertexTileConversionError(mask)),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct VertexTileConversionError(VertexMask);

impl Display for VertexTileConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "No VertexTile variant matching mask: {:?} ({})",
            self.0,
            self.0.into_raw()
        )
    }
}

impl Error for VertexTileConversionError {}

/// # VertexEdge
///
/// `VertexEdge` is an `enum` representation of two adjacent [`VertexMask`] vertices.
#[repr(u8)]
#[derive(Default, Debug, PartialEq, Eq, Clone, Copy)]
pub enum VertexEdge {
    /// No vertices.
    #[default]
    None = 0b00,
    /// A triangle tile at the lower endpoint.
    Lower = 0b01,
    /// A triangle tile at the upper endpoint.
    Upper = 0b10,
    /// A solid  edge.
    All = 0b11,
}

impl From<u8> for VertexEdge {
    fn from(value: u8) -> Self {
        match value & Self::All as u8 {
            0b00 => Self::None,
            0b01 => Self::Lower,
            0b10 => Self::Upper,
            0b11 => Self::All,
            _ => unreachable!(),
        }
    }
}
