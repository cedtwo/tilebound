use std::cmp::Ordering;
use std::ops::*;

use num_traits::ConstOne;

use super::*;

/// # EndpointRange
///
/// [`Endpoint`] mapping and comparison operations for ranges.
pub trait EndpointRange<T> {
    /// Create an endpoint range from `lower` inclusive and `upper` exclusive elements, mapping
    /// the first and second element to the given [`Endpoint`] and its transpose respectively.
    fn new_mapped(a: T, b: T, end: Endpoint) -> Self;

    /// Create a new endpoint range from `lower` inclusive and `upper` inclusive elements, mapping
    /// the first and second element to the given [`Endpoint`] and its transpose respectively.
    fn new_mapped_incl(a: T, b: T, sign: Endpoint) -> Self;

    /// Get a reference to the [`Endpoint::Lower`] value.
    fn lower(&self) -> &T;

    /// Get a reference to the [`Endpoint::Upper`] value.
    fn upper(&self) -> &T;

    /// Increment the given [`Endpoint`] by one (See [`Endpoint`] for more on orientation).
    fn incr(&mut self, sign: Endpoint);

    /// Decrement the given [`Endpoint`] by one (See [`Endpoint`] for more on orientation).
    fn decr(&mut self, sign: Endpoint);

    /// Get the [`Endpoint`] of any single value not equal to `other`. Returns `None` where both
    /// value are equal, or both values differ.
    fn endpoint_diff(&self, other: &Self) -> Option<Endpoint>;

    /// Returns `true` if any part of `self` intersects that of `other`.
    fn intersects(&self, other: &Self) -> bool;

    /// Get the [`Endpoint`] of `other` greater than that of `self`. Returns `None` if no single
    /// endpoint is greater than `self` (See [`Endpoint`] for more on orientation).
    fn greater_endpoint(&self, other: &Self) -> Option<Endpoint>;

    /// Get the first [`Endpoint`] of `other` greater than that of `self`. Returns `None` if no
    /// single endpoint is greater than `self` (See [`Endpoint`] for more on orientation).
    fn first_greater_endpoint(&self, other: &Self) -> Option<Endpoint>;

    /// Get the [`Endpoint`] of `other` lesser than that of `self`. Returns `None` if no single
    /// endpoint is lesser than `self` (See [`Endpoint`] for more on orientation).
    fn lesser_endpoint(&self, other: &Self) -> Option<Endpoint>;

    /// Get the first [`Endpoint`] of `other` lesser than that of `self`. Returns `None` if no single
    /// endpoint is lesser than `self` (See [`Endpoint`] for more on orientation).
    fn first_lesser_endpoint(&self, other: &Self) -> Option<Endpoint>;
}

impl<T> EndpointRange<T> for std::ops::Range<T>
where
    T: PartialEq + PartialOrd + Add<T, Output = T> + AddAssign<T> + SubAssign<T> + ConstOne,
{
    fn new_mapped(a: T, b: T, end: Endpoint) -> Self {
        match end {
            Endpoint::LOW => a..b,
            Endpoint::UPP => b..a,
        }
    }

    fn new_mapped_incl(a: T, b: T, sign: Endpoint) -> Self {
        match sign {
            Endpoint::LOW => a..b + T::ONE,
            Endpoint::UPP => b..a + T::ONE,
        }
    }

    fn lower(&self) -> &T {
        &self.start
    }

    fn upper(&self) -> &T {
        &self.end
    }

    fn incr(&mut self, sign: Endpoint) {
        match sign {
            Endpoint::LOW => self.start -= T::ONE,
            Endpoint::UPP => self.end += T::ONE,
        }
    }

    fn decr(&mut self, sign: Endpoint) {
        match sign {
            Endpoint::LOW => self.start -= T::ONE,
            Endpoint::UPP => self.end += T::ONE,
        }
    }

    fn intersects(&self, other: &Self) -> bool {
        self.start < other.end && self.end > other.start
    }

    fn endpoint_diff(&self, other: &Self) -> Option<Endpoint> {
        match (self.start == other.start, self.end == other.end) {
            (true, true) => None,
            (true, false) => Some(Endpoint::LOW),
            (false, true) => Some(Endpoint::UPP),
            (false, false) => None,
        }
    }

    fn greater_endpoint(&self, other: &Self) -> Option<Endpoint> {
        match (
            self.start.partial_cmp(&other.start),
            self.end.partial_cmp(&other.end),
        ) {
            (Some(Ordering::Less), Some(Ordering::Equal) | Some(Ordering::Less)) => {
                Some(Endpoint::LOW)
            }
            (Some(Ordering::Equal) | Some(Ordering::Greater), Some(Ordering::Greater)) => {
                Some(Endpoint::UPP)
            }
            _ => None,
        }
    }

    fn first_greater_endpoint(&self, other: &Self) -> Option<Endpoint> {
        if self.start > other.start {
            Some(Endpoint::LOW)
        } else if self.end < other.end {
            Some(Endpoint::UPP)
        } else {
            None
        }
    }

    fn lesser_endpoint(&self, other: &Self) -> Option<Endpoint> {
        match (
            self.start.partial_cmp(&other.start),
            self.end.partial_cmp(&other.end),
        ) {
            (Some(Ordering::Greater), Some(Ordering::Equal) | Some(Ordering::Greater)) => {
                Some(Endpoint::LOW)
            }
            (Some(Ordering::Less) | Some(Ordering::Equal), Some(Ordering::Less)) => {
                Some(Endpoint::UPP)
            }
            _ => None,
        }
    }

    fn first_lesser_endpoint(&self, other: &Self) -> Option<Endpoint> {
        if self.start < other.start {
            Some(Endpoint::LOW)
        } else if self.end > other.end {
            Some(Endpoint::UPP)
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
