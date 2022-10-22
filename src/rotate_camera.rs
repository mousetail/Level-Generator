use bevy::ecs::event::{Events, ManualEventReader};
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Default)]
struct InputState {
    reader_motion: ManualEventReader<MouseMotion>,
    pitch: f32,
    yaw: f32,
}

/// Mouse sensitivity and movement speed
pub struct MovementSettings {
    pub sensitivity: f32,
    pub speed: f32,
}

impl Default for MovementSettings {
    fn default() -> Self {
        Self {
            sensitivity: 0.00012,
            speed: 0.3,
        }
    }
}

struct GroundedInfo {
    normal: Vec3,
    grounded: bool,
}

/// A marker component used in queries when you want flycams and not other cameras
#[derive(Component)]
pub struct FlyCam;

/// Grabs/ungrabs mouse cursor
fn toggle_grab_cursor(window: &mut Window) {
    window.set_cursor_lock_mode(!window.cursor_locked());
    window.set_cursor_visibility(!window.cursor_visible());
}

/// Grabs the cursor when game first starts
fn initial_grab_cursor(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        toggle_grab_cursor(window);
    } else {
        warn!("Primary window not found for `initial_grab_cursor`!");
    }
}

/// Spawns the `Camera3dBundle` to be controlled
// fn setup_player(mut commands: Commands) {
//     commands
//         .spawn_bundle(Camera3dBundle {
//             transform: Transform::from_xyz(-2.0, 5.0, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
//             ..Default::default()
//         })
//         .insert(FlyCam);
// }
fn is_grounted(
    context: &Res<RapierContext>,
    start_pos: Vec3,
    player_handle: Entity,
) -> GroundedInfo {
    match context.cast_ray_and_get_normal(
        start_pos,
        -Vec3::Y,
        1.1,
        true,
        QueryFilter::exclude_dynamic()
            .exclude_sensors()
            .exclude_rigid_body(player_handle),
    ) {
        Some(ray_info) => GroundedInfo {
            grounded: true,
            normal: ray_info.1.normal,
        },
        None => GroundedInfo {
            grounded: false,
            normal: Vec3::Y,
        },
    }
}

// / Handles keyboard input and movement
fn player_move(
    keys: Res<Input<KeyCode>>,
    windows: Res<Windows>,
    settings: Res<MovementSettings>,
    mut query: Query<(
        &Transform,
        &mut ExternalImpulse,
        &Children,
        Entity,
        &mut Velocity,
    )>,
    q_child: Query<&Transform, With<FlyCam>>,
    rapier_context: Res<RapierContext>,
) {
    if let Some(window) = windows.get_primary() {
        for mut transform in query.iter_mut() {
            let grounded = is_grounted(&rapier_context, transform.0.translation, transform.3);

            for &child in transform.2 {
                match q_child.get(child) {
                    Ok(child) => {
                        let mut velocity = Vec3::ZERO;
                        let local_z = child.local_z();
                        let forward = -Vec3::new(local_z.x, 0., local_z.z);
                        let right = Vec3::new(local_z.z, 0., -local_z.x);
                        for key in keys.get_pressed() {
                            if window.cursor_locked() {
                                match key {
                                    KeyCode::W => velocity += forward,
                                    KeyCode::S => velocity -= forward,
                                    KeyCode::A => velocity -= right,
                                    KeyCode::D => velocity += right,
                                    KeyCode::Space => {
                                        if grounded.grounded {
                                            transform.4.linvel += Vec3::Y * 2.
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        velocity = Quat::from_rotation_arc(Vec3::Y, grounded.normal)
                            * velocity.normalize_or_zero()
                            + Vec3::Y * 0.01;
                        transform.1.impulse =
                            velocity * settings.speed * if grounded.grounded { 1.0 } else { 0.5 };
                    }
                    Err(_) => (),
                }
            }
        }
    } else {
        warn!("Primary window not found for `player_move`!");
    }
}

/// Handles looking around if cursor is locked
fn player_look(
    settings: Res<MovementSettings>,
    windows: Res<Windows>,
    mut state: ResMut<InputState>,
    motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut Transform, With<FlyCam>>,
) {
    if let Some(window) = windows.get_primary() {
        let mut delta_state = state.as_mut();
        for mut transform in query.iter_mut() {
            for ev in delta_state.reader_motion.iter(&motion) {
                if window.cursor_locked() {
                    // Using smallest of height or width ensures equal vertical and horizontal sensitivity
                    let window_scale = window.height().min(window.width());
                    delta_state.pitch -=
                        (settings.sensitivity * ev.delta.y * window_scale).to_radians();
                    delta_state.yaw -=
                        (settings.sensitivity * ev.delta.x * window_scale).to_radians();
                }

                delta_state.pitch = delta_state.pitch.clamp(-1.54, 1.54);

                // Order is important to prevent unintended roll
                transform.rotation = Quat::from_axis_angle(Vec3::Y, delta_state.yaw)
                    * Quat::from_axis_angle(Vec3::X, delta_state.pitch);
            }
        }
    } else {
        warn!("Primary window not found for `player_look`!");
    }
}

fn cursor_grab(keys: Res<Input<KeyCode>>, mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        if keys.just_pressed(KeyCode::Escape) {
            toggle_grab_cursor(window);
        }
    } else {
        warn!("Primary window not found for `cursor_grab`!");
    }
}

/// Contains everything needed to add first-person fly camera behavior to your game
pub struct PlayerPlugin;
impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputState>()
            .init_resource::<MovementSettings>()
            // .add_startup_system(setup_player)
            .add_startup_system(initial_grab_cursor)
            .add_system(player_move)
            .add_system(player_look)
            .add_system(cursor_grab);
    }
}
