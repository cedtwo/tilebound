use std::fmt::Debug;
use std::ops::*;

use marker_value::MarkerValue;
use tilebound::plane::axis::{Axis, AxisMask, AxisVec, AxisX, AxisY};
use tilebound::plane::endpoint::Endpoint;

/// # VertexMask
///
/// Bitmask representation of quad vertices.
///
/// Vertices are represented as the 4 LSB, ordered by the cartesian product of the *lower* and
/// *upper* endpoint of the *x* and *y* axes respectively. Where the LSB is index zero, each bit
/// (from the right) represents `[.., bottom-right, top_right, bottom-left, top-left]`. See also
/// [`VertexTile`](crate::VertexTile).
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub struct VertexMask(u8);

impl VertexMask {
    /// Create a `VertexMask` with no bits set.
    pub const NONE: VertexMask = VertexMask::from_bits(0b0000_0000);
    /// Create a `VertexMask` with all bits set.
    pub const ALL: VertexMask = VertexMask::from_bits(0b0000_1111);

    /// Create a `VertexMask` with the left bits set.
    pub const LEFT: VertexMask = VertexMask::from_bits(0b0000_0011);
    /// Create a `VertexMask` with the right bits set.
    pub const RIGHT: VertexMask = VertexMask::from_bits(0b0000_1100);
    /// Create a `VertexMask` with the top bits set.
    pub const TOP: VertexMask = VertexMask::from_bits(0b0000_0101);
    /// Create a `VertexMask` with the bottom bits set.
    pub const BOTTOM: VertexMask = VertexMask::from_bits(0b0000_1010);

    /// Create a `VertexMask` with only the top-left bit set.
    pub const TOP_LEFT_INCL: VertexMask = VertexMask::from_bit_index::<0>();
    /// Create a `VertexMask` with only the bottom-left bit set.
    pub const BOTTOM_LEFT_INCL: VertexMask = VertexMask::from_bit_index::<1>();
    /// Create a `VertexMask` with only the top-right bit set.
    pub const TOP_RIGHT_INCL: VertexMask = VertexMask::from_bit_index::<2>();
    /// Create a `VertexMask` with only the bottom-right bit set.
    pub const BOTTOM_RIGHT_INCL: VertexMask = VertexMask::from_bit_index::<3>();

    /// Create a `VertexMask` with only the top-left bit clear.
    pub const TOP_LEFT_EXCL: VertexMask = VertexMask::from_bits(0b0000_1110);
    /// Create a `VertexMask` with only the bottom-left bit clear.
    pub const BOTTOM_LEFT_EXCL: VertexMask = VertexMask::from_bits(0b0000_1101);
    /// Create a `VertexMask` with only the top-right bit clear.
    pub const TOP_RIGHT_EXCL: VertexMask = VertexMask::from_bits(0b1011);
    /// Create a `VertexMask` with only the bottom-right bit clear.
    pub const BOTTOM_RIGHT_EXCL: VertexMask = VertexMask::from_bits(0b0000_0111);

    /// Create a `VertexMask` from the given bits.
    #[inline]
    pub const fn from_bits(mask: u8) -> Self {
        VertexMask(mask)
    }

    /// Create a `VertexMask` with the given `vertex_endpoint` set.
    pub const fn from_vertex(vertex_endpoint: AxisVec<Endpoint>) -> Self {
        const TOP_LEFT: AxisVec<Endpoint> = AxisVec::new(Endpoint::LOW, Endpoint::LOW);
        const BOTTOM_LEFT: AxisVec<Endpoint> = AxisVec::new(Endpoint::LOW, Endpoint::UPP);
        const TOP_RIGHT: AxisVec<Endpoint> = AxisVec::new(Endpoint::UPP, Endpoint::LOW);
        const BOTTOM_RIGHT: AxisVec<Endpoint> = AxisVec::new(Endpoint::UPP, Endpoint::UPP);

        match vertex_endpoint {
            TOP_LEFT => VertexMask::TOP_LEFT_INCL,
            BOTTOM_LEFT => VertexMask::BOTTOM_LEFT_INCL,
            TOP_RIGHT => VertexMask::TOP_RIGHT_INCL,
            BOTTOM_RIGHT => VertexMask::BOTTOM_RIGHT_INCL,
        }
    }

    /// Create a new `VertexMask` with the bit at the given const `INDEX` set.
    pub const fn from_bit_index<const INDEX: u8>() -> Self {
        assert!(INDEX < 4);
        VertexMask(0b0000_0001 << INDEX)
    }

    /// Create a `VertexMask` with only the bits of the given `AXIS` and `Endpoint` set.
    pub fn from_axis<A: Axis>(endpoint: Endpoint) -> VertexMask {
        let bits = match (A::VALUE, endpoint) {
            (AxisX::VALUE, Endpoint::LOW) => VertexMask::LEFT.0,
            (AxisX::VALUE, Endpoint::UPP) => VertexMask::RIGHT.0,
            (AxisY::VALUE, Endpoint::LOW) => VertexMask::TOP.0,
            (AxisY::VALUE, Endpoint::UPP) => VertexMask::BOTTOM.0,
            _ => unreachable!(),
        };

        VertexMask(bits)
    }

    /// Create a `VertexMask` from the given [`AxisMask`], selecting the adjacent vertex bits for
    /// any axis in the `mask`.
    pub const fn from_axismask(mask: AxisMask) -> Self {
        match mask {
            AxisMask::LEFT => Self::LEFT,
            AxisMask::RIGHT => Self::RIGHT,
            AxisMask::TOP => Self::TOP,
            AxisMask::BOTTOM => Self::BOTTOM,
            AxisMask::TOP_LEFT => Self::BOTTOM_RIGHT_EXCL,
            AxisMask::TOP_RIGHT => Self::BOTTOM_LEFT_EXCL,
            AxisMask::BOTTOM_LEFT => Self::TOP_RIGHT_EXCL,
            AxisMask::BOTTOM_RIGHT => Self::TOP_LEFT_EXCL,
            AxisMask::NONE => Self::NONE,
            _ => Self::ALL,
        }
    }

    /// Returns `true` if the vertex at the given const `INDEX` is set.
    pub const fn index_is_set<const INDEX: u8>(&self) -> bool {
        self.0 & 0b0000_0001 << INDEX != 0
    }

    /// Returns `true` if the given `vertex_endpoint` is set.
    pub const fn vertex_is_set(&self, vertex_endpoint: AxisVec<Endpoint>) -> bool {
        (self.0 & VertexMask::from_vertex(vertex_endpoint).0) != 0
    }

    /// Returns `true` if only the given `vertex_endpoint` is set.
    pub const fn vertex_is_set_excl(&self, vertex_endpoint: AxisVec<Endpoint>) -> bool {
        self.0 == VertexMask::from_vertex(vertex_endpoint).0
    }

    /// Returns `true` if both vertices of the given edge are set.
    pub fn edge_all_set<A: Axis>(&self, endpoint: Endpoint) -> bool {
        self.isolate_edge::<A>(endpoint).0.count_ones() == 2
    }

    /// Returns `true` if either vertex of the given edge are set.
    pub fn edge_any_set<A: Axis>(&self, endpoint: Endpoint) -> bool {
        self.isolate_edge::<A>(endpoint).0.count_ones() != 0
    }

    /// Returns an [`AxisMask`] retaining only the endpoints of the given edge. Note that the
    /// returned [`AxisMask`] retains bits on the **transpose** axis (eg. isolating the *top* edge
    /// will return an [`AxisMask`] with (at most) the *left* and *right* bits set).
    ///
    /// # Example
    /// ```
    /// # use tilebound_halfslope::prelude::*;
    /// #
    /// let top_left = VertexMask::TOP_LEFT_INCL;
    /// let left = VertexMask::LEFT;
    /// let bottom_right = VertexMask::BOTTOM_RIGHT_INCL;
    ///
    /// // Select the left edge (The lower endpoint of axis X).
    /// assert_eq!(top_left.isolate_edge::<AxisX>(Endpoint::Lower), AxisMask::TOP);
    /// assert_eq!(left.isolate_edge::<AxisX>(Endpoint::Lower), AxisMask::Y);
    /// assert_eq!(bottom_right.isolate_edge::<AxisX>(Endpoint::Lower), AxisMask::NONE);
    /// ```
    pub fn isolate_edge<A: Axis>(&self, endpoint: Endpoint) -> AxisMask {
        let mut isolated = *self;
        isolated.clear_edge::<A>(!endpoint);
        isolated.isolate_axis::<A>()
    }

    /// Returns an [`AxisMask`] retaining only the endpoints of edges on the given axis. Note that
    /// the returned [`AxisMask`] retains bits on the **transpose** axis (eg. isolating the *top*
    /// edge will return an [`AxisMask`] with (at most) the *left* and *right* bits set). This is
    /// equivalent to [`VertexMask::isolate_edge`] where only a single edge on the given axis is
    /// present.
    ///
    /// # Example
    /// ```
    /// # use tilebound_halfslope::prelude::*;
    /// #
    /// let top_left = VertexMask::TOP_LEFT_INCL;
    /// let left = VertexMask::LEFT;
    /// let bottom_right = VertexMask::BOTTOM_RIGHT_INCL;
    ///
    /// // Select any endpoint on an axis X edge.
    /// assert_eq!(top_left.isolate_axis::<AxisX>(), AxisMask::TOP);
    /// assert_eq!(left.isolate_axis::<AxisX>(), AxisMask::Y);
    /// assert_eq!(bottom_right.isolate_axis::<AxisX>(), AxisMask::BOTTOM);
    /// ```
    pub fn isolate_axis<A: Axis>(&self) -> AxisMask {
        let mut mask = AxisMask::NONE;

        match A::VALUE {
            AxisX::VALUE => {
                if self.0 & Self::TOP.0 != 0 {
                    mask |= AxisMask::TOP
                }
                if self.0 & Self::BOTTOM.0 != 0 {
                    mask |= AxisMask::BOTTOM
                }
            }
            AxisY::VALUE => {
                if self.0 & Self::LEFT.0 != 0 {
                    mask |= AxisMask::LEFT
                }
                if self.0 & Self::RIGHT.0 != 0 {
                    mask |= AxisMask::RIGHT
                }
            }
            _ => unreachable!(),
        }
        mask
    }

    /// Clear the bits of the given axis `A` and `Endpoint`.
    pub fn clear_edge<A: Axis>(&mut self, endpoint: Endpoint) {
        self.0 &= !Self::from_axis::<A>(endpoint).0;
    }

    /// For the given axis `A`, return the `Endpoint` of the vertex opposite the hypotenuse. Returns
    /// `None` if `self` is not a triangle.
    pub fn tri_endpoint<A: Axis>(&self) -> Option<Endpoint> {
        match self {
            &Self::BOTTOM_RIGHT_EXCL => match A::VALUE {
                AxisX::VALUE => Some(Endpoint::LOW),
                AxisY::VALUE => Some(Endpoint::LOW),
                _ => unreachable!(),
            },
            &Self::TOP_RIGHT_EXCL => match A::VALUE {
                AxisX::VALUE => Some(Endpoint::LOW),
                AxisY::VALUE => Some(Endpoint::UPP),
                _ => unreachable!(),
            },
            &Self::BOTTOM_LEFT_EXCL => match A::VALUE {
                AxisX::VALUE => Some(Endpoint::UPP),
                AxisY::VALUE => Some(Endpoint::LOW),
                _ => unreachable!(),
            },
            &Self::TOP_LEFT_EXCL => match A::VALUE {
                AxisX::VALUE => Some(Endpoint::UPP),
                AxisY::VALUE => Some(Endpoint::UPP),
                _ => unreachable!(),
            },
            _ => None,
        }
    }

    /// Select the bit of the vertex opposite the hypotenuse. Returns `None` if `self` is not a
    /// triangle.
    pub const fn tri_mask(&self) -> Option<VertexMask> {
        match self {
            &Self::BOTTOM_RIGHT_EXCL => Some(Self::TOP_LEFT_INCL),
            &Self::TOP_RIGHT_EXCL => Some(Self::BOTTOM_LEFT_INCL),
            &Self::BOTTOM_LEFT_EXCL => Some(Self::TOP_RIGHT_INCL),
            &Self::TOP_LEFT_EXCL => Some(Self::BOTTOM_RIGHT_INCL),
            _ => None,
        }
    }

    /// Transpose the vertices of each axis.
    ///
    /// # Example
    /// ```rust
    /// # use tilebound_halfslope::prelude::*;
    /// assert_eq!(VertexMask::LEFT.transpose(), VertexMask::TOP);
    /// assert_eq!(VertexMask::BOTTOM.transpose(), VertexMask::RIGHT);
    /// assert_eq!(VertexMask::TOP_LEFT_EXCL.transpose(), VertexMask::TOP_LEFT_EXCL);
    /// ```
    pub const fn transpose(self) -> Self {
        // Swap the center two vertices.
        let mask = (self.0 & 0b1001)
            | match self.0 & 0b0110 {
                v @ (0b0010 | 0b0100) => v ^ 0b0110,
                /* 0b0000 | 0b0110 */ v => v,
            };
        VertexMask(mask)
    }

    /// Isolate the bits of the given axis `A` and [`Endpoint`], compacting to the
    /// right. Returns a `u8`.
    pub fn compact<A: Axis>(self, endpoint: Endpoint) -> u8 {
        let mask = match A::VALUE {
            AxisX::VALUE => self,
            AxisY::VALUE => self.transpose(),
            _ => unreachable!(),
        };

        match endpoint {
            Endpoint::LOW => mask.0 & 0b0011,
            Endpoint::UPP => mask.0 >> 2,
        }
    }

    /// Return the `VertexMask` as a `u8` bitmask.
    pub const fn into_raw(self) -> u8 {
        self.0
    }

    /// Returns `true` if any bit is set.
    #[inline]
    pub const fn any(&self) -> bool {
        self.0 != 0
    }

    /// Returns `true` if all bits are set.
    #[inline]
    pub const fn all(&self) -> bool {
        self.0 == Self::ALL.0
    }

    /// Returns `true` if no bits are set.
    #[inline]
    pub const fn none(&self) -> bool {
        self.0 == Self::NONE.0
    }
}

impl Debug for VertexMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VertexMask")
            .field(&format!("{:#06b}", self.0))
            .finish()
    }
}

impl Deref for VertexMask {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for VertexMask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BitAnd<VertexMask> for VertexMask {
    type Output = VertexMask;

    fn bitand(self, rhs: VertexMask) -> Self::Output {
        VertexMask(self.0 & rhs.0)
    }
}

impl BitAndAssign<VertexMask> for VertexMask {
    fn bitand_assign(&mut self, rhs: VertexMask) {
        self.0 &= rhs.0;
    }
}

impl BitOr<VertexMask> for VertexMask {
    type Output = VertexMask;

    fn bitor(self, rhs: VertexMask) -> Self::Output {
        VertexMask(self.0 | rhs.0)
    }
}

impl BitOrAssign<VertexMask> for VertexMask {
    fn bitor_assign(&mut self, rhs: VertexMask) {
        self.0 |= rhs.0
    }
}

impl BitXor<VertexMask> for VertexMask {
    type Output = VertexMask;

    fn bitxor(self, rhs: VertexMask) -> Self::Output {
        VertexMask(self.0 ^ rhs.0)
    }
}

impl BitXorAssign<VertexMask> for VertexMask {
    fn bitxor_assign(&mut self, rhs: VertexMask) {
        self.0 ^= rhs.0
    }
}

impl Shl<VertexMask> for VertexMask {
    type Output = VertexMask;

    fn shl(self, rhs: VertexMask) -> Self::Output {
        VertexMask(self.0 << rhs.0)
    }
}

impl ShlAssign<u8> for VertexMask {
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs
    }
}

impl Shr<u8> for VertexMask {
    type Output = VertexMask;

    fn shr(self, rhs: u8) -> Self::Output {
        VertexMask(self.0 >> rhs)
    }
}

impl ShrAssign<u8> for VertexMask {
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs
    }
}

impl Not for VertexMask {
    type Output = VertexMask;

    fn not(self) -> Self::Output {
        VertexMask(!self.0 & *VertexMask::ALL)
    }
}
impl From<u8> for VertexMask {
    fn from(mask: u8) -> Self {
        VertexMask(mask)
    }
}

impl<'a> From<&'a u8> for VertexMask {
    fn from(mask: &'a u8) -> Self {
        VertexMask(*mask)
    }
}

impl From<AxisVec<Endpoint>> for VertexMask {
    fn from(value: AxisVec<Endpoint>) -> Self {
        VertexMask::from_vertex(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolate_edge() {
        let tl_mask = VertexMask::BOTTOM_RIGHT_EXCL;
        let tl_left = tl_mask.isolate_edge::<AxisX>(Endpoint::LOW);
        let tl_right = tl_mask.isolate_edge::<AxisX>(Endpoint::UPP);
        let tl_top = tl_mask.isolate_edge::<AxisY>(Endpoint::LOW);
        let tl_bottom = tl_mask.isolate_edge::<AxisY>(Endpoint::UPP);
        assert_eq!(tl_left, AxisMask::Y);
        assert_eq!(tl_right, AxisMask::TOP);
        assert_eq!(tl_top, AxisMask::X);
        assert_eq!(tl_bottom, AxisMask::LEFT);

        let tr_mask = VertexMask::BOTTOM_LEFT_EXCL;
        let tr_left = tr_mask.isolate_edge::<AxisX>(Endpoint::LOW);
        let tr_right = tr_mask.isolate_edge::<AxisX>(Endpoint::UPP);
        let tr_top = tr_mask.isolate_edge::<AxisY>(Endpoint::LOW);
        let tr_bottom = tr_mask.isolate_edge::<AxisY>(Endpoint::UPP);
        assert_eq!(tr_left, AxisMask::TOP);
        assert_eq!(tr_right, AxisMask::Y);
        assert_eq!(tr_top, AxisMask::X);
        assert_eq!(tr_bottom, AxisMask::RIGHT);
    }
}
