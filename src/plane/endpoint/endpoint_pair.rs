use std::{marker::PhantomData, ops::Add};

use crate::plane::endpoint::{Endpoint, EndpointBound};

/// # EndpointPair
///
/// `EndpointPair` are two variables associated with two opposing [`Endpoint`]s. See also
/// [`EndpointRange`](super::EndpointRange).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct EndpointPair<A, T> {
    pub axis: PhantomData<A>,
    pub lower: T,
    pub upper: T,
}

impl<A, T> EndpointPair<A, T> {
    /// Create `EndpointPair` from the given `lower` and `upper` variables.
    #[inline]
    pub fn new(lower: T, upper: T) -> Self {
        Self {
            axis: PhantomData,
            lower,
            upper,
        }
    }

    /// Create a new `EndpointPair` with element *i* and *j* passed to the given `endpoint` and its
    /// inverse respectively.
    pub fn new_mapped(a: T, b: T, end: Endpoint<A>) -> Self {
        match *end {
            EndpointBound::Lower => EndpointPair::<A, T>::new(a, b),
            EndpointBound::Upper => EndpointPair::<A, T>::new(b, a),
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
    pub fn get(&self, end: Endpoint<A>) -> T
    where
        T: Copy,
    {
        match *end {
            EndpointBound::Lower => self.lower,
            EndpointBound::Upper => self.upper,
        }
    }

    /// Get a reference to the value assigned to the given `end`.
    pub fn get_ref(&self, end: Endpoint<A>) -> &T {
        match *end {
            EndpointBound::Lower => &self.lower,
            EndpointBound::Upper => &self.upper,
        }
    }

    /// Get a mutable reference to the value assigned to the given `end`.
    pub fn get_mut(&mut self, end: Endpoint<A>) -> &mut T {
        match *end {
            EndpointBound::Lower => &mut self.lower,
            EndpointBound::Upper => &mut self.upper,
        }
    }

    /// Return an `EndpointPair` with the result of the function `F` applied to each
    /// element.
    pub fn map<F: FnMut(T) -> U, U>(self, mut f: F) -> EndpointPair<A, U> {
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

impl<A, T> From<(T, T)> for EndpointPair<A, T> {
    fn from((lower, upper): (T, T)) -> Self {
        EndpointPair {
            axis: PhantomData,
            lower,
            upper,
        }
    }
}

impl<A, T> From<[T; 2]> for EndpointPair<A, T> {
    fn from([lower, upper]: [T; 2]) -> Self {
        EndpointPair {
            axis: PhantomData,
            lower,
            upper,
        }
    }
}

impl<A, T: Copy + Add<T, Output = T>> EndpointPair<A, T> {
    /// Return the sum of the lower and upper variables.
    pub fn sum(&self) -> T {
        self.lower + self.upper
    }
}
