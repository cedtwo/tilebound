use std::error::Error;
use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::{Neg, Range};

use tilebound_plane::axis::{Axis, AxisVec};
use tilebound_plane::endpoint::{Endpoint, EndpointPair, EndpointRange};

/// # SliceIndex
///
/// `SliceIndex` is a (validated) range of tiles which at least partially intersect the map on both
/// axes. Indices are composed of a range of elements on the generic axis `A`, and a transpose axis
/// slice index. [`AxisX`] and [`AxisY`] is therefore a *row* and *column* respectively.
///
/// `SliceIndex` offers operations for cropping an index range in-bounds, and retrieving the number
/// of out-of-bound elements in the range (referred to as *padding* in methods).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceIndex<A: Axis> {
    /// The tile range axis.
    pub _axis: PhantomData<A>,
    /// The tile index on the transpose axis.
    pub t_index: usize,
    /// The given slice range on axis `A`.
    pub range: Range<i32>,
    /// The upper exclusive index bound on axis `A`.
    pub range_lmt: i32,
    /// The tile bound [`Endpoint`] on the tranpose axis.
    pub end: Endpoint,
}

impl<A: Axis> SliceIndex<A> {
    /// Create a new `SliceIndex` from the given variables. Clamps the range to elements within
    /// the `map_size`. Returns `None` if all elements are out-of-bounds.
    pub fn try_new(
        t_index: i32,
        range: Range<i32>,
        end: Endpoint,
        map_size: AxisVec<usize>,
    ) -> Result<Self, SliceIndexError<A>> {
        let limit = map_size.get::<A::T>() as i32;
        let Some(index_inb) = usize::try_from(t_index)
            .ok()
            .filter(|index| *index < map_size.get::<A>() && range.intersects(&(0..limit)))
        else {
            return Err(SliceIndexError::new(t_index, range, end));
        };

        Ok(SliceIndex {
            _axis: PhantomData,
            end,
            t_index: index_inb,
            range,
            range_lmt: limit,
        })
    }

    /// Create a new `SliceIndex` assuming the given `index` and `t_range` is at least partially
    /// in-bounds.
    pub fn new_unchecked(t_index: i32, range: Range<i32>, range_lmt: i32, end: Endpoint) -> Self {
        debug_assert!(t_index >= 0);
        SliceIndex {
            _axis: PhantomData,
            end,
            t_index: t_index as usize,
            range,
            range_lmt,
        }
    }

    /// Get the padding (number of out of bound elements) for each endpoint.
    pub fn padding(&self) -> EndpointPair<i32> {
        let neg_pad = if self.range.start.is_negative() {
            self.range.start.neg()
        } else {
            0
        };
        let pos_pad = {
            let diff = self.range_lmt - self.range.end;
            if diff < 0 { diff.neg() } else { 0 }
        };
        EndpointPair::new(neg_pad, pos_pad)
    }

    /// Get the slice starting index.
    pub fn start_index(&self) -> AxisVec<i32> {
        match self.end {
            Endpoint::Lower => AxisVec::new_mapped::<A>(self.t_index as i32, *self.range.lower()),
            Endpoint::Upper => {
                AxisVec::new_mapped::<A>(self.t_index as i32 + 1, *self.range.lower())
            }
        }
    }

    /// Get the index on the transpose axis.
    pub fn t_index(&self) -> usize {
        self.t_index
    }

    /// Return the slice [`Range`].
    pub fn range(&self) -> Range<i32> {
        self.range.clone()
    }

    /// Return the slice [`Range`] cropped to in-bounds.
    pub fn range_cropped(&self) -> Range<usize> {
        self.inbound_range_start() as usize..self.inbound_range_end() as usize
    }

    /// Return the inbound range start.
    pub fn inbound_range_start(&self) -> i32 {
        self.range.start.max(0)
    }

    /// Return the inbound range end.
    pub fn inbound_range_end(&self) -> i32 {
        self.range.end.min(self.range_lmt)
    }

    /// Returns the number of tiles in the slice (including out-of-bound tiles). See also
    /// [`Self::len_cropped`].
    pub fn len(&self) -> usize {
        self.range.len()
    }

    /// Returns the number of in-bound tiles in the slice. See also [`Self::len`].
    pub fn len_cropped(&self) -> usize {
        (self.inbound_range_start()..self.inbound_range_end()).len()
    }
}

/// # SliceIndexError
///
/// [`Error`] type for out-of-bound indices.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SliceIndexError<A> {
    /// The tile range axis.
    pub _axis: PhantomData<A>,
    /// The transpose axis index.
    pub t_index: i32,
    /// The input range on axis `A`.
    pub range: Range<i32>,
    /// The tile bound [`Endpoint`].
    pub end: Endpoint,
}

impl<A: Axis> Display for SliceIndexError<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let start = AxisVec::new_mapped::<A>(self.t_index, self.range.start);
        let end = start.map_axis::<A::T, _>(|_| self.range.end);
        write!(
            f,
            "index out of bounds: {:?}.. {:?}",
            start.split(),
            end.split()
        )
    }
}

impl<A> SliceIndexError<A> {
    /// Create a new `OutOfBoundsError` from an index, a transpose range and an endpoint.
    pub fn new(t_index: i32, range: Range<i32>, end: Endpoint) -> Self {
        Self {
            _axis: PhantomData,
            end,
            t_index,
            range,
        }
    }
}

impl<A: Axis> Error for SliceIndexError<A> {}

#[cfg(test)]
mod tests {

    use super::*;
    use tilebound_plane::axis::AxisX;

    #[test]
    fn try_new() {
        let end = Endpoint::LOW;
        let map_bounds = AxisVec::new(8, 8);
        let idx_0 = SliceIndex::<AxisX>::try_new(0, (1..2).into(), end, map_bounds);
        let idx_1 = SliceIndex::<AxisX>::try_new(-1, (1..2).into(), end, map_bounds);
        let idx_2 = SliceIndex::<AxisX>::try_new(8, (1..2).into(), end, map_bounds);
        let idx_3 = SliceIndex::<AxisX>::try_new(0, (-2..2).into(), end, map_bounds);
        let idx_4 = SliceIndex::<AxisX>::try_new(0, (6..10).into(), end, map_bounds);
        let idx_5 = SliceIndex::<AxisX>::try_new(0, (-2..10).into(), end, map_bounds);
        let idx_6 = SliceIndex::<AxisX>::try_new(0, (-2..-1).into(), end, map_bounds);
        let idx_7 = SliceIndex::<AxisX>::try_new(0, (8..9).into(), end, map_bounds);

        assert!(idx_0.is_ok());
        assert!(idx_1.is_err());
        assert!(idx_2.is_err());
        assert!(idx_3.is_ok());
        assert!(idx_4.is_ok());
        assert!(idx_5.is_ok());
        assert!(idx_6.is_err());
        assert!(idx_7.is_err());
    }

    #[test]
    fn padding() {
        let map_bounds = AxisVec::new(3, 3);
        let idx =
            SliceIndex::<AxisX>::try_new(0, (-2..5).into(), Endpoint::Lower, map_bounds).unwrap();

        assert_eq!(idx.padding(), (2, 2).into());
    }

    #[test]
    fn range_cropped() {
        let map_bounds = AxisVec::new(3, 3);
        let idx =
            SliceIndex::<AxisX>::try_new(0, (-2..5).into(), Endpoint::Lower, map_bounds).unwrap();

        assert_eq!(idx.range_cropped(), 0..3);
    }

    #[test]
    fn len_cropped() {
        let map_bounds = AxisVec::new(3, 3);
        let idx =
            SliceIndex::<AxisX>::try_new(0, (-2..5).into(), Endpoint::Lower, map_bounds).unwrap();

        assert_eq!(idx.len_cropped(), 3);
    }
}
