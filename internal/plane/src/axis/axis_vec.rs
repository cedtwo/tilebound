use std::fmt::Debug;
use std::iter::{Product, Sum};
use std::ops::{Add, Mul, Sub};

use marker_value::MarkerValue;
use num_traits::{ConstZero, Num};

use crate::axis::*;

/// # AxisVec
///
/// A two-dimensional vector of values `T` mapped to an axis index. Provides operations for element
/// access and mutation by either generic or dynamic axis ([`Axis`] or [`DynAxis`] respectively).
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub struct AxisVec<T>([T; 2]);

impl<T> AxisVec<T> {
    /// Create a new `AxisVec` of the given variables.
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Self([x, y])
    }

    /// Create a new `AxisVec` setting both variables to the given `value`.
    #[inline]
    pub const fn fill(value: T) -> Self
    where
        T: Copy,
    {
        Self([value, value])
    }

    /// Get a copy of the value assigned to axis *x*.
    pub const fn x(&self) -> T
    where
        T: Copy,
    {
        self.0[0]
    }

    /// Get a copy of the value assigned to axis *y*.
    pub const fn y(&self) -> T
    where
        T: Copy,
    {
        self.0[1]
    }

    /// Get a reference to the value assigned to axis *x*.
    pub const fn x_ref(&self) -> &T {
        &self.0[0]
    }

    /// Get a reference to the value assigned to axis *y*.
    pub const fn y_ref(&self) -> &T {
        &self.0[1]
    }

    /// Get a mutable reference to the value assigned to axis *x*.
    pub const fn x_mut(&mut self) -> &mut T {
        &mut self.0[0]
    }

    /// Get a mutable reference to the value assigned to axis *y*.
    pub const fn y_mut(&mut self) -> &mut T {
        &mut self.0[1]
    }

    /// Consume the `AxisVec`, returning the inner array.
    pub fn take(self) -> [T; 2] {
        self.0
    }

    /// Return the `AxisVec` as a tuple of the *x* and *y* variables respectively.
    pub const fn split(&self) -> (T, T)
    where
        T: Copy,
    {
        (self.x(), self.y())
    }

    /// Get a reference to the inner array.
    pub const fn as_array(&self) -> &[T; 2] {
        &self.0
    }

    /// Return an `AxisVec` with the result of the function `F` applied to each element.
    pub fn map<F: FnMut(T) -> U, U>(self, mut f: F) -> AxisVec<U> {
        AxisVec(self.0.map(|el| f(el)))
    }

    /// Return an `AxisVec` with the result of the function `F` applied to the elements of `self`
    /// and other.
    pub fn map_with<F: FnMut(T, U) -> V, U, V>(self, other: AxisVec<U>, mut f: F) -> AxisVec<V>
    where
        T: Copy,
        U: Copy,
    {
        AxisVec::new(f(self.x(), other.x()), f(self.y(), other.y()))
    }

    /// Calling [`Into::into`] on the inner variables, returning `AxisVec<U>`.
    pub fn map_into<U>(self) -> AxisVec<U>
    where
        T: Into<U>,
    {
        AxisVec(self.0.map(|el| el.into()))
    }

    /// Calling [`TryInto::try_into`] on the inner variables, returning `Result<AxisVec>`.
    pub fn try_map_into<U>(self) -> Result<AxisVec<U>, T::Error>
    where
        T: TryInto<U, Error: Debug>,
    {
        AxisVec(self.0.map(|el| el.try_into())).flatten()
    }

    /// Apply an operation on the inner array, returning the result.
    pub fn flat_map<F: FnMut([T; 2]) -> U, U>(self, mut f: F) -> U {
        f(self.0)
    }

    /// Apply a fallible operation to the inner array, returning the result.
    pub fn try_flat_map<F: FnMut([T; 2]) -> Option<U>, U>(self, mut f: F) -> Option<U> {
        f(self.0)
    }

    /// Returns `true` if any element satisfies the predicate `f`.
    #[inline]
    pub fn any<F: Fn(&T) -> bool>(self, f: F) -> bool {
        self.0.iter().any(|el| f(el))
    }

    /// Returns `true` if all elements satisfy the predicate `f`.
    #[inline]
    pub fn all<F: Fn(&T) -> bool>(self, f: F) -> bool {
        self.0.iter().all(|el| f(el))
    }
}

impl<T> AxisVec<T> {
    /// Create a new `AxisVec` with element *i* and *j* passed to the given `const` axis `A` and
    /// its transpose respectively.
    pub fn new_mapped<A: Axis>(i: T, j: T) -> Self {
        match A::VALUE {
            AxisX::VALUE => AxisVec::new(i, j),
            AxisY::VALUE => AxisVec::new(j, i),
            _ => unreachable!(),
        }
    }

    /// Get a copy of the value assigned to the `const` axis `A`.
    pub fn get<A: Axis>(&self) -> T
    where
        T: Copy,
    {
        self.0[A::VALUE]
    }

    /// Get a reference to the value assigned to axis `A`.
    pub fn get_ref<A: Axis>(&self) -> &T {
        &self.0[A::VALUE]
    }

    /// Get a mutable reference to the value assigned to axis `A`.
    pub fn get_mut<A: Axis>(&mut self) -> &mut T {
        &mut self.0[A::VALUE]
    }

    /// Get a clone of the value assigned to axis `A`.
    pub fn get_cloned<A: Axis>(&self) -> T
    where
        T: Clone,
    {
        self.0[A::VALUE].clone()
    }

    /// Set default the value assigned to the axis `A`.
    pub fn clear<A: Axis>(&mut self)
    where
        T: Default,
    {
        *self.get_mut::<A>() = T::default();
    }

    /// Return an `AxisVec` with the result of the function `F` applied the const axis `A`.
    pub fn map_axis<A: Axis, F: FnMut(T) -> T>(&self, mut f: F) -> Self
    where
        T: Copy,
    {
        match A::VALUE {
            AxisX::VALUE => AxisVec::new(f(self.0[0]), self.0[1]),
            AxisY::VALUE => AxisVec::new(self.0[0], f(self.0[1])),
            _ => unreachable!(),
        }
    }

    /// Split variables into a `(T, T)` tuple with the axis `A` variable first.
    pub fn split_axis<A: Axis>(self) -> (T, T)
    where
        T: Copy,
    {
        match A::VALUE {
            AxisX::VALUE => (self.x(), self.y()),
            AxisY::VALUE => (self.y(), self.x()),
            _ => unreachable!(),
        }
    }
}

impl<T> AxisVec<T> {
    /// Create a new `AxisVec` with element *i* and *j* passed to the given dynamic axis `a` and
    /// its transpose respectively.
    pub fn new_dyn_mapped(a: DynAxis, i: T, j: T) -> Self {
        match a as usize {
            AxisX::VALUE => AxisVec::new(i, j),
            AxisY::VALUE => AxisVec::new(j, i),
            _ => unreachable!(),
        }
    }

    /// Get a copy of the value assigned to the dynamic axis `a`.
    pub fn get_dyn(&self, a: DynAxis) -> T
    where
        T: Copy,
    {
        self.0[a as usize]
    }

    /// Get a reference to the value assigned to the given dynamic axis `a`.
    pub fn get_dyn_ref(&self, a: DynAxis) -> &T {
        &self.0[a as usize]
    }

    /// Get a mutable reference to the value assigned to the given dynamic axis `a`.
    pub fn get_dyn_mut(&mut self, a: DynAxis) -> &mut T {
        &mut self.0[a as usize]
    }

    /// Get a clone of the value assigned to the given dynamic axis `a`.
    pub fn get_dyn_cloned(&self, a: DynAxis) -> T
    where
        T: Clone,
    {
        self.0[a as usize].clone()
    }

    /// Set default the value assigned to the given dynamic axis `a`.
    pub fn clear_dyn(&mut self, a: DynAxis)
    where
        T: Default,
    {
        *self.get_dyn_mut(a) = T::default();
    }

    /// Return an `AxisVec` with the result of the function `F` applied the dynamic axis `a`.
    pub fn map_dyn_axis<F: FnMut(T) -> T>(&self, a: DynAxis, mut f: F) -> Self
    where
        T: Copy,
    {
        match a as usize {
            AxisX::VALUE => AxisVec::new(f(self.0[0]), self.0[1]),
            AxisY::VALUE => AxisVec::new(self.0[0], f(self.0[1])),
            _ => unreachable!(),
        }
    }

    /// Split variables into a `(T, T)` tuple with the dynamic axis `a` variable first.
    pub fn split_dyn_axis(self, a: DynAxis) -> (T, T)
    where
        T: Copy,
    {
        match a as usize {
            AxisX::VALUE => (self.x(), self.y()),
            AxisY::VALUE => (self.y(), self.x()),
            _ => unreachable!(),
        }
    }
}

impl<T: Copy + Num + ConstZero> AxisVec<T> {
    /// Returns the sum of both axis elements.
    pub fn sum(&self) -> T
    where
        T: Sum<T>,
    {
        self.0.into_iter().sum()
    }

    /// Returns the product of both axis elements.
    pub fn product(&self) -> T
    where
        T: Product<T>,
    {
        self.0.into_iter().product()
    }

    /// Returns `true` if both axis elements are equal to zero.
    #[inline]
    pub fn are_zero(&self) -> bool {
        self.0.iter().all(|el| *el == T::ZERO)
    }
}

impl<T> AxisVec<Option<T>> {
    /// Converts from `AxisVec<Option<T>>` to `Option<AxisVec<T>>`.
    pub fn flatten(self) -> Option<AxisVec<T>> {
        match self.0 {
            [Some(x), Some(y)] => Some(AxisVec::new(x, y)),
            _ => None,
        }
    }
}

impl<T, E> AxisVec<Result<T, E>> {
    /// Converts from `AxisVec<Result<T, E>>` to `Result<AxisVec<T>, E>`.
    pub fn flatten(self) -> Result<AxisVec<T>, E> {
        match self.0 {
            [Ok(x), Ok(y)] => Ok(AxisVec::new(x, y)),
            [Err(e), _] | [_, Err(e)] => Err(e),
        }
    }
}

impl<T: Default + Copy> Default for AxisVec<T> {
    fn default() -> Self {
        Self([T::default(); 2])
    }
}

impl<T> From<(T, T)> for AxisVec<T> {
    fn from((x, y): (T, T)) -> Self {
        AxisVec::new(x, y)
    }
}

impl<T> From<[T; 2]> for AxisVec<T> {
    fn from(values: [T; 2]) -> Self {
        AxisVec(values)
    }
}

impl<T> Add<T> for AxisVec<T>
where
    T: Clone + Copy + Add<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn add(mut self, rhs: T) -> Self::Output {
        self.0.iter_mut().for_each(|el| *el = *el + rhs);
        self
    }
}

impl<T> Sub<T> for AxisVec<T>
where
    T: Clone + Copy + Sub<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn sub(mut self, rhs: T) -> Self::Output {
        self.0.iter_mut().for_each(|el| *el = *el - rhs);
        self
    }
}

impl<T> Mul<T> for AxisVec<T>
where
    T: Clone + Copy + Mul<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn mul(mut self, rhs: T) -> Self::Output {
        self.0.iter_mut().for_each(|el| *el = *el * rhs);
        self
    }
}
