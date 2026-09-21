use std::cmp::Ordering;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Add, Deref, DerefMut, Not, Sub};

use num_traits::ConstOne;

use crate::plane::axis::{Axis, AxisMask, AxisVec, AxisX, AxisY, DynAxis};

/// # Endpoint
///
/// Describes a direction on an axis.
///
/// `Endpoint` is the combination of a generic [`Axis`], and an [`EndpointBound`]. It provides axis
/// type enforcement and orientates numerical operations toward an endpoint. A large number of
/// `Endpoint` operations follow numerical ordering for the [`EndpointBound::Upper`], variants
/// but are reversed for [`EndpointBound::Lower`] variants.
///
/// # Example
///
/// ```
/// # use tilebound::plane::endpoint::Endpoint;
/// # use std::cmp::Ordering;
/// #
/// // For `Endpoint::RIGHT` `1` is greater than `0`.
/// assert!(Endpoint::RIGHT.gt(1, 0));
/// assert_eq!(Endpoint::RIGHT.ordering(1, 0), Ordering::Greater);
/// // For `Endpoint::LEFT` `0` is greater than `1`.
/// assert!(!Endpoint::LEFT.gt(1, 0));
/// assert_eq!(Endpoint::LEFT.ordering(1, 0), Ordering::Less);
/// ```
#[repr(transparent)]
#[derive(Eq, PartialOrd, Ord)]
pub struct Endpoint<A> {
    /// The endpoint axis.
    axis: PhantomData<A>,
    /// The endpoint bound.
    bound: EndpointBound,
}

pub type RawEndpoint = Endpoint<DynAxis>;

impl Endpoint<AxisX> {
    /// A left endpoint representation.
    pub const LEFT: Endpoint<AxisX> = Endpoint::LOWER;
    /// A right endpoint representation.
    pub const RIGHT: Endpoint<AxisX> = Endpoint::UPPER;
}

impl Endpoint<AxisY> {
    /// A top endpoint representation.
    pub const TOP: Endpoint<AxisY> = Endpoint::LOWER;
    /// A bottom endpoint representation.
    pub const BOTTOM: Endpoint<AxisY> = Endpoint::UPPER;
}

impl<A> Endpoint<A> {
    /// The [`EndpointBound::Lower`] variant for the given axis `A`.
    pub const LOWER: Self = Self {
        axis: PhantomData,
        bound: EndpointBound::Lower,
    };
    /// The [`EndpointBound::Upper`] variant for the given axis `A`.
    pub const UPPER: Self = Self {
        axis: PhantomData,
        bound: EndpointBound::Upper,
    };

    /// Return the inner [`EndpointBound`];
    pub const fn bound(self) -> EndpointBound {
        self.bound
    }

    /// Flip the [`EndpointBound`].
    pub const fn invert(self) -> Self {
        Endpoint {
            axis: PhantomData,
            bound: self.bound.invert(),
        }
    }

    /// Return the `Endpoint` as a [`AxisMask`].
    pub fn into_axismask(self) -> AxisMask
    where
        A: Axis,
    {
        AxisMask::from_end(self)
    }

    /// Cast the `Endpoint` axis marker to another type.
    pub const fn cast<B>(self) -> Endpoint<B> {
        Endpoint {
            axis: PhantomData,
            bound: self.bound,
        }
    }

    /// Return an `Endpoint` from the sign of an `f32` value. Returns `None` if `value == 0.0`.
    pub fn from_f32(value: f32) -> Option<Endpoint<A>> {
        match value.total_cmp(&0.0) {
            Ordering::Less => Some(Endpoint::LOWER),
            Ordering::Equal => None,
            Ordering::Greater => Some(Endpoint::UPPER),
        }
    }

    /// Return an `Endpoint` from the sign of an `f32` value. **Panics** if `value == 0.0`.
    pub fn from_f32_unchecked(value: f32) -> Endpoint<A> {
        match value.total_cmp(&0.0) {
            Ordering::Less => Endpoint::LOWER,
            Ordering::Equal => unreachable!("Expected non-zero value"),
            Ordering::Greater => Endpoint::UPPER,
        }
    }

    /// Return an `Endpoint` from the given [`Ordering`].
    pub const fn from_ordering(ord: Ordering) -> Option<Endpoint<A>> {
        match ord {
            Ordering::Less => Some(Endpoint::LOWER),
            Ordering::Equal => None,
            Ordering::Greater => Some(Endpoint::UPPER),
        }
    }

    /// Return the `Endpoint` of `a` in relation to `b`. Returns `None` if both values are equal.
    pub fn from_cmp<T: Ord>(a: T, b: T) -> Option<Endpoint<A>> {
        match a.cmp(&b) {
            Ordering::Less => Some(Endpoint::LOWER),
            Ordering::Equal => None,
            Ordering::Greater => Some(Endpoint::UPPER),
        }
    }

    /// Return the `Endpoint` of `a` in relation to `b`. Panics if both values are equal.
    pub fn from_cmp_unchecked<T: Ord>(a: T, b: T) -> Endpoint<A> {
        match a.cmp(&b) {
            Ordering::Less => Endpoint::LOWER,
            Ordering::Equal => unreachable!("Expected ordered values"),
            Ordering::Greater => Endpoint::UPPER,
        }
    }

    /// Return the `Endpoint` of `a` in relation to `b`. Returns `None` if values are either equal
    /// or cannot be compared.
    pub fn from_partial_cmp<T: PartialOrd>(a: T, b: T) -> Option<Endpoint<A>> {
        match a.partial_cmp(&b) {
            Some(Ordering::Less) => Some(Endpoint::LOWER),
            Some(Ordering::Equal) | None => None,
            Some(Ordering::Greater) => Some(Endpoint::UPPER),
        }
    }

    /// Return the `Endpoint` of `b` in relation to `a`. Panics if both values are equal or cannot
    /// be compared.
    pub fn from_partial_cmp_unchecked<T: PartialOrd>(a: T, b: T) -> Endpoint<A> {
        match a.partial_cmp(&b) {
            Some(Ordering::Less) => Endpoint::LOWER,
            Some(Ordering::Equal) | None => {
                unreachable!("Expected ordered values")
            }
            Some(Ordering::Greater) => Endpoint::UPPER,
        }
    }
}

impl<A> Debug for Endpoint<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Endpoint")
            .field("axis", &self.axis)
            .field("bound", &self.bound)
            .finish()
    }
}

impl<A> Default for Endpoint<A> {
    fn default() -> Self {
        Self {
            axis: Default::default(),
            bound: Default::default(),
        }
    }
}

impl<A> Clone for Endpoint<A> {
    fn clone(&self) -> Self {
        Self {
            axis: self.axis.clone(),
            bound: self.bound.clone(),
        }
    }
}

impl<A> Copy for Endpoint<A> {}

impl<A> Deref for Endpoint<A> {
    type Target = EndpointBound;

    fn deref(&self) -> &Self::Target {
        &self.bound
    }
}

impl<A> DerefMut for Endpoint<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bound
    }
}

impl<A> PartialEq for Endpoint<A> {
    fn eq(&self, other: &Self) -> bool {
        self.axis == other.axis && self.bound == other.bound
    }
}

impl<A> From<EndpointBound> for Endpoint<A> {
    fn from(bound: EndpointBound) -> Self {
        Endpoint {
            axis: PhantomData,
            bound,
        }
    }
}

impl<A> Not for Endpoint<A> {
    type Output = Endpoint<A>;

    fn not(self) -> Self::Output {
        self.invert()
    }
}

/// # EndpointBound
///
/// Differentiates between a lower and upper endpoint on an axis.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EndpointBound {
    /// A lower or negative of two directions, sides, bounds or endpoints.
    #[default]
    Lower = 0,
    /// An upper or positive of two directions, sides, bounds or endpoints.
    Upper = 1,
}

impl EndpointBound {
    /// The bit representation of [`EndpointBound::Lower`].
    pub const LOW_REPR: u8 = EndpointBound::Lower as u8;
    /// The bit representation of [`EndpointBound::Upper`].
    pub const UPP_REPR: u8 = EndpointBound::Upper as u8;

    /// A top-left two-dimensional endpoint bound representation.
    pub const TOP_LEFT: AxisVec<EndpointBound> =
        AxisVec::new(EndpointBound::Lower, EndpointBound::Lower);
    /// A top-right two-dimensional endpoint bound representation.
    pub const TOP_RIGHT: AxisVec<EndpointBound> =
        AxisVec::new(EndpointBound::Upper, EndpointBound::Lower);
    /// A bottom-left two-dimensional endpoint bound representation.
    pub const BOTTOM_LEFT: AxisVec<EndpointBound> =
        AxisVec::new(EndpointBound::Lower, EndpointBound::Upper);
    /// A bottom-right two-dimensional endpoint bound representation.
    pub const BOTTOM_RIGHT: AxisVec<EndpointBound> =
        AxisVec::new(EndpointBound::Upper, EndpointBound::Upper);

    /// Return the `EndpointBound` as a `u8`.
    pub const fn into_bits(self) -> u8 {
        self as u8
    }

    /// Flip the `EndpointBound`.
    pub const fn invert(self) -> Self {
        match self {
            EndpointBound::Upper => EndpointBound::Lower,
            EndpointBound::Lower => EndpointBound::Upper,
        }
    }

    /// Returns `true` if the variant is `EndpointBound::Lower`.
    #[inline]
    pub const fn is_lower(self) -> bool {
        self.into_bits() == EndpointBound::LOW_REPR
    }

    /// Returns `true` if the variant is `EndpointBound::Upper`.
    #[inline]
    pub const fn is_upper(self) -> bool {
        self.into_bits() == EndpointBound::UPP_REPR
    }

    /// Test if variable `a` if greater than `b` for the given `EndpointBound` orientation.
    pub fn gt<T: Ord>(&self, a: T, b: T) -> bool {
        match self {
            EndpointBound::Lower => a < b,
            EndpointBound::Upper => a > b,
        }
    }

    /// Test if variable `a` if greater than or equal to `b` for the given `EndpointBound` orientation.
    pub fn gte<T: Ord>(&self, a: T, b: T) -> bool {
        match self {
            EndpointBound::Lower => a <= b,
            EndpointBound::Upper => a >= b,
        }
    }

    /// Test if variable `a` if greater than `b` for the given `EndpointBound` orientation.
    pub fn partial_gt<T: PartialOrd>(&self, a: T, b: &T) -> bool {
        match self {
            EndpointBound::Lower => a.lt(b),
            EndpointBound::Upper => a.gt(b),
        }
    }

    /// Test if variable `a` if greater than or equal to `b` for the given `EndpointBound` orientation.
    pub fn partial_gte<T: PartialOrd>(&self, a: T, b: &T) -> bool {
        match self {
            EndpointBound::Lower => a.le(b),
            EndpointBound::Upper => a.ge(b),
        }
    }

    /// Returns the orientated [`Ordering`] of `a` in relation to `b`.
    pub fn ordering<T: Ord>(&self, a: T, b: T) -> Ordering {
        match self {
            EndpointBound::Lower => b.cmp(&a),
            EndpointBound::Upper => a.cmp(&b),
        }
    }

    /// Returns the orientated [`Ordering`] of `a` in relation to `b` using [`PartialOrd`].
    pub fn partial_ordering<T: PartialOrd>(&self, a: T, b: T) -> Option<Ordering> {
        match self {
            EndpointBound::Lower => b.partial_cmp(&a),
            EndpointBound::Upper => a.partial_cmp(&b),
        }
    }

    /// Returns the orientated ordering of `a` in relation to `b` using [`f32::total_cmp`].
    pub fn f32_total_ordering(&self, a: f32, b: f32) -> Ordering {
        match self {
            EndpointBound::Lower => b.total_cmp(&a),
            EndpointBound::Upper => a.total_cmp(&b),
        }
    }

    /// Returns the orientated ordering of `a` in relation to `b` using [`f64::total_cmp`].
    pub fn f64_total_ordering(&self, a: f64, b: f64) -> Ordering {
        match self {
            EndpointBound::Lower => b.total_cmp(&a),
            EndpointBound::Upper => a.total_cmp(&b),
        }
    }

    /// Compares and returns the minimum of two values in relation to the `EndpointBound` orientation.
    pub fn min<T: Ord>(&self, a: T, b: T) -> T {
        match self {
            EndpointBound::Lower => a.max(b),
            EndpointBound::Upper => a.min(b),
        }
    }

    /// Compares and returns the maximum of two values in relation to `EndpointBound` orientation.
    #[inline]
    pub fn max<T: Ord>(&self, a: T, b: T) -> T {
        match self {
            EndpointBound::Lower => a.min(b),
            EndpointBound::Upper => a.max(b),
        }
    }

    /// Compares and returns the minimum of two values in relation to the `EndpointBound` orientation
    /// using [`PartialOrd`].
    pub fn partial_min<T: PartialOrd + Copy>(&self, a: T, b: T) -> T {
        if self.partial_gt(a, &b) { b } else { a }
    }

    /// Compares and returns the maximum of two values in relation to `EndpointBound` orientation
    /// using [`PartialOrd`].
    #[inline]
    pub fn partial_max<T: PartialOrd + Copy>(&self, a: T, b: T) -> T {
        self.invert().partial_min(a, b)
    }

    /// Increment the `endpoint` `value` by one resulting in a greater value (in relation to the
    /// `EndpointBound` orientation).
    #[inline]
    pub fn incr<T: Add<T, Output = T> + Sub<T, Output = T> + ConstOne>(&self, value: T) -> T {
        match self {
            EndpointBound::Lower => value - T::ONE,
            EndpointBound::Upper => value + T::ONE,
        }
    }

    /// Increment the `EndpointBound` `value` by `n` resulting in a greater value (in relation to the
    /// `EndpointBound` orientation).
    #[inline]
    pub fn incr_n<T: Add<T, Output = T> + Sub<T, Output = T> + ConstOne>(
        &self,
        value: T,
        n: T,
    ) -> T {
        match self {
            EndpointBound::Lower => value - n,
            EndpointBound::Upper => value + n,
        }
    }

    /// Decrement the `EndpointBound` `value` by one resulting in a lesser value (in relation to the
    /// `EndpointBound` orientation).
    #[inline]
    pub fn decr<T: Add<T, Output = T> + Sub<T, Output = T> + ConstOne>(&self, value: T) -> T {
        match self {
            EndpointBound::Lower => value + T::ONE,
            EndpointBound::Upper => value - T::ONE,
        }
    }

    /// Decrement the `EndpointBound` `value` by `n` resulting in a lesser value (in relation to the
    /// `EndpointBound` orientation).
    #[inline]
    pub fn decr_n<T: Add<T, Output = T> + Sub<T, Output = T> + ConstOne>(
        &self,
        value: T,
        n: T,
    ) -> T {
        match self {
            EndpointBound::Lower => value + n,
            EndpointBound::Upper => value - n,
        }
    }
}

impl Not for EndpointBound {
    type Output = EndpointBound;

    fn not(self) -> Self::Output {
        self.invert()
    }
}
