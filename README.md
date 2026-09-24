## Tilebound

A framework for resolving tilemap collisions.

### Summary

`tilebound` is a library of types, representations and operations for handling and resolving
bounding-box displacement and collision on a 2d array. `tilebound` includes:
- Types and operations for single-axis displacement,
- Tilemap traits and implementations for simple `std` types, more abstract bit based types, and
external libraries,
- Exact tile bound alignment with no [`f32::EPSILON`], [`f32::next_down`]/[`f32::next_up`] or
other offsetting (with respect to the limits of floating-point accuracy),
- Collision representation and assertions (referred within as *attach* and *detach*),
- Various helper methods and operations for common workflows and optimizations,
- Generic axis markers allowing identical functionality on both axes (if desired) and reduced testing,
- `macroquad` examples.

### Core Traits and Types

The following are some core traits and types prevalent throughout this crate. See the individual
type documentation for more.

#### Traits

Type | Description
---|---
[`Axis`](prelude::Axis) | `const` axis marker types. Resolves to variables and associated view types for tilemaps.
[`Scale`](prelude::Scale) | Declares a `const` integer tile size for both axes. Used for most mathematical operations.
[`TileMap`](prelude::TileMap)/[`TileMapView`](prelude::TileMapView) | Exposes tilemap dimensions, view operations and associated row/column types for various array types.
[`BoundBoxView`](prelude::BoundBoxView) | Variable access for `BoundBox` representations (See below).

#### Types

Type | Description
---|---
[`Scene`](prelude::Scene) | Wraps a tilemap type with some minimal `tilebound` configuration.
[`BoundBox`](prelude::BoundBox) | Bounding box and collision data representations.
[`State`](ctx::state::State) | An intermediate used for bounding box displacement operations.
[`Endpoint`](prelude::Endpoint) | Describes a single direction on an axis, orientating numerical operations.
[`Vertex`](topology::vertex::Vertex)/[`Edge`](topology::edge::Edge) | A bounding box `Endpoint` position for alignment and displacement.
[`AxisMask`](plane::axis::AxisMask) | A bit mask indicating endpoints or direction(s) on the *x* and *y* plane.
[`Delta`](topology::delta::Delta) | Steps over a delta from an *origin* to a *target* producing vertex positions.
[`Break`](ctx::brk::Break) | A simple enum return type for displacement operations and control-flow, possibly including collision data.

#### Helper operations

`tilebound` further includes various helper functions to simplify common workflows (See [`ops`]).

### Usage

The following is a simple `tilebound` workflow. It omits functionality and optimizations for
simplicity. See [`tilebound_solid`](#implementations-1) below for an extended workflow with
optimizations, collision masks, map boundaries, and more.

#### Tilemap Setup

[`Scene`](prelude::Scene) describes a tilemap (usually) implementing [`TileMap`](prelude::TileMap)
and [`TileMapView`](prelude::TileMapView). Tile represention and interpretation (solid tiles,
slopes, one way tiles etc.) is left to implemention. The following is a simple binary tilemap
where `0` and `1` represent empty and solid respectively.

```rust
// A simple (4 * 5) boolean tilemap representation (where 1 == solid).
let map = VecMap::new([
    0, 0, 0, 1, 0,
    0, 0, 0, 1, 0,
    0, 0, 0, 0, 0,
    0, 0, 0, 0, 0,
].to_vec(), (4, 5));

// Specify a 16 * 16 tile size.
type Sc = ConSc<16>;
// Wrap the tilemap, pass in the tile size and declare no solid map bounds.
let scene = Scene::<Sc, _>::new(map, AxisMask::NONE);
```

#### Sweep displacement functionality

`tilebound` uses [`State`](ctx::state::State) as an intermediate representation of bounding box
variables. It represents each side of a bounding box as a [`Vertex`](topology::vertex::Vertex)
endpoint for tile bound alignment. Here we declare a function that uses [`Delta`](topology::delta::Delta)
to iterate over vertices positioned at the furthest point of each tile to the target position.
We query the tiles for each vertex, and set each position to our state if no solid tile is
encountered. Note that many operations require the scale (`Sc`) defined earlier.

```rust
// We accept a generic `Axis` so we can displace on both axes, and require a tilemap of `u8` variables.
fn displace_by_delta<A, Sc, Map>(delta: f32, state: &mut State, scene: &Scene<Sc, Map>)
where
    A: Axis,
    Sc: Scale,
    Map: TileMapView<A::T, El = u8>
{
    // Create a `Delta` of the given amount.
    let Some(delta) = delta_by::<A, Sc, _>(&state, delta) else {
        // Return if we already intersect the target.
        return;
    };
    // Get the closest vertex to the target.
    let origin = state.vertex::<_, Sc>(delta.end());

    // Iterate over furthest point in each tile until the target is reached.
    for tgt in delta.iter_inbound_to_target::<Sc>(origin) {
        // We view the intersected tiles on the tranpose axis `A::T`. That is, when displacing on axis *x*, we check a column (an axis *y* slice view).
        match scene.view_intersected(tgt, state) {
             Ok(view) => {
                 // Check if any tile of the view is solid and either and either end displacement, or set to the next position.
                 if view.into_iter().any(|tile| *tile != 0) {
                     // There is at least one solid tile on the tile bound. End displacement.
                     break;
                 } else {
                     // There are no solid tiles on the view. Set the position to the target (and continue).
                     state.set_vertex(tgt);
                 }
             }
             Err(_) => {
                 // No tiles are in-bound. Displace immediately.
                 state.set_vertex(tgt);
             }
        }
    }
}
```

Note that this example omits a large amount of functionality. See the [`tilebound_solid`](#implementations-1)
for an extended example.

#### Displacing a bounding box

With the core functionality implemented, we can now displace a bounding box.

```rust
// A bounding box at position (0.0, 0.0) equal to one tile in size (16.0, 16.0) with no collisions set.
let mut bounding_box = BoundBox::new((0.0, 0.0), (16.0, 16.0), AxisMask::NONE);
let mut state = State::new::<Sc, _>(&bounding_box);

// Displace 6 tiles to the right.
displace_by_delta::<AxisX, _, _>(Sc::SCALE * 6.0, &mut state, &scene);
// Assert we collided without reaching our target.
assert_eq!(state.pos_vec::<Sc>(), (32.0, 0.0).into());

// Displace 2 tiles downward.
displace_by_delta::<AxisY, _, _>(Sc::SCALE * 2.0, &mut state, &scene);
// Assert we reaching our target.
assert_eq!(state.pos_vec::<Sc>(), (32.0, 32.0).into());

// Displace 8.0 units to the right.
displace_by_delta::<AxisX, _, _>(8.0, &mut state, &scene);
// Assert we collided without reaching our target.
assert_eq!(state.pos_vec::<Sc>(), (40.0, 32.0).into());

// Commit all changes back to the bounding box.
state.apply::<Sc, _>(&mut bounding_box);
assert_eq!(bounding_box.pos(), (40.0, 32.0).into())
```

### Implementations

The following crates include feature-complete (but experimental) `tilebound` implementations:

Name | Description
---|---
[`tilebound_solid`](https://crates.io/crates/tilebound_solid) | A small boolean/bit tilemap collision libary and a simple example of `tilebound` usage.
[`tilebound_halfslope`](https://crates.io/crates/tilebound_halfslope) | A tilemap collision libary supporting slopes and one-way tiles of any orientation.

### Features

`tilebound` exposes various [`TileMap`](view::tilemap::TileMap) and [`TileMapView`](view::tilemap::TileMapView)
implementations as optional features.

Feature | Description
---|---
`arraymap` | Exports tilemap implementations build on `std` types, namely [`ArrayMap`](prelude::ArrayMap) and [`VecMap`](prelude::VecMap).
`bitmap` | Exports [`BitMap`](prelude::BitMap), a `bitvec` backed tilemap where each tile represents one bit.
`packmap` | Exports [`packmap`](prelude::PackMap), an *experimental* bitfield backed tilemap where each tile represents `2` or `4` bits.
`ndarray` | Implements [`TileMap`](prelude::TileMap) and [`TileMapView`](prelude::TileMapView) for `ndarray`s `Array2` type.
`mdarray` | Implements [`TileMap`](prelude::TileMap) and [`TileMapView`](prelude::TileMapView) for `mdarray`s two-dimensional `Array` type.

License: MIT OR Apache-2.0
