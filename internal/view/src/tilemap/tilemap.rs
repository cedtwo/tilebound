use std::ops::{Deref, Range};

use tilebound_plane::axis::{Axis, AxisVec, AxisX, AxisY};
use tilebound_plane::endpoint::{Endpoint, EndpointRange};

/// # TileMap
///
/// `TileMap` is implemented on 2-dimensional arrays of tile representations. Implementations expose
/// the tilemap element type (consumed by [`TileMapView`]), tilemap dimensions and assertion methods.
/// See also [`TileMapView`].
pub trait TileMap {
    /// The tilemap element type.
    type El;

    /// Get the number of elements in a slice of axis `A`.
    fn axis_len<A: Axis>(&self) -> usize;

    /// Get the number of elements on both axes.
    fn size(&self) -> AxisVec<usize> {
        AxisVec::new(self.axis_len::<AxisX>(), self.axis_len::<AxisY>())
    }

    /// Get the lower/upper inclusive tile index for the given [`Endpoint`] of axis `A`.
    fn bound_index<A: Axis>(&self, end: Endpoint) -> usize {
        match end {
            Endpoint::LOW => 0,
            Endpoint::UPP => self.axis_len::<A>() - 1,
        }
    }

    /// Get a [`Range`] containing the bound indices.
    fn index_range<A: Axis>(&self) -> Range<usize> {
        0..self.axis_len::<A>()
    }

    /// Returns `true` if the given `index` is in bounds.
    fn is_in_bounds<A: Axis>(&self, index: i32) -> bool {
        index >= 0 && index < self.axis_len::<A>() as i32
    }

    /// Returns `true` if *any* part of the given `index` and transpose `t_range` is in bounds.
    fn range_is_in_bounds<A: Axis>(&self, index: i32, t_range: Range<i32>) -> bool {
        self.is_in_bounds::<A>(index) && t_range.intersects(&(0..self.axis_len::<A::T>() as i32))
    }
}

// Implementation for tilemap references.
impl<'a, T: TileMap> TileMap for &'a T {
    type El = T::El;

    fn axis_len<A: Axis>(&self) -> usize {
        T::axis_len::<A>(&self)
    }

    fn size(&self) -> AxisVec<usize> {
        T::size(&self)
    }
}

/// # TileMapView
///
/// Exposes a one-dimensional tilemap slice view. Implementations return a range of elements on a
/// transpose axis index (an [`AxisX`] row and an [`AxisY`] column respectively.
pub trait TileMapView<A: Axis>: TileMap {
    /// The axis slice view type.
    type View<'a>: IntoIterator<Item: Deref<Target = Self::El>>
    where
        Self: 'a;

    /// Iterate over over a range of elements for the fixed axis `A` index, and the transpose range.
    fn view(&self, idx: usize, t_range: Range<usize>) -> Self::View<'_>;
}

impl<'a, A: Axis, T: TileMapView<A>> TileMapView<A> for &'a T {
    type View<'b>
        = T::View<'b>
    where
        Self: 'b;

    fn view(&self, idx: usize, range: Range<usize>) -> Self::View<'_> {
        T::view(&self, idx, range)
    }
}
