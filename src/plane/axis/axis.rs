use std::fmt::Debug;

use marker_value::MarkerValue;

/// # Axis
///
/// Axis markers and indices.
///
/// `Axis` is is a `const` marker type used for variable access type check enforcement.
///
/// `Axis` is used as a generic argument for types and operation that differ by axis (usually in
/// relation to mapping or retrieving variables). [`Axis`] types can refer to the transpose of the
/// given axis with [`Axis`], and an index for array access or identification using
/// [`MarkerValue::VALUE`]. It offers the potential for better optimization at the cost of an
/// increased binary size. See also [`DynAxis`] and [`Axis::into_dyn`].
pub trait Axis: MarkerValue<usize> + Debug + Clone + Copy + PartialEq + Eq {
    /// The transpose axis type.
    type T: Axis<T = Self>;

    /// Return the axis as a [`DynAxis`].
    fn into_dyn() -> DynAxis {
        match Self::VALUE {
            AxisX::VALUE => DynAxis::X,
            AxisY::VALUE => DynAxis::Y,
            _ => unreachable!(),
        }
    }
}

/// Constant axis **X** identifier. See [`Axis`].
#[derive(MarkerValue, Debug, Clone, Copy, PartialEq, Eq)]
#[marker_value(val = 0)]
pub struct AxisX;

impl Axis for AxisX {
    type T = AxisY;
}

/// Constant axis **Y** identifier. See [`Axis`].
#[derive(MarkerValue, Debug, Clone, Copy, PartialEq, Eq)]
#[marker_value(val = 1)]
pub struct AxisY;

impl Axis for AxisY {
    type T = AxisX;
}

/// # DynAxis (Dynamic Axis)
///
/// Dynamic axis identifier. Used as an alternative to a `const` [`Axis`] marker variant where a
/// fixed generic axis is cumbersome, or a single type is preferred. This is sometimes used as a
/// placeholder marker type where axis is inferred (eg. in [`AxisVec`](crate::axis::AxisVec)). Use
/// `*self as usize` to cast as a usize index. See also [`Axis`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynAxis {
    X = AxisX::VALUE as _,
    Y = AxisY::VALUE as _,
}

impl DynAxis {
    /// Return the transpose axis.
    pub const fn t(self) -> Self {
        match self {
            DynAxis::X => DynAxis::Y,
            DynAxis::Y => DynAxis::X,
        }
    }
}
