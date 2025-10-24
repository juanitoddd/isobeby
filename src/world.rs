use bevy::{prelude::*};

/// Mark things that block movement on the grid (tiles, walls, crates).
#[derive(Component, Debug)]
pub struct Solid;

/// A resource holding blocked cells (could come from a tilemap).
#[derive(Resource, Default, Debug)]
pub struct Blocked(pub std::collections::HashSet<(i32, i32)>);

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GridPos { pub x: i32, pub y: i32 }

/// Grid (gx, gy) -> world (x, z) for rendering (Y is height).
pub fn iso_world_from_grid(gx: i32, gy: i32, tile_w: f32, tile_h: f32) -> Vec3 {
    let x = (gx as f32 - gy as f32) * (tile_w * 0.5);
    let z = (gx as f32 + gy as f32) * (tile_h * 0.5);
    Vec3::new(x, 0.0, z)
}

/// Inverse: world (x, z) -> fractional grid (gx, gy).
/// Useful if you ever need to pick tiles with a cursor or do continuous motion.
/// Round to i32 as needed.
pub fn grid_from_iso_world(x: f32, z: f32, tile_w: f32, tile_h: f32) -> Vec2 {
    let a = x / (tile_w * 0.5);
    let b = z / (tile_h * 0.5);
    // Solve:
    // a = gx - gy
    // b = gx + gy
    // => gx = (a + b)/2, gy = (b - a)/2
    Vec2::new((a + b) * 0.5, (b - a) * 0.5)
}