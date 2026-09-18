use std::ops::Add;

use crate::plane::endpoint::Endpoint;

/// # EndpointPair
///
/// `EndpointPair` are two variables associated with two opposing [`Endpoint`]s. See also
/// [`EndpointRange`](super::EndpointRange).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndpointPair<T> {
    pub lower: T,
    pub upper: T,
}

impl<T> EndpointPair<T> {
    /// Create `EndpointPair` from the given `lower` and `upper` variables.
    #[inline]
    pub fn new(lower: T, upper: T) -> Self {
        Self { lower, upper }
    }

    /// Create a new `EndpointPair` with element *i* and *j* passed to the given `endpoint` and its
    /// inverse respectively.
    pub fn new_mapped(a: T, b: T, end: Endpoint) -> Self {
        match end {
            Endpoint::LOW => EndpointPair::new(a, b),
            Endpoint::UPP => EndpointPair::new(b, a),
        }
    }

    /// Get the lower value.
    pub const fn lower(&self) -> T
    where
        T: Copy,
    {
        self.lower
    }

    /// Get the upper value.
    pub const fn upper(&self) -> T
    where
        T: Copy,
    {
        self.upper
    }

    /// Get the value assigned to the given `end`.
    pub const fn get(&self, end: Endpoint) -> T
    where
        T: Copy,
    {
        match end {
            Endpoint::LOW => self.lower,
            Endpoint::UPP => self.upper,
        }
    }

    /// Get a reference to the value assigned to the given `end`.
    pub const fn get_ref(&self, end: Endpoint) -> &T {
        match end {
            Endpoint::LOW => &self.lower,
            Endpoint::UPP => &self.upper,
        }
    }

    /// Get a mutable reference to the value assigned to the given `end`.
    pub const fn get_mut(&mut self, end: Endpoint) -> &mut T {
        match end {
            Endpoint::LOW => &mut self.lower,
            Endpoint::UPP => &mut self.upper,
        }
    }

    /// Return an `EndpointPair` with the result of the function `F` applied to each
    /// element.
    pub fn map<F: FnMut(T) -> U, U>(self, mut f: F) -> EndpointPair<U> {
        EndpointPair::new(f(self.lower), f(self.upper))
    }

    /// Returns `true` if any element satisfies the predicate `f`.
    #[inline]
    pub fn any<F: Fn(&T) -> bool>(self, f: F) -> bool {
        f(&self.lower) || f(&self.upper)
    }

    /// Returns `true` if both elements satisfy the predicate `f`.
    #[inline]
    pub fn all<F: Fn(&T) -> bool>(self, f: F) -> bool {
        f(&self.lower) && f(&self.upper)
    }
}

impl<T> From<(T, T)> for EndpointPair<T> {
    fn from((lower, upper): (T, T)) -> Self {
        EndpointPair { lower, upper }
    }
}

impl<T> From<[T; 2]> for EndpointPair<T> {
    fn from([lower, upper]: [T; 2]) -> Self {
        EndpointPair { lower, upper }
    }
}

impl<T: Copy + Add<T, Output = T>> EndpointPair<T> {
    /// Return the sum of the lower and upper variables.
    pub fn sum(&self) -> T {
        self.lower + self.upper
    }
}
