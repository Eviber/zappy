use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use super::TILE_SIZE;

use super::MapSize;

const ZOOM_SPEED: f32 = 1.0;
const MOVE_SPEED: f32 = 50.0;
const MIN_CAMERA_DISTANCE: f32 = 5.0;
const MAX_CAMERA_DISTANCE: f32 = 100.0;
const MIN_PITCH_ANGLE: f32 = 10f32.to_radians();
const MAX_PITCH_ANGLE: f32 = 80f32.to_radians();
const BASE_ROTATION_SENSITIVITY: f32 = 2.0;

/// Plugin to handle user input for camera control
pub(crate) struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CameraFocus(Vec3::ZERO));
        app.add_systems(
            Update,
            (
                update_camera_focus_on_map_size_change,
                zoom_camera,
                rotate_camera,
                translate_camera,
            ),
        );
    }
}

fn update_camera_focus_on_map_size_change(map_size: Res<MapSize>, mut focus: ResMut<CameraFocus>) {
    if !map_size.is_changed() {
        return;
    }
    let map_size: &MapSize = &map_size;
    let delta_x = map_size.width as f32 * TILE_SIZE / 2. - TILE_SIZE / 2.;
    let delta_y = map_size.height as f32 * TILE_SIZE / 2. - TILE_SIZE / 2.;
    focus.0 = Vec3 {
        x: delta_x,
        y: 0.,
        z: delta_y,
    };
}

#[derive(Resource)]
struct CameraFocus(Vec3);

fn translate_camera(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_query: Single<&mut Transform, With<Camera3d>>,
    mut focus: ResMut<CameraFocus>,
    map_size: Res<MapSize>,
    time: Res<Time>,
) {
    let mut direction = Vec3::ZERO;
    if keyboard.pressed(KeyCode::KeyW) {
        direction += Vec3::Z;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        direction += Vec3::Z * -1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        direction += Vec3::X * -1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        direction += Vec3::X;
    }

    // No movement
    if direction == Vec3::ZERO {
        return;
    }

    direction = direction.normalize();
    let mut camera_forward: Vec3 = camera_query.forward().into();
    let mut camera_right: Vec3 = camera_query.right().into();
    camera_forward.y = 0.0;
    camera_forward = camera_forward.normalize();
    camera_right.y = 0.0;
    camera_right = camera_right.normalize();
    direction = (camera_forward * direction.z + camera_right * direction.x).normalize();
    let mut movement = direction * MOVE_SPEED * time.delta_secs();
    let future_focus = focus.0 + movement;
    let min_x = 0.0;
    let max_x = map_size.width as f32 * TILE_SIZE;
    let min_z = 0.0;
    let max_z = map_size.height as f32 * TILE_SIZE;
    if future_focus.x < min_x {
        movement.x += min_x - future_focus.x;
    } else if future_focus.x > max_x {
        movement.x += max_x - future_focus.x;
    }
    if future_focus.z < min_z {
        movement.z += min_z - future_focus.z;
    } else if future_focus.z > max_z {
        movement.z += max_z - future_focus.z;
    }
    camera_query.translation += movement;
    focus.0 += movement;
}

/// Update the camera distance with the scroll
fn zoom_camera(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    focus: Res<CameraFocus>,
) {
    let center = focus.0;
    for event in scroll_events.read() {
        let scroll_amount = -event.y;
        debug_assert_ne!(camera.translation, center);
        let direction = (camera.translation - center).normalize();
        let zoom_speed = ZOOM_SPEED;
        camera.translation += direction * scroll_amount * zoom_speed;
        // Ensure the camera doesn't get too close or too far
        let min_distance = MIN_CAMERA_DISTANCE;
        let max_distance = MAX_CAMERA_DISTANCE;
        let current_distance = (camera.translation - center).length();
        if current_distance < min_distance {
            camera.translation = center + direction * min_distance;
        } else if current_distance > max_distance {
            camera.translation = center + direction * max_distance;
        }
    }
}

fn rotate_camera(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    camera_query: Single<&mut Transform, With<Camera3d>>,
    windows: Query<&Window>,
    focus: Res<CameraFocus>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    // Only rotate when left mouse button is pressed
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let center = focus.0;
    let mut camera_transform = camera_query;

    // Process all mouse motion events this frame
    for motion in mouse_motion.read() {
        // Scale sensitivity based on window size
        let base_sensitivity = BASE_ROTATION_SENSITIVITY;
        let sensitivity = base_sensitivity / window.width().min(window.height());
        let yaw_delta = -motion.delta.x * sensitivity;
        let pitch_delta = motion.delta.y * sensitivity;

        // Get current position relative to center
        let current_pos = camera_transform.translation - center;
        let distance = current_pos.length();

        debug_assert!(distance > f32::EPSILON);

        // Calculate current pitch angle (angle from horizontal plane)
        let current_pitch = (current_pos.y / distance).asin();

        // Clamp the new pitch angle within bounds
        let new_pitch = (current_pitch + pitch_delta).clamp(MIN_PITCH_ANGLE, MAX_PITCH_ANGLE);
        let actual_pitch_delta = current_pitch - new_pitch;

        // Apply yaw rotation (around world Y axis) - no clamping needed
        let yaw_rotation = Quat::from_rotation_y(yaw_delta);
        let pos_after_yaw = yaw_rotation * current_pos;

        // Apply pitch rotation (around camera's local X axis) with clamping
        let pitch_axis = camera_transform.local_x();
        let pitch_rotation = Quat::from_axis_angle(*pitch_axis, actual_pitch_delta);
        let new_pos = pitch_rotation * pos_after_yaw;

        // Ensure we maintain the same distance from center
        let new_pos = new_pos.normalize() * distance;

        // Update camera position
        camera_transform.translation = center + new_pos;

        // Make camera look at center
        camera_transform.look_at(center, Vec3::Y);
    }
}
