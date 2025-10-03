mod server_communication;

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use server_communication::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ServerCommunicationPlugin)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                draw_cursor,
                rotate_camera,
                zoom_camera,
                draw_grid,
                draw_axes,
                update_map_size,
                update_game_tick,
                update_tile_content,
                add_team,
                add_player,
            ),
        )
        .run();
}

/// Draw 3D axes of the players
fn draw_axes(mut gizmos: Gizmos, query: Query<(&GlobalTransform,), With<Mesh3d>>) {
    for (transform,) in &query {
        let length = 1.5;
        gizmos.axes(*transform, length);
    }
}

fn draw_grid(ground: Single<&GlobalTransform, With<Ground>>, mut gizmos: Gizmos) {
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

fn draw_cursor(
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

const CENTER: Vec3 = Vec3 {
    x: 4. * 5. / 2. - 2.5,
    y: 0.,
    z: 4. * 5. / 2. - 2.5,
};

/// Update the camera distance with the scroll
fn zoom_camera(
    mut scroll_events: MessageReader<MouseWheel>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    for event in scroll_events.read() {
        let scroll_amount = -event.y;
        let direction = (camera.translation - CENTER).normalize();
        let zoom_speed = 1.0;
        camera.translation += direction * scroll_amount * zoom_speed;
        // Ensure the camera doesn't get too close or too far
        let min_distance = 5.0;
        let max_distance = 100.0;
        let current_distance = (camera.translation - CENTER).length();
        if current_distance < min_distance {
            camera.translation = CENTER + direction * min_distance;
        } else if current_distance > max_distance {
            camera.translation = CENTER + direction * max_distance;
        }
    }
}

fn rotate_camera(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    camera_query: Single<&mut Transform, With<Camera3d>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };

    // Only rotate when left mouse button is pressed
    if !mouse_input.pressed(MouseButton::Left) {
        return;
    }

    let mut camera_transform = camera_query;

    let min_pitch = 10f32.to_radians();
    let max_pitch = 80f32.to_radians();

    // Process all mouse motion events this frame
    for motion in mouse_motion.read() {
        // Scale sensitivity based on window size
        let base_sensitivity = 2.0;
        let sensitivity = base_sensitivity / window.width().min(window.height());
        let yaw_delta = -motion.delta.x * sensitivity;
        let pitch_delta = motion.delta.y * sensitivity;

        // Get current position relative to center
        let current_pos = camera_transform.translation - CENTER;
        let distance = current_pos.length();

        // Calculate current pitch angle (angle from horizontal plane)
        let current_pitch = (current_pos.y / distance).asin();

        // Clamp the new pitch angle within bounds
        let new_pitch = (current_pitch + pitch_delta).clamp(min_pitch, max_pitch);
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
        camera_transform.translation = CENTER + new_pos;

        // Make camera look at center
        camera_transform.look_at(CENTER, Vec3::Y);
    }
}

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let map_size = MapSize {
        width: 0,
        height: 0,
    };
    commands.insert_resource(map_size);
    commands.insert_resource(TimeUnit(0));

    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh())),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::default(),
        Ground,
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((Camera3d::default(), Transform::default()));
}

#[derive(Clone, Copy, Debug, Resource)]
struct MapSize {
    width: usize,
    height: usize,
}

#[allow(dead_code)]
#[derive(Resource)]
struct TimeUnit(u32);

const TILE_SIZE: f32 = 5.0;

fn update_map_size(
    mut reader: MessageReader<UpdateMapSize>,
    mut map_size: ResMut<MapSize>,
    mut ground: Single<(&mut Transform, &mut Mesh3d), With<Ground>>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<Ground>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for msg in reader.read() {
        info!("Map size updated: {}x{}", msg.width, msg.height);
        map_size.width = msg.width;
        map_size.height = msg.height;
        let plane_width = map_size.width as f32 * TILE_SIZE;
        let plane_height = map_size.height as f32 * TILE_SIZE;
        let delta_x = plane_width / 2. - TILE_SIZE / 2.;
        let delta_y = plane_height / 2. - TILE_SIZE / 2.;
        *ground.1 = Mesh3d(meshes.add(Plane3d::default().mesh().size(plane_width, plane_height)));
        ground.0.translation = Vec3::new(delta_x, 0.0, delta_y);
        // reposition camera to still look at center of the map
        let initial_distance = (delta_x.powi(2) + delta_y.powi(2)).sqrt() + 5.0;
        let initial_height = initial_distance * (45f32.to_radians().sin());
        let initial_horizontal_distance = initial_distance * (45f32.to_radians().cos());
        let initial_position = Vec3::new(
            delta_x + initial_horizontal_distance / (2f32).sqrt(),
            initial_height,
            delta_y + initial_horizontal_distance / (2f32).sqrt(),
        );
        camera.translation = initial_position;
        camera.look_at(CENTER, Vec3::Y);
    }
}

fn update_game_tick(mut reader: MessageReader<UpdateGameTick>, mut time_unit: ResMut<TimeUnit>) {
    for msg in reader.read() {
        info!("Game tick updated: {}", msg.0);
        time_unit.0 = msg.0;
    }
}

fn update_tile_content(mut reader: MessageReader<UpdateTileContent>) {
    for msg in reader.read() {
        info!("Tile ({}, {}) resources: {:?}", msg.x, msg.y, msg.resources);
    }
}

fn add_team(mut reader: MessageReader<TeamName>) {
    for msg in reader.read() {
        info!("Team name: {}", msg.0);
    }
}

fn add_player(
    mut reader: MessageReader<NewPlayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for msg in reader.read() {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 1.5, 0.8).mesh())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            Transform::from_translation(Vec3::new(msg.x as f32 * 5., 0.75, msg.y as f32 * 5.)),
        ));
        info!("Added player #{}", msg.id);
    }
}
