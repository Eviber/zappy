use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                draw_cursor,
                display_pitch,
                rotate_camera,
                zoom_camera,
                draw_grid,
                draw_axes,
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
        Vec2::splat(5.0),
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
    mut scroll_events: EventReader<MouseWheel>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    for event in scroll_events.read() {
        let scroll_amount = -event.y;
        let direction = (camera.translation - CENTER).normalize();
        let zoom_speed = 0.5;
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

#[allow(dead_code)]
#[derive(Debug, Resource)]
struct GameState {
    map_size: (u32, u32),
    time_unit: u32,
    teams: Vec<String>,
    players: Vec<Player>,
    eggs: Vec<Egg>,
    resources: Vec<Vec<u32>>, // 2D array for resources in each cell
}

#[allow(dead_code)]
#[derive(Debug)]
struct Player {
    id: u32,
    position: (u32, u32),
    orientation: u8,
    level: u8,
    team: String,
}

#[allow(dead_code)]
#[derive(Debug)]
struct Egg {
    id: u32,
    position: (u32, u32),
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let game_state = get_initial_game_state();
    // eprintln!("Initial game state: {:#?}", game_state);

    // commands.insert_resource(game_state);

    let delta_x = game_state.map_size.0 as f32 * 5. / 2.;
    let delta_y = game_state.map_size.1 as f32 * 5. / 2.;

    for player in &game_state.players {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 1.5, 0.8).mesh())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            Transform::from_translation(Vec3::new(
                player.position.0 as f32 * 5.,
                0.75,
                player.position.1 as f32 * 5.,
            )),
        ));
    }

    // sphere at the center
    // commands.spawn((
    //     Mesh3d(meshes.add(Sphere::default().mesh())),
    //     MeshMaterial3d(materials.add(Color::srgb(0.2, 0.2, 0.8))),
    //     Transform::from_translation(CENTER),
    // ));

    // plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::default().mesh().size(
            game_state.map_size.0 as f32 * 5.,
            game_state.map_size.1 as f32 * 5.,
        ))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3))),
        Transform::from_xyz(delta_x - 2.5, 0.0, delta_y - 2.5),
        Ground,
    ));

    // light
    commands.spawn((
        DirectionalLight::default(),
        Transform::from_translation(Vec3::ONE).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    // positioned to look at CENTER with a pitch of 45 degrees from the bottom right corner of the
    // map with a distance that ensures the whole map is visible (so add some padding)
    let initial_distance = (delta_x.powi(2) + delta_y.powi(2)).sqrt() + 5.0;
    let initial_height = initial_distance * (45f32.to_radians().sin());
    let initial_horizontal_distance = initial_distance * (45f32.to_radians().cos());
    let initial_position = Vec3::new(
        delta_x + initial_horizontal_distance / (2f32).sqrt(),
        initial_height,
        delta_y + initial_horizontal_distance / (2f32).sqrt(),
    );
    commands.spawn((
        Camera3d::default(),
        Transform::from_translation(initial_position).looking_at(CENTER, Vec3::Y),
    ));

    commands
        .spawn(Text::new("Current pitch: "))
        .with_child(TextSpan::default());
}

fn read_line(line: &mut String) {
    line.clear();
    std::io::stdin().read_line(line).unwrap();
}

fn get_initial_game_state() -> GameState {
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
    println!("GRAPHIC");
    // read from stdin and parse the initial game state
    let mut line = String::new();
    read_line(&mut line);
    if line.trim() != "BIENVENUE" {
        panic!("Expected BIENVENUE, got {}", line);
    }
    read_line(&mut line);
    if !line.starts_with("msz") {
        panic!("Expected msz, got {}", line);
    }
    let msz_parts: Vec<&str> = line.split_whitespace().collect();
    if msz_parts.len() != 3 {
        panic!("Invalid msz format");
    }
    let map_size = (
        msz_parts[1].parse::<u32>().unwrap(),
        msz_parts[2].parse::<u32>().unwrap(),
    );
    read_line(&mut line);
    if !line.starts_with("sgt") {
        panic!("Expected sgt, got {}", line);
    }
    let sgt_parts: Vec<&str> = line.split_whitespace().collect();
    if sgt_parts.len() != 2 {
        panic!("Invalid sgt format");
    }
    let time_unit = sgt_parts[1].parse::<u32>().unwrap();
    let mut resources = vec![vec![0; 7]; (map_size.0 * map_size.1) as usize];
    let mut teams = Vec::new();
    let mut players = Vec::new();
    let mut eggs = Vec::new();
    // read bct lines
    for _ in 0..(map_size.0 * map_size.1) {
        read_line(&mut line);
        if !line.starts_with("bct") {
            panic!("Expected bct, got {}", line);
        }
        let bct_parts: Vec<&str> = line.split_whitespace().collect();
        if bct_parts.len() != 10 {
            panic!("Invalid bct format");
        }
        let x = bct_parts[1].parse::<u32>().unwrap();
        let y = bct_parts[2].parse::<u32>().unwrap();
        let idx = (y * map_size.0 + x) as usize;
        for i in 0..7 {
            resources[idx][i] = bct_parts[3 + i].parse::<u32>().unwrap();
        }
    }
    // read tna lines until we get a line that doesn't start with tna
    read_line(&mut line);
    while line.starts_with("tna") {
        let tna_parts: Vec<&str> = line.split_whitespace().collect();
        if tna_parts.len() != 2 {
            panic!("Invalid tna format");
        }
        teams.push(tna_parts[1].to_string());
        read_line(&mut line);
    }
    // read pnw lines until we get a line that doesn't start with pnw
    while line.starts_with("pnw") {
        let pnw_parts: Vec<&str> = line.split_whitespace().collect();
        if pnw_parts.len() != 7 {
            panic!("Invalid pnw format");
        }
        let player = Player {
            id: pnw_parts[1].trim_start_matches('#').parse::<u32>().unwrap(),
            position: (
                pnw_parts[2].parse::<u32>().unwrap(),
                pnw_parts[3].parse::<u32>().unwrap(),
            ),
            orientation: pnw_parts[4].parse::<u8>().unwrap(),
            level: pnw_parts[5].parse::<u8>().unwrap(),
            team: pnw_parts[6].to_string(),
        };
        players.push(player);
        read_line(&mut line);
    }
    // read enw lines until we get a line that doesn't start with enw
    while line.starts_with("enw") {
        let enw_parts: Vec<&str> = line.split_whitespace().collect();
        if enw_parts.len() != 4 {
            panic!("Invalid enw format");
        }
        let egg = Egg {
            id: enw_parts[1].trim_start_matches('#').parse::<u32>().unwrap(),
            position: (
                enw_parts[2].parse::<u32>().unwrap(),
                enw_parts[3].parse::<u32>().unwrap(),
            ),
        };
        eggs.push(egg);
        read_line(&mut line);
    }
    GameState {
        map_size,
        time_unit,
        teams,
        players,
        eggs,
        resources,
    }
}
