use crate::server_message_handlers::HoverInfo;
use bevy::prelude::*;

use super::TILE_SIZE;

use super::Ground;
use super::Player;

pub(crate) struct DrawPlugin;

impl Plugin for DrawPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MeshPickingPlugin);
        app.add_systems(Startup, setup);
        app.add_systems(Update, (axes, grid, cursor, draw_player_info));
    }
}

#[derive(Component)]
struct PlayerInfoText;

/// Setup the info text
fn setup(mut commands: Commands) {
    // Spawn UI overlay text in top-left
    commands.spawn((
            Node {
                ..default()
            },
            Text::new(""),
            TextColor(Color::BLACK),
            PlayerInfoText,
    ));
}

/// Draw 3D axes of the players
fn axes(mut gizmos: Gizmos, query: Query<(&GlobalTransform,), With<Player>>) {
    for (transform,) in &query {
        let length = 1.5;
        gizmos.axes(*transform, length);
    }
}

/// Draw a grid on the ground
fn grid(ground: Single<&GlobalTransform, With<Ground>>, mut gizmos: Gizmos) {
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

/// Draw a cursor on the ground where the mouse is pointing
fn cursor(
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

/// Draws the player info UI when hovering over a player
fn draw_player_info(
    hovered_player: Option<Res<HoverInfo>>,
    mut text: Single<&mut Text, With<PlayerInfoText>>,
) {
    if let Some(info) = hovered_player {
        text.0 = info.0.clone();
    } else {
        text.0 = "".to_string();
    }
}
