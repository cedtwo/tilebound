# Tilebound

A framework for resolving tilemap collisions.

## Summary

`tilebound` is a library of types, representations and operations for handling and resolving
bounding-box displacement and collision on a 2d array. `tilebound` includes:
- Types and operations for single-axis displacement,
- Tilemap traits and implementations for simple `std` types, more abstract bit based types, and
external libraries,
- Exact tile bound alignment with no [`f32::EPSILON`], [`f32::next_down`]/[`f32::next_up`] or
other offsetting (with respect to the limits of floating-point accuracy),
- Collision representation and assertions (referred within as *attach* and *detach*),
- Various helper methods and operations for common workflows and optimizations,
- Generic axis markers allowing for a single axis implementation to function identically on both
axes (if desired) while reducing testing to a single axis,
- Implementations that demonstrate a workflow with `macroquad` examples.

## Core Traits and Types

The following are some core traits and types prevalent throughout this crate. See the individual
type documentation for more.

### Traits

Type | Description
---|---
[`Axis`](prelude::Axis) | `const` axis marker types. Resolves to variables and associated view types for tilemaps.
[`Scale`](prelude::Scale) | Declares a `const` integer tile size for both axes. Used for most mathematical operations.
[`TileMap`](prelude::TileMap)/[`TileMapView`](prelude::TileMapView) | Exposes tilemap dimensions, view operations and associated row/column types for various array types.

### Types

Type | Description
---|---
[`Scene`](prelude::Scene) | Wraps a tilemap type with some minimal `tilebound` configuration.
[`State`](prelude::State) | An intermediate used for bounding box displacement operations.
[`Vertex`](topology::vertex::Vertex)/[`Edge`](topology::edge::Edge)/[`Endpoint`](prelude::Endpoint) | Bounding box position alignment, orientation, displacement and indices.
[`Delta`](ctx::delta::Delta) | Steps over a delta from an *origin* to a *target* producing vertex positions.
[`Break`](ctx::brk::Break) | A simple enum return type for displacement operations and control-flow, possibly including collision data.

### Helper operations

`tilebound` further includes various helper functions to simplify common workflows (See [`ops`]).

## Usage

`tilebound` provides operations to simplify single-axis displacement on a tilemap. It was build
with the core goal of sweeping over a delta, only checking the tilemap on-demand (where
displacing into a new tile, or clearing an attachment when moving off a ledge). This is not
enforced at all, and there is no reason why a implementation wouldn't, for example, check the
tilemap each frame to avoid the complexity of tracking state. The above workflow is what was
used for the included [implementations](#implementations) and the workflow is described below.

For such a workflow an implementation would create a [`Delta`](topology::delta::Delta), loop over
the furthest (*outer*) vertices for each tile index intersected by the delta, check for a
collision within that tile and either displace, or end displacement at the furthest point prior
to a collision. This process can be further optimized by skipping a tilemap check if already
intersecting the first vertex index produced, or displacing immediately if out-of-bounds. A
large number of these common optimizations are included in the [`ops`] module.

`tilebound_solid` is the simplest demonstration of `tilebound`, implementing the above workflow
in barely over 200 lines (including documentation). `tilebound_halfslope` implements the same
workflow however is significantly more complex. See [implementations](#implementations) below.

## Implementations

The following crates include feature-complete (but experimental) `tilebound` implementations:

Name | Description
---|---
[`tilebound_solid`](https://crates.io/crates/tilebound_solid) | A small boolean/bit tilemap collision libary and a simple example of `tilebound` usage.
[`tilebound_halfslope`](https://crates.io/crates/tilebound_halfslope) | A tilemap collision libary supporting slopes and one-way tiles of any orientation.

## Features

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
