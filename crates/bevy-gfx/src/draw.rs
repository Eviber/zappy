use super::TILE_SIZE;
use bevy::prelude::*;

use super::Ground;

pub(crate) struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (axes, grid, cursor));
    }
}

/// Draw 3D axes of the players
pub(crate) fn axes(mut gizmos: Gizmos, query: Query<(&GlobalTransform,), With<Mesh3d>>) {
    for (transform,) in &query {
        let length = 1.5;
        gizmos.axes(*transform, length);
    }
}

pub(crate) fn grid(ground: Single<&GlobalTransform, With<Ground>>, mut gizmos: Gizmos) {
    gizmos.grid(
        Isometry3d::new(
            ground.translation(),
            Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
        ),
        UVec2::new(8, 8),
        Vec2::splat(TILE_SIZE),
        LinearRgba::gray(0.6),
    );
}

pub(crate) fn cursor(
    camera_query: Single<(&Camera, &GlobalTransform)>,
    ground: Single<&GlobalTransform, With<Ground>>,
    windows: Query<&Window>,
    mut gizmos: Gizmos,
) {
    let Ok(windows) = windows.single() else {
        return;
    };

    let (camera, camera_transform) = *camera_query;

    let Some(cursor_position) = windows.cursor_position() else {
        return;
    };

    // Calculate a ray pointing from the camera into the world based on the cursor's position.
    let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_position) else {
        return;
    };

    // Calculate if and where the ray is hitting the ground plane.
    let Some(distance) =
        ray.intersect_plane(ground.translation(), InfinitePlane3d::new(ground.up()))
    else {
        return;
    };
    let point = ray.get_point(distance);

    // Draw a circle just above the ground plane at that position.
    gizmos.circle(
        Isometry3d::new(
            point + ground.up() * 0.01,
            Quat::from_rotation_arc(Vec3::Z, ground.up().as_vec3()),
        ),
        0.2,
        Color::WHITE,
    );
}
