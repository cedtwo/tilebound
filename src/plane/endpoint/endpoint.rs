use std::cmp::Ordering;
use std::ops::{Add, Not, Sub};

use num_traits::ConstOne;

use crate::plane::axis::AxisVec;

/// # Endpoint
///
/// Differentiates between the lower and upper endpoints on an axis.
///
/// `Endpoint` is primarily used for operation orientation (especially in relation to a delta). Many
/// operations (eg. [`Endpoint::gt`], [`Endpoint::lt`]) follow numerical ordering for the [`Endpoint::Upper`],
/// variant, but are reversed for [`Endpoint::Lower`] orientation.
///
/// # Example
///
/// ```
/// # use tilebound::plane::endpoint::Endpoint;
/// # use std::cmp::Ordering;
/// #
/// // For `Endpoint::Upper` `1` is greater than `0`.
/// assert!(Endpoint::Upper.gt(1, 0));
/// assert_eq!(Endpoint::Upper.ordering(1, 0), Ordering::Greater);
/// // For `Endpoint::Lower` `0` is greater than `1`.
/// assert!(!Endpoint::Lower.gt(1, 0));
/// assert_eq!(Endpoint::Lower.ordering(1, 0), Ordering::Less);
/// ```
///
/// Similar to [`Axis`](crate::axis::Axis), `Endpoint` is sometimes also used for variable access
/// and mutation, albeit not in a generic context (See [`EndpointPair`](super::EndpointPair) and
/// [`EndpointRange`](super::EndpointRange)).
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Endpoint {
    /// A lower of two directions, sides, bounds or endpoints.
    #[default]
    Lower = 0,
    /// An upper of two directions, sides, bounds or endpoints.
    Upper = 1,
}

impl Endpoint {
    /// Shorthand for [`Endpoint::Lower`].
    pub const LOW: Self = Self::Lower;
    /// Shorthand for [`Endpoint::Upper`].
    pub const UPP: Self = Self::Upper;

    /// The bit representation of [`Endpoint::Lower`].
    pub const LOW_REPR: u8 = Endpoint::LOW as u8;
    /// The bit representation of [`Endpoint::Upper`].
    pub const UPP_REPR: u8 = Endpoint::UPP as u8;

    /// A top-left two-dimensional endpoint representation.
    pub const TOP_LEFT: AxisVec<Endpoint> = AxisVec::new(Endpoint::LOW, Endpoint::LOW);
    /// A top-right two-dimensional endpoint representation.
    pub const TOP_RIGHT: AxisVec<Endpoint> = AxisVec::new(Endpoint::UPP, Endpoint::LOW);
    /// A bottom-left two-dimensional endpoint representation.
    pub const BOTTOM_LEFT: AxisVec<Endpoint> = AxisVec::new(Endpoint::LOW, Endpoint::UPP);
    /// A bottom-right two-dimensional endpoint representation.
    pub const BOTTOM_RIGHT: AxisVec<Endpoint> = AxisVec::new(Endpoint::UPP, Endpoint::UPP);

    /// Return an `Endpoint` from the given `u8` variable.
    pub const fn from_bits(value: u8) -> Self {
        match value {
            Self::LOW_REPR => Endpoint::LOW,
            Self::UPP_REPR => Endpoint::UPP,
            _ => unreachable!(),
        }
    }

    /// Return the `Endpoint` as a `u8`.
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    /// Flip the `Endpoint`.
    pub const fn invert(self) -> Self {
        match self {
            Endpoint::UPP => Endpoint::LOW,
            Endpoint::LOW => Endpoint::UPP,
        }
    }

    /// Returns `true` if `self` and `other` are equal.
    pub const fn const_eq(&self, other: u8) -> bool {
        *self as u8 == other
    }

    /// Returns `true` if the variant is `Endpoint::Upper`.
    #[inline]
    pub const fn is_upper(self) -> bool {
        self.into_bits() == Endpoint::UPP_REPR
    }

    /// Returns `true` if the variant is `Endpoint::Lower`.
    #[inline]
    pub const fn is_lower(self) -> bool {
        self.into_bits() == Endpoint::LOW_REPR
    }

    /// Return an `Endpoint` from the sign of an `f32` value. Returns `None` if `value == 0.0`.
    pub fn from_f32(value: f32) -> Option<Endpoint> {
        match value.total_cmp(&0.0) {
            Ordering::Less => Some(Endpoint::LOW),
            Ordering::Equal => None,
            Ordering::Greater => Some(Endpoint::UPP),
        }
    }

    /// Return an `Endpoint` from the sign of an `f32` value. **Panics** if `value == 0.0`.
    pub fn from_f32_unchecked(value: f32) -> Endpoint {
        match value.total_cmp(&0.0) {
            Ordering::Less => Endpoint::LOW,
            Ordering::Equal => unreachable!("Expected non-zero value"),
            Ordering::Greater => Endpoint::UPP,
        }
    }

    /// Return an `Endpoint` from the given [`Ordering`].
    pub const fn from_ordering(ord: Ordering) -> Option<Endpoint> {
        match ord {
            Ordering::Less => Some(Endpoint::LOW),
            Ordering::Equal => None,
            Ordering::Greater => Some(Endpoint::UPP),
        }
    }

    /// Return the `Endpoint` of `a` in relation to `b`. Returns `None` if both values are equal.
    pub fn from_cmp<T: Ord>(a: T, b: T) -> Option<Endpoint> {
        match a.cmp(&b) {
            Ordering::Less => Some(Endpoint::LOW),
            Ordering::Equal => None,
            Ordering::Greater => Some(Endpoint::UPP),
        }
    }

    /// Return the `Endpoint` of `a` in relation to `b`. Panics if both values are equal.
    pub fn from_cmp_unchecked<T: Ord>(a: T, b: T) -> Endpoint {
        match a.cmp(&b) {
            Ordering::Less => Endpoint::LOW,
            Ordering::Equal => unreachable!("Expected ordered values"),
            Ordering::Greater => Endpoint::UPP,
        }
    }

    /// Return the `Endpoint` of `a` in relation to `b`. Returns `None` if values are either equal
    /// or cannot be compared.
    pub fn from_partial_cmp<T: PartialOrd>(a: T, b: T) -> Option<Endpoint> {
        match a.partial_cmp(&b) {
            Some(Ordering::Less) => Some(Endpoint::LOW),
            Some(Ordering::Equal) | None => None,
            Some(Ordering::Greater) => Some(Endpoint::UPP),
        }
    }

    /// Return the `Endpoint` of `b` in relation to `a`. Panics if both values are equal or cannot
    /// be compared.
    pub fn from_partial_cmp_unchecked<T: PartialOrd>(a: T, b: T) -> Endpoint {
        match a.partial_cmp(&b) {
            Some(Ordering::Less) => Endpoint::LOW,
            Some(Ordering::Equal) | None => {
                unreachable!("Expected ordered values")
            }
            Some(Ordering::Greater) => Endpoint::UPP,
        }
    }

    /// Test if variable `a` if greater than `b` for the given `Endpoint` orientation. See
    /// [`Endpoint`] documentation for more on orientation.
    pub fn gt<T: Ord>(&self, a: T, b: T) -> bool {
        match self {
            Endpoint::Lower => a < b,
            Endpoint::Upper => a > b,
        }
    }

    /// Test if variable `a` if greater than or equal to `b` for the given `Endpoint` orientation.
    /// See [`Endpoint`] documentation for more on orientation.
    pub fn gte<T: Ord>(&self, a: T, b: T) -> bool {
        match self {
            Endpoint::Lower => a <= b,
            Endpoint::Upper => a >= b,
        }
    }

    /// Test if variable `a` if greater than `b` for the given `Endpoint` orientation. See
    /// [`Endpoint`] documentation for more on orientation.
    pub fn partial_gt<T: PartialOrd>(&self, a: T, b: &T) -> bool {
        match self {
            Endpoint::Lower => a.lt(b),
            Endpoint::Upper => a.gt(b),
        }
    }

    /// Test if variable `a` if greater than or equal to `b` for the given `Endpoint` orientation.
    /// See [`Endpoint`] documentation for more on orientation.
    pub fn partial_gte<T: PartialOrd>(&self, a: T, b: &T) -> bool {
        match self {
            Endpoint::Lower => a.le(b),
            Endpoint::Upper => a.ge(b),
        }
    }

    /// Returns the orientated [`Ordering`] of `a` in relation to `b`. See [`Endpoint`] documentation
    /// for more on orientation.
    pub fn ordering<T: Ord>(&self, a: T, b: T) -> Ordering {
        match self {
            Endpoint::Lower => b.cmp(&a),
            Endpoint::Upper => a.cmp(&b),
        }
    }

    /// Returns the orientated [`Ordering`] of `a` in relation to `b` using [`PartialOrd`]. See
    /// [`Endpoint`] documentation for more on orientation.
    pub fn partial_ordering<T: PartialOrd>(&self, a: T, b: T) -> Option<Ordering> {
        match self {
            Endpoint::Lower => b.partial_cmp(&a),
            Endpoint::Upper => a.partial_cmp(&b),
        }
    }

    /// Returns the orientated ordering of `a` in relation to `b` using [`f32::total_cmp`]. See
    /// [`Endpoint`] documentation for more on orientation.
    pub fn f32_total_ordering(&self, a: f32, b: f32) -> Ordering {
        match self {
            Endpoint::Lower => b.total_cmp(&a),
            Endpoint::Upper => a.total_cmp(&b),
        }
    }

    /// Returns the orientated ordering of `a` in relation to `b` using [`f64::total_cmp`]. See
    /// [`Endpoint`] documentation for more on orientation.
    pub fn f64_total_ordering(&self, a: f64, b: f64) -> Ordering {
        match self {
            Endpoint::Lower => b.total_cmp(&a),
            Endpoint::Upper => a.total_cmp(&b),
        }
    }

    /// Compares and returns the minimum of two values in relation to the `Endpoint` orientation.
    pub fn min<T: Ord>(&self, a: T, b: T) -> T {
        match self {
            Endpoint::Lower => a.max(b),
            Endpoint::Upper => a.min(b),
        }
    }

    /// Compares and returns the maximum of two values in relation to `Endpoint` orientation.
    #[inline]
    pub fn max<T: Ord>(&self, a: T, b: T) -> T {
        match self {
            Endpoint::Lower => a.min(b),
            Endpoint::Upper => a.max(b),
        }
    }

    /// Compares and returns the minimum of two values in relation to the `Endpoint` orientation
    /// using [`PartialOrd`]. See [`Endpoint`] documentation for more on orientation.
    pub fn partial_min<T: PartialOrd + Copy>(&self, a: T, b: T) -> T {
        if self.partial_gt(a, &b) { b } else { a }
    }

    /// Compares and returns the maximum of two values in relation to `Endpoint` orientation
    /// using [`PartialOrd`]. See [`Endpoint`] documentation for more on orientation.
    #[inline]
    pub fn partial_max<T: PartialOrd + Copy>(&self, a: T, b: T) -> T {
        self.invert().partial_min(a, b)
    }

    /// Increment the `endpoint` `value` by one resulting in a greater value (in relation to the
    /// `Endpoint` orientation). See [`Endpoint`] documentation for more on orientation.
    #[inline]
    pub fn incr<T: Add<T, Output = T> + Sub<T, Output = T> + ConstOne>(&self, value: T) -> T {
        match self {
            Endpoint::Lower => value - T::ONE,
            Endpoint::Upper => value + T::ONE,
        }
    }

    /// Increment the `Endpoint` `value` by `n` resulting in a greater value (in relation to the
    /// `Endpoint` orientation). See [`Endpoint`] documentation for more on orientation.
    #[inline]
    pub fn incr_n<T: Add<T, Output = T> + Sub<T, Output = T> + ConstOne>(
        &self,
        value: T,
        n: T,
    ) -> T {
        match self {
            Endpoint::Lower => value - n,
            Endpoint::Upper => value + n,
        }
    }

    /// Decrement the `Endpoint` `value` by one resulting in a lesser value (in relation to the
    /// `Endpoint` orientation). See [`Endpoint`] documentation for more on orientation.
    #[inline]
    pub fn decr<T: Add<T, Output = T> + Sub<T, Output = T> + ConstOne>(&self, value: T) -> T {
        match self {
            Endpoint::Lower => value + T::ONE,
            Endpoint::Upper => value - T::ONE,
        }
    }

    /// Decrement the `Endpoint` `value` by `n` resulting in a lesser value (in relation to the
    /// `Endpoint` orientation). See [`Endpoint`] documentation for more on orientation.
    #[inline]
    pub fn decr_n<T: Add<T, Output = T> + Sub<T, Output = T> + ConstOne>(
        &self,
        value: T,
        n: T,
    ) -> T {
        match self {
            Endpoint::Lower => value + n,
            Endpoint::Upper => value - n,
        }
    }
}

impl From<u8> for Endpoint {
    fn from(value: u8) -> Self {
        Self::from_bits(value)
    }
}

impl From<Endpoint> for u8 {
    fn from(value: Endpoint) -> Self {
        value as u8
    }
}

impl Not for Endpoint {
    type Output = Endpoint;

    fn not(self) -> Self::Output {
        self.invert()
    }
}
