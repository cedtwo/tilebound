use std::marker::PhantomData;
use std::ops::Range;

use crate::ctx::state::State;
use crate::plane::axis::{Axis, AxisMask, AxisVec};
use crate::plane::endpoint::Endpoint;
use crate::plane::scale::Scale;
use crate::topology::vertex::Vertex;
use crate::view::index::{SliceIndex, SliceIndexError};
use crate::view::tilemap::{TileMap, TileMapView};

/// # Scene
///
/// [`TileMap`] access and configuration.
///
/// `Scene` is used to define a constant integer [`Scale`] (`Sc`) for both axes of a tile, a
/// [`TileMap`] (and likely [`TileMapView`]) implementing type (`Map`), and the solid map bounds
/// represented as an [`AxisMask`]. Fields variables can be freely accessed, mutated and replaced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scene<Sc, Map> {
    /// The (integer) size of a tile on either dimension.
    _scale: PhantomData<Sc>,

    /// The [`TileMap`] implementing type.
    pub map: Map,
    /// Indicates which sides of the map are solid.
    pub map_bounds: AxisMask,
}

impl<Sc, Map> Scene<Sc, Map> {
    /// Create a new `Scene` with the given `map` and `map_bounds`.
    pub fn new(map: Map, map_bounds: AxisMask) -> Self {
        Self {
            _scale: PhantomData,
            map,
            map_bounds,
        }
    }
}

impl<Sc: Scale, Map: TileMap> Scene<Sc, Map> {
    /// Return the [`TileMap`] size.
    pub fn size(&self) -> AxisVec<usize> {
        self.map.size()
    }

    /// Return a one-dimensional slice of the [`TileMap`] at the given `index` and transpose range.
    pub fn view_slice<'a, A>(
        &'a self,
        index: i32,
        t_range: Range<i32>,
        end: Endpoint,
    ) -> Result<Map::View<'a>, SliceIndexError<A>>
    where
        Map: TileMapView<A>,
        A: Axis,
    {
        SliceIndex::<A>::try_new(index, t_range, end, self.size())
            .map(|index| self.map.view(index.t_index, index.range_cropped()))
    }

    /// Return a one-dimensional slice of the [`TileMap`] intersecting the given `vertex`.
    pub fn view_intersected<'a, A, R>(
        &'a self,
        state: &State<R>,
        vertex: Vertex<A>,
    ) -> Result<Map::View<'a>, SliceIndexError<A>>
    where
        Map: TileMapView<A>,
        A: Axis,
    {
        let t_range = state.index_range::<A::T, Sc>();
        SliceIndex::<A>::try_new(vertex.index(), t_range, vertex.endpoint(), self.map.size())
            .map(|index| self.map.view(index.t_index, index.range_cropped()))
    }
}
