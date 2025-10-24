use bevy::{prelude::*};
use crate::constants::TILE_W;
use crate::constants::TILE_H;
/// Draws a grid on the XZ plane using gizmos.
/// - `extent` controls how far the grid goes in tiles
/// - `y_level` lets you offset the grid up/down if you like
pub fn draw_grid_gizmos(mut gizmos: Gizmos) {
    let extent: i32 = 20;          // 20 tiles in each direction
    let y_level: f32 = 0.0;        // grid height
    let step_x = TILE_W * 1.0;     // iso mapping spacing along X
    let step_z = TILE_H * 1.0;     // iso mapping spacing along Z

    // Minor/major line colors (slightly transparent)
    let minor = Color::srgba(1.0, 1.0, 1.0, 0.18);
    let major = Color::srgba(1.0, 1.0, 1.0, 0.35);
    let major_every = 4;

    // Lines parallel to Z (varying X)
    for i in -extent..=extent {
        let x = i as f32 * step_x;
        let color = if i % major_every == 0 { major } else { minor };
        gizmos.line(
            Vec3::new(x, y_level, -extent as f32 * step_z),
            Vec3::new(x, y_level,  extent as f32 * step_z),
            color,
        );
    }

    // Lines parallel to X (varying Z)
    for j in -extent..=extent {
        let z = j as f32 * step_z;
        let color = if j % major_every == 0 { major } else { minor };
        gizmos.line(
            Vec3::new(-extent as f32 * step_x, y_level, z),
            Vec3::new( extent as f32 * step_x, y_level, z),
            color,
        );
    }

    // Optional: emphasize origin axes
    gizmos.line(Vec3::new(-10.0, y_level, 0.0), Vec3::new(10.0, y_level, 0.0), Color::srgba(0.9, 0.2, 0.2, 0.7));
    gizmos.line(Vec3::new(0.0, y_level, -10.0), Vec3::new(0.0, y_level, 10.0), Color::srgba(0.2, 0.9, 0.2, 0.7));
}