use bevy::{prelude::*, input::keyboard::KeyCode};
use crate::world;
use crate::constants;

/// Reads keyboard, updates the entity's GridPos in discrete tile steps.
///
/// Controls:
/// - W/A/S/D = screen-aligned (↑/←/↓/→ on screen)
///   W: (-1,-1), A: (-1,+1), S: (+1,+1), D: (+1,-1)
/// - Arrow keys = grid axes (isometric diagonals)
///   Up: (0,-1), Left: (-1,0), Down: (0,+1), Right: (+1,0)
fn input_move_grid(
    keys: Res<ButtonInput<KeyCode>>,
) -> (i32, i32) {
    let mut dx = 0;
    let mut dy = 0;

    // Screen-aligned moves (1 tile per key press)
    // if keys.just_pressed(KeyCode::KeyW) { dx -= 1; dy -= 1; }
    // if keys.just_pressed(KeyCode::KeyS) { dx += 1; dy += 1; }
    // if keys.just_pressed(KeyCode::KeyA) { dx -= 1; dy += 1; }
    // if keys.just_pressed(KeyCode::KeyD) { dx += 1; dy -= 1; }

    if keys.just_pressed(KeyCode::KeyW) { dy += 1; }
    if keys.just_pressed(KeyCode::KeyS) { dy -= 1; }
    if keys.just_pressed(KeyCode::KeyA) { dx -= 1; }
    if keys.just_pressed(KeyCode::KeyD) { dx += 1; }
    
    // Optional: grid-axis arrows
    
    if keys.just_pressed(KeyCode::ArrowUp)    { dy += 1; }
    if keys.just_pressed(KeyCode::ArrowDown)  { dy -= 1; }
    if keys.just_pressed(KeyCode::ArrowLeft)  { dx -= 1; }
    if keys.just_pressed(KeyCode::ArrowRight) { dx += 1; }
    
    (dx, dy)
}

pub fn move_with_collision_system(
    blocked: Res<world::Blocked>,
    keys: Res<ButtonInput<KeyCode>>,
    mut movers: Query<&mut world::GridPos, Without<world::Solid>>, // entities that can move
) {    
    let (dx, dy) = input_move_grid(keys);    

    if dx == 0 && dy == 0 { return; }    

    for mut gp in &mut movers {
        let next = (gp.x + dx, gp.y + dy);        
        // Simple tile collision
        if !blocked.0.contains(&next) {
            gp.x = next.0;
            gp.y = next.1;
        }
        // else: blocked; optionally try sliding along one axis here
    }
}

/// After GridPos changes, sync the actual Transform to the correct world position.
pub fn sync_render_from_grid(
    mut q: Query<(&world::GridPos, &mut Transform)>,
) {
    for (gp, mut t) in &mut q {                
        t.translation = world::grid_to_iso(gp.x, gp.y, constants::TILE_W, constants::TILE_H);
    }
}