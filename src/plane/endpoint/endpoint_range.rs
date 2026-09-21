use std::cmp::Ordering;
use std::ops::*;

use num_traits::ConstOne;

use crate::plane::axis::Axis;

use super::*;

/// # EndpointRange
///
/// [`Endpoint`] mapping and comparison operations for ranges.
pub trait EndpointRange<T> {
    /// Create an endpoint range from `lower` inclusive and `upper` exclusive elements, mapping
    /// the first and second element to the given [`Endpoint`] and its transpose respectively.
    fn new_mapped<A: Axis>(a: T, b: T, end: Endpoint<A>) -> Self;

    /// Create a new endpoint range from `lower` inclusive and `upper` inclusive elements, mapping
    /// the first and second element to the given [`Endpoint<A>`] and its transpose respectively.
    fn new_mapped_incl<A: Axis>(a: T, b: T, end: Endpoint<A>) -> Self;

    /// Get a reference to the [`Endpoint<A>::Lower`] value.
    fn lower(&self) -> &T;

    /// Get a reference to the [`Endpoint<A>::Upper`] value.
    fn upper(&self) -> &T;

    /// Increment the given [`Endpoint<A>`] by one (See [`Endpoint<A>`] for more on orientation).
    fn incr<A: Axis>(&mut self, end: Endpoint<A>);

    /// Decrement the given [`Endpoint<A>`] by one (See [`Endpoint<A>`] for more on orientation).
    fn decr<A: Axis>(&mut self, end: Endpoint<A>);

    /// Get the [`Endpoint<A>`] of any single value not equal to `other`. Returns `None` where both
    /// value are equal, or both values differ.
    fn endpoint_diff<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>>;

    /// Returns `true` if any part of `self` intersects that of `other`.
    fn intersects(&self, other: &Self) -> bool;

    /// Get the [`Endpoint<A>`] of `other` greater than that of `self`. Returns `None` if no single
    /// endpoint is greater than `self` (See [`Endpoint<A>`] for more on orientation).
    fn greater_endpoint<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>>;

    /// Get the first [`Endpoint<A>`] of `other` greater than that of `self`. Returns `None` if no
    /// single endpoint is greater than `self` (See [`Endpoint<A>`] for more on orientation).
    fn first_greater_endpoint<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>>;

    /// Get the [`Endpoint<A>`] of `other` lesser than that of `self`. Returns `None` if no single
    /// endpoint is lesser than `self` (See [`Endpoint<A>`] for more on orientation).
    fn lesser_endpoint<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>>;

    /// Get the first [`Endpoint<A>`] of `other` lesser than that of `self`. Returns `None` if no single
    /// endpoint is lesser than `self` (See [`Endpoint<A>`] for more on orientation).
    fn first_lesser_endpoint<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>>;
}

impl<T> EndpointRange<T> for std::ops::Range<T>
where
    T: PartialEq + PartialOrd + Add<T, Output = T> + AddAssign<T> + SubAssign<T> + ConstOne,
{
    fn new_mapped<A: Axis>(a: T, b: T, end: Endpoint<A>) -> Self {
        match *end {
            EndpointBound::Lower => a..b,
            EndpointBound::Upper => b..a,
        }
    }

    fn new_mapped_incl<A: Axis>(a: T, b: T, end: Endpoint<A>) -> Self {
        match *end {
            EndpointBound::Lower => a..b + T::ONE,
            EndpointBound::Upper => b..a + T::ONE,
        }
    }

    fn lower(&self) -> &T {
        &self.start
    }

    fn upper(&self) -> &T {
        &self.end
    }

    fn incr<A: Axis>(&mut self, end: Endpoint<A>) {
        match *end {
            EndpointBound::Lower => self.start -= T::ONE,
            EndpointBound::Upper => self.end += T::ONE,
        }
    }

    fn decr<A: Axis>(&mut self, end: Endpoint<A>) {
        match *end {
            EndpointBound::Lower => self.start -= T::ONE,
            EndpointBound::Upper => self.end += T::ONE,
        }
    }

    fn intersects(&self, other: &Self) -> bool {
        self.start < other.end && self.end > other.start
    }

    fn endpoint_diff<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>> {
        match (self.start == other.start, self.end == other.end) {
            (true, true) => None,
            (true, false) => Some(Endpoint::LOWER),
            (false, true) => Some(Endpoint::UPPER),
            (false, false) => None,
        }
    }

    fn greater_endpoint<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>> {
        match (
            self.start.partial_cmp(&other.start),
            self.end.partial_cmp(&other.end),
        ) {
            (Some(Ordering::Less), Some(Ordering::Equal) | Some(Ordering::Less)) => {
                Some(Endpoint::LOWER)
            }
            (Some(Ordering::Equal) | Some(Ordering::Greater), Some(Ordering::Greater)) => {
                Some(Endpoint::UPPER)
            }
            _ => None,
        }
    }

    fn first_greater_endpoint<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>> {
        if self.start > other.start {
            Some(Endpoint::LOWER)
        } else if self.end < other.end {
            Some(Endpoint::UPPER)
        } else {
            None
        }
    }

    fn lesser_endpoint<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>> {
        match (
            self.start.partial_cmp(&other.start),
            self.end.partial_cmp(&other.end),
        ) {
            (Some(Ordering::Greater), Some(Ordering::Equal) | Some(Ordering::Greater)) => {
                Some(Endpoint::LOWER)
            }
            (Some(Ordering::Less) | Some(Ordering::Equal), Some(Ordering::Less)) => {
                Some(Endpoint::UPPER)
            }
            _ => None,
        }
    }

    fn first_lesser_endpoint<A: Axis>(&self, other: &Self) -> Option<Endpoint<A>> {
        if self.start < other.start {
            Some(Endpoint::LOWER)
        } else if self.end > other.end {
            Some(Endpoint::UPPER)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn intersects() {
        let a = -4..-3;
        let b = -3..-2;
        let c = -3..3;
        let d = 2..4;
        let e = 4..5;

        assert!(!a.intersects(&b));
        assert!(b.intersects(&c));
        assert!(c.intersects(&d));
        assert!(!d.intersects(&e));

        assert_eq!(a.intersects(&b), b.intersects(&a));
        assert_eq!(a.intersects(&c), c.intersects(&a));
        assert_eq!(a.intersects(&d), d.intersects(&a));
        assert_eq!(b.intersects(&c), c.intersects(&b));
        assert_eq!(b.intersects(&d), d.intersects(&b));
        assert_eq!(c.intersects(&d), d.intersects(&c));
    }
}
