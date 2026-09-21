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
}
