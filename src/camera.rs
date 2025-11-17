use bevy::{prelude::*, input::keyboard::KeyCode};
use crate::constants;
use crate::world;

/* ---------------- Camera state ---------------- */

#[derive(Component)]
pub struct IsoCamera {
    pub yaw_deg: f32,   // current yaw
    pub pitch_deg: f32, // ~35.264 for iso
    pub radius: f32,    // distance from target
}

#[derive(Component)]
pub struct MinimapCamera {
    pub height: f32, // how high above the world to render from                       
    pub center: Vec3 // what point to look at (usually Vec3::ZERO or player)
}


/// Handles in-progress spin + queued steps.
#[derive(Component)]
pub struct CameraSpin {
    pub start_yaw: f32,
    pub end_yaw: f32,
    pub t: f32,            // elapsed
    pub duration: f32,     // seconds for one 90° spin
    pub queued_steps: i32, // additional ±90° steps waiting
}

#[derive(Component)]
pub struct CameraFollow { 
    pub stiffness: f32, 
    pub damping: f32, 
    pub vel: Vec3 
}

/* ---------------- Input: queue spins ---------------- */

pub fn handle_spin_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut q: Query<(&IsoCamera, &mut CameraSpin)>,
) {
    let Ok((iso, mut spin)) = q.single_mut() else { panic!()};

    let mut steps: i32 = 0;
    if keys.just_pressed(KeyCode::KeyQ) { steps -= 1; }
    if keys.just_pressed(KeyCode::KeyE) { steps += 1; }
    if steps == 0 { return; }

    // If idle, start immediately; else queue.
    let spinning = spin.t < spin.duration;
    if !spinning {
        let step = steps.signum(); // take one step now
        spin.start_yaw = iso.yaw_deg;
        spin.end_yaw = iso.yaw_deg + 90.0 * step as f32;
        spin.t = 0.0;
        spin.queued_steps += steps - step;
    } else {
        spin.queued_steps += steps;
    }
}

/* ---------------- Animation ---------------- */

pub fn animate_camera_spin(
    time: Res<Time>,
    player_q: Query<(&world::GridPos), Without<world::Solid>>,
    mut q: Query<(&mut IsoCamera, &mut CameraSpin, &mut Transform)>,
) {
    let Ok((mut iso, mut spin, mut tform)) = q.single_mut() else { panic!() };    
    
    let target = {
        let gp = player_q.single().unwrap();
        world::grid_to_iso(gp.x, gp.y, constants::TILE_W, constants::TILE_H)
    };
    // If we’re mid-spin, advance it.
    if spin.t < spin.duration {
        spin.t += time.delta_secs();
        let alpha = (spin.t / spin.duration).clamp(0.0, 1.0);
        let eased = ease_in_out_cubic(alpha);

        let yaw = lerp_angle_deg(spin.start_yaw, spin.end_yaw, eased);
        iso.yaw_deg = yaw.rem_euclid(360.0);
        // *tform = iso_camera_transform(iso.yaw_deg, iso.pitch_deg, iso.radius);
        *tform = iso_camera_transform_at(target, iso.yaw_deg, iso.pitch_deg, iso.radius);


        // Finished this step?
        if alpha >= 1.0 {
            iso.yaw_deg = snap_to_quarter_turns(iso.yaw_deg);
            // *tform = iso_camera_transform(iso.yaw_deg, iso.pitch_deg, iso.radius);
            *tform = iso_camera_transform_at(target, iso.yaw_deg, iso.pitch_deg, iso.radius);

            // Launch next queued step if any.
            if spin.queued_steps != 0 {
                let step = spin.queued_steps.signum();
                spin.queued_steps -= step;
                spin.start_yaw = iso.yaw_deg;
                spin.end_yaw = iso.yaw_deg + 90.0 * step as f32;
                spin.t = 0.0;
            }
        }
    } 
}

/// Call this every frame (or whenever yaw changes):
/// - Keeps the minimap camera top-down (forward = -Y)
/// - Rotates its "up" around Y by the same yaw as the iso camera.
///   Using `up = rotate_y(-Z, yaw)` turns the minimap by the identical 90° steps.
pub fn sync_minimap_to_iso_yaw(
    player_q: Query<(&world::GridPos), Without<world::Solid>>,
    iso_q: Query<&IsoCamera>,
    mut mini_q: Query<(&MinimapCamera, &mut Transform)>,
) {
    let gp = player_q.single().unwrap();
    let iso = iso_q.single().unwrap();
    let yaw = (iso.yaw_deg - 45.0).to_radians();    

    // Up vector rotated around Y by yaw (start from -Z for Cartesian feel)
    let up = Quat::from_rotation_y(yaw) * -Vec3::Z;

    for (mini, mut t) in &mut mini_q {
        // Keep the camera straight above the center, looking down:        
        let eye = Vec3::new(mini.center.x, mini.height, mini.center.z);
        *t = Transform::from_translation(eye).looking_at(mini.center, up);
    }
}


pub fn follow_center_snap(
    player_q: Query<(&world::GridPos), Without<world::Solid>>,
    mut cam_q: Query<(&IsoCamera, &mut Transform)>,
) {
    let gp = player_q.single().unwrap();
    let target = world::grid_to_iso(gp.x, gp.y, constants::TILE_W, constants::TILE_H);

    for (iso, mut cam_tf) in &mut cam_q {
        *cam_tf = iso_camera_transform_at(target, iso.yaw_deg, iso.pitch_deg, iso.radius);
    }
}

pub fn follow_center_smooth(
    time: Res<Time>,    
    player_q: Query<(&world::GridPos), Without<world::Solid>>,
    mut cam_q: Query<(&IsoCamera, &mut Transform, &mut CameraFollow)>,
) {
    let dt = time.delta_secs();
    let gp = player_q.single().unwrap();
    let target = world::grid_to_iso(gp.x, gp.y, constants::TILE_W, constants::TILE_H);     
    println!("gp ~ {}, {}", gp.x, gp.y);
    let (iso, mut cam_tf, mut f) = cam_q.single_mut().unwrap();
    // desired camera position along iso orbit
    // let yaw = iso.yaw_deg.to_radians();
    // let pitch = iso.pitch_deg.to_radians();
    // let dir = Vec3::new(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos());
    let desired = target;

    // critically-damped spring to desired
    let k = f.stiffness;       // e.g. 20.0
    let c = f.damping;         // e.g. 2.0 * (k).sqrt() or just ~10.0
    let mut pos = cam_tf.translation;
    let mut vel = f.vel;

    let accel = (desired - pos) * k - vel * c;
    vel += accel * dt;
    // pos += vel * dt;

    f.vel = vel;
    // cam_tf.translation = pos;
    // cam_tf.look_at(target, Vec3::Y);
}

/* ---------------- Helpers ---------------- */

pub fn iso_camera_transform_at(target: Vec3, yaw_deg: f32, pitch_deg: f32, radius: f32) -> Transform {
    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians();
    let dir = Vec3::new(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos());
    Transform::from_translation(target + dir * radius).looking_at(target, Vec3::Y)
}

pub fn iso_camera_transform(yaw_deg: f32, pitch_deg: f32, radius: f32) -> Transform {
    let yaw = yaw_deg.to_radians();
    let pitch = pitch_deg.to_radians();
    let dir = Vec3::new(yaw.cos() * pitch.cos(), pitch.sin(), yaw.sin() * pitch.cos());
    Transform::from_translation(dir * radius).looking_at(Vec3::ZERO, Vec3::Y)
}

/// Eases the motion (0..1 -> 0..1).
fn ease_in_out_cubic(t: f32) -> f32 {
    if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
}

/// Shortest-arc angle lerp in degrees.
fn lerp_angle_deg(a: f32, b: f32, t: f32) -> f32 {
    let mut delta = (b - a) % 360.0;
    if delta > 180.0 { delta -= 360.0; }
    if delta < -180.0 { delta += 360.0; }
    a + delta * t
}

/// Snap to exact 45° + n·90° if you want clean quadrants (optional).
fn snap_to_quarter_turns(yaw: f32) -> f32 {
    // Base at 45°, step 90°
    let rel = yaw - 45.0;
    let snapped = (rel / 90.0).round() * 90.0 + 45.0;
    snapped.rem_euclid(360.0)
}