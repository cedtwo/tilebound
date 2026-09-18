use crate::index::SliceIndex;
use crate::tilemap::TileMapView;
use tilebound_plane::axis::Axis;

/// # TileMapInspect
///
/// `TileMapInspect` types are a contextual representation of a
/// [`TileMapView::View`](crate::tilemap::TileMapView::View).
///
/// Implementations usually inform on whether a slice of tiles can be entered, possibly providing
/// additional data on a collision, or state variables. Implementations are provided with the
/// [`TileMapView`] and [`SliceIndex`] for flexibility in modify indices if needed. Implementations
/// usually need only call `map.view(index.index, index.t_range_cropped())` for a tilemap view.
pub trait TileMapInspect<A: Axis, Map: TileMapView<A>>: Sized {
    /// Return the implementing type from a [`TileMap`](crate::TileMap) and `index`.
    fn inspect(map: &Map, index: &SliceIndex<A>) -> Self;
}
