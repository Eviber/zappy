use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

use super::TILE_SIZE;

use super::MapSize;

const ZOOM_SPEED: f32 = 1.0;
const MIN_CAMERA_DISTANCE: f32 = 5.0;
const MAX_CAMERA_DISTANCE: f32 = 100.0;
const MIN_PITCH_ANGLE: f32 = 10f32.to_radians();
const MAX_PITCH_ANGLE: f32 = 80f32.to_radians();
const BASE_ROTATION_SENSITIVITY: f32 = 2.0;

/// Plugin to handle user input for camera control
pub(crate) struct UserInputPlugin;

impl Plugin for UserInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (zoom_camera, rotate_camera));
    }
}

/// Update the camera distance with the scroll
pub(crate) fn zoom_camera(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    map_size: Res<MapSize>,
) {
    let delta_x = map_size.width as f32 * TILE_SIZE / 2. - TILE_SIZE / 2.;
    let delta_y = map_size.height as f32 * TILE_SIZE / 2. - TILE_SIZE / 2.;
    let center: Vec3 = Vec3 {
        x: delta_x,
        y: 0.,
        z: delta_y,
    };
    for event in scroll_events.read() {
        let scroll_amount = -event.y;
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

pub(crate) fn rotate_camera(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    camera_query: Single<&mut Transform, With<Camera3d>>,
    windows: Query<&Window>,
    map_size: Res<MapSize>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    // Only rotate when left mouse button is pressed
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let delta_x = map_size.width as f32 * TILE_SIZE / 2. - TILE_SIZE / 2.;
    let delta_y = map_size.height as f32 * TILE_SIZE / 2. - TILE_SIZE / 2.;
    let center: Vec3 = Vec3 {
        x: delta_x,
        y: 0.,
        z: delta_y,
    };
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
