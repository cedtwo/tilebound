use std::fmt::Debug;
use std::iter::{Product, Sum};
use std::ops::{Add, Deref, Mul, Sub};

use marker_value::MarkerValue;
use num_traits::{ConstZero, Num};

use crate::plane::axis::*;

/// # AxisVec
///
/// A two-dimensional vector of values `T` mapped to an axis index. Provides operations for element
/// access and mutation by either generic or dynamic axis ([`Axis`] or [`DynAxis`] respectively).
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq, Ord, Eq)]
pub struct AxisVec<T> {
    pub x: T,
    pub y: T,
}

impl<T> AxisVec<T> {
    /// Create a new `AxisVec` of the given variables.
    #[inline]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    /// Create a new `AxisVec` setting both variables to the given `value`.
    #[inline]
    pub const fn fill(value: T) -> Self
    where
        T: Copy,
    {
        Self::new(value, value)
    }

    /// Get a reference to the value assigned to axis *x*.
    pub const fn x_ref(&self) -> &T {
        &self.x
    }

    /// Get a reference to the value assigned to axis *y*.
    pub const fn y_ref(&self) -> &T {
        &self.y
    }

    /// Get a mutable reference to the value assigned to axis *x*.
    pub const fn x_mut(&mut self) -> &mut T {
        &mut self.x
    }

    /// Get a mutable reference to the value assigned to axis *y*.
    pub const fn y_mut(&mut self) -> &mut T {
        &mut self.y
    }

    /// Return the `AxisVec` as a tuple of the *x* and *y* variables respectively.
    pub fn split(self) -> (T, T) {
        (self.x, self.y)
    }

    /// Return the `AxisVec` as an array of the *x* and *y* variables respectively.
    pub fn into_array(self) -> [T; 2] {
        [self.x, self.y]
    }

    /// Return an `AxisVec` with the result of the function `F` applied to each element.
    pub fn map<F: FnMut(T) -> U, U>(self, mut f: F) -> AxisVec<U> {
        AxisVec::new(f(self.x), f(self.y))
    }

    /// Return an `AxisVec` with the result of the function `F` applied to the elements of `self`
    /// and other.
    pub fn map_with<F: FnMut(T, U) -> V, U, V>(self, other: AxisVec<U>, mut f: F) -> AxisVec<V>
    where
        T: Copy,
        U: Copy,
    {
        AxisVec::new(f(self.x, other.x), f(self.y, other.y))
    }

    /// Calling [`Into::into`] on the inner variables, returning `AxisVec<U>`.
    pub fn map_into<U>(self) -> AxisVec<U>
    where
        T: Into<U>,
    {
        AxisVec::new(self.x.into(), self.y.into())
    }

    /// Calling [`Deref::deref`] on the inner variables, returning `AxisVec<U>`.
    pub fn map_deref<U>(self) -> AxisVec<U>
    where
        T: Deref<Target = U>,
        U: Copy,
    {
        AxisVec::new(*self.x, *self.y)
    }

    /// Calling [`TryInto::try_into`] on the inner variables, returning `Result<AxisVec>`.
    pub fn try_map_into<U>(self) -> Result<AxisVec<U>, T::Error>
    where
        T: TryInto<U, Error: Debug>,
    {
        AxisVec::new(self.x.try_into(), self.y.try_into()).flatten()
    }

    /// Returns `true` if any element satisfies the predicate `f`.
    #[inline]
    pub fn any<F: Fn(&T) -> bool>(self, f: F) -> bool {
        f(&self.x) || f(&self.y)
    }

    /// Returns `true` if all elements satisfy the predicate `f`.
    #[inline]
    pub fn all<F: Fn(&T) -> bool>(self, f: F) -> bool {
        f(&self.x) && f(&self.y)
    }
}

impl<T> AxisVec<T> {
    /// Create a new `AxisVec` with element *i* and *j* passed to the given `const` axis `A` and
    /// its transpose respectively.
    pub const fn new_mapped<A: Axis>(i: T, j: T) -> Self {
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
        match A::VALUE {
            AxisX::VALUE => self.x,
            AxisY::VALUE => self.y,
            _ => unreachable!(),
        }
    }

    /// Get a reference to the value assigned to axis `A`.
    pub fn get_ref<A: Axis>(&self) -> &T {
        match A::VALUE {
            AxisX::VALUE => &self.x,
            AxisY::VALUE => &self.y,
            _ => unreachable!(),
        }
    }

    /// Get a mutable reference to the value assigned to axis `A`.
    pub fn get_mut<A: Axis>(&mut self) -> &mut T {
        match A::VALUE {
            AxisX::VALUE => &mut self.x,
            AxisY::VALUE => &mut self.y,
            _ => unreachable!(),
        }
    }

    /// Get a clone of the value assigned to axis `A`.
    pub fn get_cloned<A: Axis>(&self) -> T
    where
        T: Clone,
    {
        match A::VALUE {
            AxisX::VALUE => self.x.clone(),
            AxisY::VALUE => self.y.clone(),
            _ => unreachable!(),
        }
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
            AxisX::VALUE => AxisVec::new(f(self.x), self.y),
            AxisY::VALUE => AxisVec::new(self.x, f(self.y)),
            _ => unreachable!(),
        }
    }

    /// Split variables into a `(T, T)` tuple with the axis `A` variable first.
    pub fn split_axis<A: Axis>(self) -> (T, T) {
        match A::VALUE {
            AxisX::VALUE => (self.x, self.y),
            AxisY::VALUE => (self.y, self.x),
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
        match a as usize {
            AxisX::VALUE => self.x,
            AxisY::VALUE => self.y,
            _ => unreachable!(),
        }
    }

    /// Get a reference to the value assigned to the given dynamic axis `a`.
    pub fn get_dyn_ref(&self, a: DynAxis) -> &T {
        match a as usize {
            AxisX::VALUE => &self.x,
            AxisY::VALUE => &self.y,
            _ => unreachable!(),
        }
    }

    /// Get a mutable reference to the value assigned to the given dynamic axis `a`.
    pub fn get_dyn_mut(&mut self, a: DynAxis) -> &mut T {
        match a as usize {
            AxisX::VALUE => &mut self.x,
            AxisY::VALUE => &mut self.y,
            _ => unreachable!(),
        }
    }

    /// Get a clone of the value assigned to the given dynamic axis `a`.
    pub fn get_dyn_cloned(&self, a: DynAxis) -> T
    where
        T: Clone,
    {
        match a as usize {
            AxisX::VALUE => self.x.clone(),
            AxisY::VALUE => self.y.clone(),
            _ => unreachable!(),
        }
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
            AxisX::VALUE => AxisVec::new(f(self.x), self.y),
            AxisY::VALUE => AxisVec::new(self.x, f(self.y)),
            _ => unreachable!(),
        }
    }

    /// Split variables into a `(T, T)` tuple with the dynamic axis `a` variable first.
    pub fn split_dyn_axis(self, a: DynAxis) -> (T, T)
    where
        T: Copy,
    {
        match a as usize {
            AxisX::VALUE => (self.x, self.y),
            AxisY::VALUE => (self.y, self.x),
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
        self.x + self.y
    }

    /// Returns the product of both axis elements.
    pub fn product(&self) -> T
    where
        T: Product<T>,
    {
        self.x * self.y
    }

    /// Returns `true` if both axis elements are equal to zero.
    #[inline]
    pub fn are_zero(&self) -> bool {
        self.x == T::ZERO && self.y == T::ZERO
    }
}

impl<T: Clone> AxisVec<&T> {
    /// Return an an [`AxisVec`] with the [`Clone::clone`] called on the inner variables.
    pub fn cloned(self) -> AxisVec<T> {
        self.map(|var| var.clone())
    }
}

impl<T> AxisVec<Option<T>> {
    /// Converts from `AxisVec<Option<T>>` to `Option<AxisVec<T>>`.
    pub fn flatten(self) -> Option<AxisVec<T>> {
        match self.split() {
            (Some(x), Some(y)) => Some(AxisVec::new(x, y)),
            _ => None,
        }
    }
}

impl<T, E> AxisVec<Result<T, E>> {
    /// Converts from `AxisVec<Result<T, E>>` to `Result<AxisVec<T>, E>`.
    pub fn flatten(self) -> Result<AxisVec<T>, E> {
        match self.split() {
            (Ok(x), Ok(y)) => Ok(AxisVec::new(x, y)),
            (Err(e), _) | (_, Err(e)) => Err(e),
        }
    }
}

impl<T: Default + Copy> Default for AxisVec<T> {
    fn default() -> Self {
        Self::new(T::default(), T::default())
    }
}

impl<T> From<(T, T)> for AxisVec<T> {
    fn from((x, y): (T, T)) -> Self {
        AxisVec::new(x, y)
    }
}

impl<T> Add<T> for AxisVec<T>
where
    T: Clone + Copy + Add<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn add(self, rhs: T) -> Self::Output {
        self.map(|el| el + rhs)
    }
}

impl<T> Add<AxisVec<T>> for AxisVec<T>
where
    T: Clone + Copy + Add<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn add(self, rhs: AxisVec<T>) -> Self::Output {
        self.map_with(rhs, |a, b| a + b)
    }
}

impl<T> Sub<T> for AxisVec<T>
where
    T: Clone + Copy + Sub<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn sub(self, rhs: T) -> Self::Output {
        self.map(|el| el - rhs)
    }
}

impl<T> Sub<AxisVec<T>> for AxisVec<T>
where
    T: Clone + Copy + Sub<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn sub(self, rhs: AxisVec<T>) -> Self::Output {
        self.map_with(rhs, |a, b| a - b)
    }
}

impl<T> Mul<T> for AxisVec<T>
where
    T: Clone + Copy + Mul<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn mul(self, rhs: T) -> Self::Output {
        self.map(|el| el * rhs)
    }
}

impl<T> Mul<AxisVec<T>> for AxisVec<T>
where
    T: Clone + Copy + Mul<T, Output = T>,
{
    type Output = AxisVec<T>;

    fn mul(self, rhs: AxisVec<T>) -> Self::Output {
        self.map_with(rhs, |a, b| a * b)
    }
}
