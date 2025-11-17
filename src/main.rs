use bevy::prelude::*;
mod grid;
mod constants;
mod camera;
mod collision;
mod world;
mod setup;

fn main() {
    App::new()
        .insert_resource(world::Blocked::default()) // fill this at load
        .add_plugins(DefaultPlugins)
        // .add_systems(Startup, setup)
        .add_systems(Startup, (setup::scene, setup::minimap))
        // .add_systems(Startup, spawn_asset.after(setup::scene))
        .add_systems(Update, grid::draw_grid_gizmos) // draw grid
        .add_systems(Update, (collision::move_with_collision_system, collision::sync_render_from_grid))
        .add_systems(Update, (camera::handle_spin_input, (camera::animate_camera_spin, camera::follow_center_snap, camera::sync_minimap_to_iso_yaw).chain()))        
        .run();
}

// --- Separate system to load your Blender asset (.glb) ---
fn spawn_asset(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((        
        SceneRoot(asset_server.load("models/resource.glb#Scene0")),
        Transform::from_xyz(1.0, 1.0, 1.0),
    ));
}