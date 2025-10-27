use bevy::{
    prelude::*,
    camera::{ScalingMode, RenderTarget},
    ui::prelude::*,
    render::{            
            render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        }
    },
};
mod grid;
mod constants;
mod camera;
mod collision;
mod world;

fn main() {
    App::new()
        .insert_resource(world::Blocked::default()) // fill this at load
        .add_plugins(DefaultPlugins)
        // .add_systems(Startup, setup)
        .add_systems(Startup, (setup_scene, setup_minimap))
        .add_systems(Update, grid::draw_grid_gizmos) // draw grid        
        .add_systems(Update, (collision::move_with_collision_system, collision::sync_render_from_grid))
        .add_systems(Update, (camera::handle_spin_input, camera::animate_camera_spin))
        // .add_systems(Startup, spawn_asset.after(setup))
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut blocked: ResMut<world::Blocked>,
) {

    let yaw = 45.0;
    let pitch = 35.264; // classic isometric tilt
    let radius = 10.0; // distance from origin    

    // --- Camera: orthographic + isometric angle ---
    commands.spawn((
        Camera3d::default(),
        // Orthographic projection (no perspective)
        Projection::from(OrthographicProjection {
            // Keep a fixed vertical world size; tweak to your liking
            scaling_mode: ScalingMode::FixedVertical { viewport_height: 10.0 },
            ..OrthographicProjection::default_3d()
        }),
        camera::iso_camera_transform(yaw, pitch, radius),
        camera::IsoCamera { yaw_deg: yaw, pitch_deg: pitch, radius },
        camera::CameraSpin {
            start_yaw: yaw,
            end_yaw: yaw,
            t: 0.0,
            duration: 0.35,     // tweak for snappier/slower spin
            queued_steps: 0,
        },
        // Put camera on a diagonal and look at the origin.
        // Using equal XYZ like (10,10,10) gives a classic iso feel (~45° around Y, ~35.264° tilt).
        // Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Example walls (blocked cells)
    blocked.0.extend([
        (1, 0), (2, 1), (0, 2) // add any cells you want to be solid
    ]);

    // Visualize blocked cells
    for &(x, y) in &blocked.0 {
        let p = world::iso_world_from_grid(x, y, constants::TILE_W, constants::TILE_H);
        commands.spawn((
            world::Solid,
            Mesh3d(meshes.add(Cuboid::new(1.0, 0.4, 1.0))),
            MeshMaterial3d(materials.add(Color::srgb(0.7, 0.2, 0.2))),
            Transform::from_translation(p + Vec3::Y * 0.2),
        ));
    }

    // The movable player cube
    let start = world::GridPos { x: 0, y: 0 };
    let p = world::iso_world_from_grid(start.x, start.y, constants::TILE_W, constants::TILE_H);
    commands.spawn((
        start,
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_translation(p),
    ));

    // --- Quick scene to verify ---
    // commands.spawn((
    //     Mesh3d(meshes.add(Plane3d::default().mesh().size(8.0, 8.0))),
    //     MeshMaterial3d(materials.add(Color::srgb(0.25, 0.45, 0.3))),
    // ));    

    // Light
    commands.spawn((PointLight::default(), Transform::from_xyz(6.0, 10.0, 6.0)));
}

fn setup_minimap(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    // 1) Create a render texture (what the minimap camera will draw into)
    let width: u32 = 256;
    let height: u32 = 256;

    let image = Image::new_target_texture(width, height, TextureFormat::bevy_default());

    let image_handle = images.add(image);

    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("minimap_rt"),
            size: Extent3d { width, height, depth_or_array_layers: 1 },
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    // Transparent background (optional)
    image.resize(Extent3d { width, height, depth_or_array_layers: 1 });
    let rt_handle = images.add(image);

    // 2) Minimap camera: top-down orthographic
    //    - Look straight down from +Y onto the XZ plane.
    //    - Up vector = -Z so that +X is right and +Z is down on the minimap (Cartesian screen look).
    let ortho = OrthographicProjection {
        // Choose a fixed world height; bigger => more area visible
        scaling_mode: ScalingMode::FixedVertical { viewport_height: 20.0 },
        ..OrthographicProjection::default_3d()
    };
    commands.spawn((
        Camera3d::default(),
        Camera {
            // Render into the texture instead of the screen
            target: RenderTarget::Image(rt_handle.clone().into()),    
            ..default()
        },
        Projection::from(ortho),
        // Place camera above origin looking down
        Transform::from_translation(Vec3::new(0.0, 50.0, 0.001)) // small z offset to avoid singular up vector
            .looking_at(Vec3::ZERO, -Vec3::Z), // up = -Z gives Cartesian feel on the image
        // If you want to show extra overlays only on minimap:
        // RenderLayers::from_layers(&[0, 1]),
    ));

    // 3) UI: place the render texture in the corner
    commands.spawn((Node {
        width:Val::Px(180.0),
        height: Val::Px(180.0),        
        position_type: PositionType::Absolute,
        right: Val::Px(10.0),
        top: Val::Px(10.0),
        ..default()
    }))
    .with_children(|parent| {
        parent.spawn(ImageNode {            
            image: rt_handle,
            ..default()
        });
    });
}

// --- Separate system to load your Blender asset (.glb) ---
fn spawn_asset(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((        
        SceneRoot(asset_server.load("models/resource.glb#Scene0")),
        Transform::from_xyz(0.0, 1.0, 0.0),
    ));
}