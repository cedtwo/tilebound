use std::fmt::Debug;
use std::ops::*;

use marker_value::MarkerValue;

use crate::plane::axis::{Axis, AxisX, AxisY};
use crate::plane::endpoint::{Endpoint, EndpointBound};

/// # AxisMask
///
/// Bitmask representation of directions or axis bound extremes.
///
/// Bounds are represented as the 4 LSB, ordered by the *lower* and *upper* bounds of the `x` and
/// `y` axes respectively. In binary, each bit represents `[_, _, _, _, down, up, right, left]`
/// or `[_, _, _, _, bottom, top, right, left]`.
#[derive(Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AxisMask(pub u8);

impl AxisMask {
    /// Create a `AxisMask` with no bits set.
    pub const NONE: AxisMask = AxisMask::from_bits(0b0000_0000);
    /// Create a `AxisMask` with all bits set.
    pub const ALL: AxisMask = AxisMask::from_bits(0b0000_1111);

    /// Create a `AxisMask` with the bits of axis *x* set.
    pub const X: AxisMask = AxisMask::from_bits(0b0000_0011);
    /// Create a `AxisMask` with the bits of axis *y* set.
    pub const Y: AxisMask = AxisMask::from_bits(0b0000_1100);

    /// Create a `AxisMask` with only the left bit set.
    pub const LEFT: AxisMask = AxisMask::from_bits(0b0000_0001);
    /// Create a `AxisMask` with only the right bit set.
    pub const RIGHT: AxisMask = AxisMask::from_bits(0b0000_0010);
    /// Create a `AxisMask` with only the top bit set.
    pub const TOP: AxisMask = AxisMask::from_bits(0b0000_0100);
    /// Create a `AxisMask` with only the bottom bit set.
    pub const BOTTOM: AxisMask = AxisMask::from_bits(0b0000_1000);

    /// Create a `AxisMask` the top and left bits set.
    pub const TOP_LEFT: AxisMask = AxisMask::from_bits(0b0000_0101);
    /// Create a `AxisMask` with the bottom and left bits set.
    pub const BOTTOM_LEFT: AxisMask = AxisMask::from_bits(0b0000_1001);
    /// Create a `AxisMask` with the top and right bits set.
    pub const TOP_RIGHT: AxisMask = AxisMask::from_bits(0b0000_0110);
    /// Create a `AxisMask` with the bottom and right bits set.
    pub const BOTTOM_RIGHT: AxisMask = AxisMask::from_bits(0b0000_1010);

    /// Create a `AxisMask` from the given bits.
    pub const fn from_bits(mask: u8) -> Self {
        AxisMask(mask)
    }

    /// Create an `AxisMask` with all bits of the given axis set.
    pub fn from_axis<A: Axis>() -> Self {
        match A::VALUE {
            AxisX::VALUE => AxisMask::X,
            AxisY::VALUE => AxisMask::Y,
            _ => unreachable!(),
        }
    }

    /// Create an `AxisMask` with only the bit of the given axis and [`Endpoint<A>`] set.
    pub fn from_end<A: Axis>(end: Endpoint<A>) -> Self {
        let mask = match (A::VALUE, *end) {
            (AxisX::VALUE, EndpointBound::Lower) => AxisMask::LEFT.0,
            (AxisX::VALUE, EndpointBound::Upper) => AxisMask::RIGHT.0,
            (AxisY::VALUE, EndpointBound::Lower) => AxisMask::TOP.0,
            (AxisY::VALUE, EndpointBound::Upper) => AxisMask::BOTTOM.0,
            _ => unreachable!(),
        };
        AxisMask(mask)
    }

    /// Returns `true` if the bit of the given axis and [`Endpoint<A>`] is set.
    pub fn end_is_set<A: Axis>(&self, end: Endpoint<A>) -> bool {
        self.0 & AxisMask::from_end(end).0 != 0
    }

    /// Returns `true` if either bit of the given [`Endpoint<A>`] is set.
    pub fn any_end_is_set<A>(&self, end: Endpoint<A>) -> bool {
        match *end {
            EndpointBound::Lower => self.0 & AxisMask::TOP_LEFT.0 != 0,
            EndpointBound::Upper => self.0 & AxisMask::BOTTOM_RIGHT.0 != 0,
        }
    }

    /// Select only the bits of the given axis.
    pub fn isolate_axis<A: Axis>(self) -> AxisMask {
        AxisMask(self.0 & Self::from_axis::<A>().0)
    }

    /// Set the bit at the given axis [`Endpoint<A>`].
    pub fn set_end<A: Axis>(&mut self, end: Endpoint<A>) {
        self.0 |= AxisMask::from_end::<A>(end).0;
    }

    /// Clear all bits of the `AxisMask`.
    pub fn clear(&mut self) {
        self.0 = 0;
    }

    /// Clear the bit of the given axis and `Sign`.
    pub fn clear_end<A: Axis>(&mut self, sign: Endpoint<A>) {
        self.0 &= !Self::from_end::<A>(sign).0;
    }

    /// Clear the bits of the given axis.
    pub fn clear_axis<A: Axis>(&mut self) {
        self.0 &= Self::from_axis::<A::T>().0;
    }

    /// Transpose the bits of each axis.
    ///
    /// # Example
    /// ```rust
    /// # use tilebound::plane::axis::AxisMask;
    /// assert_eq!(AxisMask::LEFT.transpose(), AxisMask::TOP);
    /// assert_eq!(AxisMask::BOTTOM.transpose(), AxisMask::RIGHT);
    /// assert_eq!(AxisMask::TOP_LEFT.transpose(), AxisMask::TOP_LEFT);
    /// ```
    pub const fn transpose(self) -> Self {
        let mask = (self.0 & 0b0011) << 2 | (self.0 & 0b1100) >> 2;
        AxisMask(mask)
    }

    /// Return the first set [`Endpoint<A>`] (using LSB ordering) for the given axis.
    pub fn first_end<A: Axis>(&self) -> Option<Endpoint<A>> {
        // self.isolate_axis::<A>().any_first_end()

        let mask = *&self.isolate_axis::<A>();
        if (mask & Self::TOP_LEFT).0 != 0 {
            Some(Endpoint::LOWER)
        } else if (mask & Self::BOTTOM_RIGHT).0 != 0 {
            Some(Endpoint::UPPER)
        } else {
            None
        }
    }

    // /// Return the first set [`Endpoint<A>`] (using LSB ordering) for either axis.
    // pub fn any_first_end(&self) -> Option<Endpoint<A>> {
    //     let mask = *self;
    //     if (mask & Self::TOP_LEFT).0 != 0 {
    //         Some(Endpoint<A>::LOWER)
    //     } else if (mask & Self::BOTTOM_RIGHT).0 != 0 {
    //         Some(Endpoint<A>::UPPER)
    //     } else {
    //         None
    //     }
    // }

    // /// Return the first set [`Endpoint<A>`] (using LSB ordering) for either axis. Assumes at least one
    // /// bit is set, panicing in debug and returning [`Endpoint<A>::Upper`] where no bit is set. This
    // /// should only be used where certain (at least) one bit is set. Prefer
    // /// [`AxisMask::any_first_end`] if unsure.
    // pub fn any_first_end_unchecked(&self) -> Endpoint<A> {
    //     debug_assert!(self.any());
    //     let mask = *self;
    //     if (mask & Self::TOP_LEFT).0 != 0 {
    //         Endpoint::LOWER
    //     } else {
    //         debug_assert!((mask & Self::BOTTOM_RIGHT).0 != 0);
    //         Endpoint<A>::UPPER
    //     }
    // }

    /// Returns true if the [`AxisMask::LEFT`] bit is set.
    pub const fn left(&self) -> bool {
        self.0 & AxisMask::LEFT.0 > 0
    }

    /// Returns true if the [`AxisMask::RIGHT`] bit is set.
    pub const fn right(&self) -> bool {
        self.0 & AxisMask::RIGHT.0 > 0
    }

    /// Returns true if the [`AxisMask::TOP`] bit is set.
    pub const fn top(&self) -> bool {
        self.0 & AxisMask::TOP.0 > 0
    }

    /// Returns true if the [`AxisMask::BOTTOM`] bit is set.
    pub const fn bottom(&self) -> bool {
        self.0 & AxisMask::BOTTOM.0 > 0
    }

    /// Returns `true` if any bit is set.
    pub const fn any(&self) -> bool {
        self.0 != 0
    }

    /// Returns `true` if all bits are set.
    pub const fn all(&self) -> bool {
        self.0 == Self::ALL.0
    }

    /// Returns `true` if any bit is set on the given axis.
    pub fn any_on_axis<A: Axis>(&self) -> bool {
        match A::VALUE {
            AxisX::VALUE => self.0 & Self::X.0 != 0,
            AxisY::VALUE => self.0 & Self::Y.0 != 0,
            _ => unreachable!(),
        }
    }

    /// Returns `true` if all bits are set on the given axis.
    pub fn all_on_axis<A: Axis>(&self) -> bool {
        match A::VALUE {
            AxisX::VALUE => self.0 == Self::X.0,
            AxisY::VALUE => self.0 == Self::Y.0,
            _ => unreachable!(),
        }
    }

    /// Returns `true` if no bits are set.
    #[inline]
    pub const fn none(&self) -> bool {
        self.0 == Self::NONE.0
    }
}

impl Debug for AxisMask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AxisMask")
            .field(&format!("{:#06b}", self.0))
            .finish()
    }
}

impl Deref for AxisMask {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AxisMask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BitAnd<AxisMask> for AxisMask {
    type Output = AxisMask;

    fn bitand(self, rhs: AxisMask) -> Self::Output {
        AxisMask(self.0 & rhs.0)
    }
}

impl BitAndAssign<AxisMask> for AxisMask {
    fn bitand_assign(&mut self, rhs: AxisMask) {
        self.0 &= rhs.0;
    }
}

impl BitOr<AxisMask> for AxisMask {
    type Output = AxisMask;

    fn bitor(self, rhs: AxisMask) -> Self::Output {
        AxisMask(self.0 | rhs.0)
    }
}

impl BitOrAssign<AxisMask> for AxisMask {
    fn bitor_assign(&mut self, rhs: AxisMask) {
        self.0 |= rhs.0
    }
}

impl BitXor<AxisMask> for AxisMask {
    type Output = AxisMask;

    fn bitxor(self, rhs: AxisMask) -> Self::Output {
        AxisMask(self.0 ^ rhs.0)
    }
}

impl BitXorAssign<AxisMask> for AxisMask {
    fn bitxor_assign(&mut self, rhs: AxisMask) {
        self.0 ^= rhs.0
    }
}

impl Shl<AxisMask> for AxisMask {
    type Output = AxisMask;

    fn shl(self, rhs: AxisMask) -> Self::Output {
        AxisMask(self.0 << rhs.0)
    }
}

impl ShlAssign<u8> for AxisMask {
    fn shl_assign(&mut self, rhs: u8) {
        self.0 <<= rhs
    }
}

impl Shr<u8> for AxisMask {
    type Output = AxisMask;

    fn shr(self, rhs: u8) -> Self::Output {
        AxisMask(self.0 >> rhs)
    }
}

impl ShrAssign<u8> for AxisMask {
    fn shr_assign(&mut self, rhs: u8) {
        self.0 >>= rhs
    }
}

impl Not for AxisMask {
    type Output = AxisMask;

    fn not(self) -> Self::Output {
        AxisMask(!self.0 & *AxisMask::ALL)
    }
}

impl From<u8> for AxisMask {
    fn from(mask: u8) -> Self {
        AxisMask(mask)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn clear_end() {
        let mut mask_br = AxisMask::ALL;
        mask_br.clear_end::<AxisX>(Endpoint::LOWER);
        mask_br.clear_end::<AxisY>(Endpoint::LOWER);
        let mut mask_tl = AxisMask::ALL;
        mask_tl.clear_end::<AxisX>(Endpoint::UPPER);
        mask_tl.clear_end::<AxisY>(Endpoint::UPPER);

        assert_eq!(mask_br, AxisMask::BOTTOM_RIGHT);
        assert_eq!(mask_tl, AxisMask::TOP_LEFT);
    }

    #[test]
    fn clear_axis() {
        let mut mask_x = AxisMask::ALL;
        mask_x.clear_axis::<AxisY>();
        let mut mask_y = AxisMask::ALL;
        mask_y.clear_axis::<AxisX>();

        assert_eq!(mask_x, AxisMask::X);
        assert_eq!(mask_y, AxisMask::Y);
    }

    #[test]
    fn transpose() {
        let mask_x = AxisMask::X;
        let mask_y = AxisMask::Y;
        let mask_l = AxisMask::LEFT;
        let mask_r = AxisMask::RIGHT;
        let mask_t = AxisMask::TOP;
        let mask_b = AxisMask::BOTTOM;

        assert_eq!(mask_x.transpose(), AxisMask::Y);
        assert_eq!(mask_y.transpose(), AxisMask::X);
        assert_eq!(mask_l.transpose(), AxisMask::TOP);
        assert_eq!(mask_r.transpose(), AxisMask::BOTTOM);
        assert_eq!(mask_t.transpose(), AxisMask::LEFT);
        assert_eq!(mask_b.transpose(), AxisMask::RIGHT);
    }
}
