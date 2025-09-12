use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (draw_cursor, display_pitch, rotate_camera))
        .run();
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

fn rotate_camera(
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
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
    let center = Vec3::ZERO; // Center point to rotate around

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
        let current_pos = camera_transform.translation - center;
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
        camera_transform.translation = center + new_pos;

        // Make camera look at center
        camera_transform.look_at(center, Vec3::Y);
    }
}

fn display_pitch(
    camera_query: Single<&Transform, With<Camera3d>>,
    mut gizmos: Gizmos,
    mut query: Single<&mut TextSpan>,
) {
    let camera_transform = *camera_query;
    let center = Vec3::ZERO;

    // Calculate current pitch angle
    let current_pos = camera_transform.translation - center;
    let distance = current_pos.length();
    let current_pitch = (current_pos.y / distance).asin();

    // Convert to degrees for easier reading
    let pitch_degrees = current_pitch.to_degrees();

    // Display the pitch angle as text using gizmos
    let text_pos = Vec3::new(-8.0, 8.0, 0.0);
    gizmos.ray(text_pos, Vec3::X * 0.1, Color::srgb(1.0, 0.0, 0.0));

    ***query = format!("{:.1}°", pitch_degrees);
}

#[derive(Component)]
struct Ground;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // initial communications, using stdin and stdout for now, later with tcp

    // Symbol Meaning
    // X Width or horizontal position
    // Y Height or vertical position
    // N Team name
    // q Quantity
    // R Incantation result
    // n Player number
    // M Message
    // O Orientation (N:1, E:2, S:3, O:4)
    // i Resource number
    // L Player level or incantation level
    // e Egg number
    // T Time unit

    // read one line that should be "BIENVENUE"
    // sends "GRAPHIC"
    // "msz X Y" is received (map size)
    // "sgt T" is received (time unit)
    // "bct X Y q q q q q q q q q" is received for each cell of the map
    // "tna N" is received for each team
    // "pnw #n X Y O L N" is received for each player
    // "enw #e X Y" is received for each egg
    println!("Setup scene");

    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(20., 20.))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Ground,
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 20.0, 15.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands
        .spawn(Text::new("Current pitch: "))
        .with_child(TextSpan::default());
}
