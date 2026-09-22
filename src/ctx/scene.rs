use std::marker::PhantomData;

use crate::ctx::state::State;
use crate::plane::axis::{Axis, AxisMask, AxisVec};
use crate::plane::endpoint::Endpoint;
use crate::plane::scale::Scale;
use crate::topology::vertex::Vertex;
use crate::view::index::{SliceIndex, SliceIndexError};
use crate::view::inspect::TileMapInspect;
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

    /// Initialize an intersecting [`SliceIndex`] from a vertex. This will create a [`SliceIndex`]
    /// on the **transpose** axis for the **inverted** endpoint of the given vertex index. In
    /// practical terms, for an [`Endpoint::RIGHT`] vertex, this will return an index for the
    /// left-hand side of the intersecting *column*. Returns a [`SliceIndexError`] if no index is in
    /// bounds.
    pub fn index_intersected<'a, A, P>(
        &'a self,
        vertex: Vertex<A>,
        state: &State<P>,
    ) -> Result<SliceIndex<A::T>, SliceIndexError<A::T>>
    where
        A: Axis,
        Sc: Scale,
    {
        let (t_index, range) = (vertex.index(), state.index_range::<A::T, Sc>());
        SliceIndex::<A::T>::try_new(t_index, range, !vertex.endpoint(), self.map.size())
    }

    /// Return a [`TileMapInspect`] implementing type for indices intersecting the given `index`
    /// slice. Returns a [`SliceIndexError`] if no index is in bounds.
    pub fn inspect_index<A, P, Ins>(
        &self,
        index: i32,
        end: Endpoint<A::T>,
        state: &State<P>,
    ) -> Result<Ins, SliceIndexError<A>>
    where
        A: Axis,
        Sc: Scale,
        Map: TileMapView<A>,
        Ins: TileMapInspect<A, Map>,
    {
        let range = state.index_range::<A::T, Sc>();
        SliceIndex::<A>::try_new(index, range, !end, self.map.size())
            .map(|index| Ins::inspect(&self.map, &index))
    }

    /// Return a [`TileMapInspect`] implementing type for indices intersecting the given `vertex`.
    /// This will create a [`TileMapInspect`] type for an index on the **transpose** axis for the
    /// **inverted** endpoint of the given vertex index. In practical terms, for an
    /// [`Endpoint::RIGHT`] vertex, this will return a type aggregate for the left-hand side of the
    /// intersecting *column*. Returns a [`SliceIndexError`] if no index is in bounds.
    pub fn inspect_intersected<A, P, Ins>(
        &self,
        vertex: Vertex<A>,
        state: &State<P>,
    ) -> Result<Ins, SliceIndexError<A::T>>
    where
        A: Axis,
        Sc: Scale,
        Map: TileMapView<A::T>,
        Ins: TileMapInspect<A::T, Map>,
    {
        self.index_intersected(vertex, state)
            .map(|index| Ins::inspect(&self.map, &index))
    }
}
