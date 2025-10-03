use super::*;
use bevy::prelude::*;

mod server_communication;
use server_communication::*;

/// Plugin to handle messages from the server
pub(crate) struct ServerMessageHandlersPlugin;

impl Plugin for ServerMessageHandlersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileStacks::default());
        app.add_plugins(ServerCommunicationPlugin);
        app.add_systems(
            Update,
            (
                update_map_size,
                update_game_tick,
                update_tile_content,
                add_team,
                add_player,
            ),
        );
    }
}

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
        let center: Vec3 = Vec3 {
            x: delta_x,
            y: 0.,
            z: delta_y,
        };
        camera.look_at(center, Vec3::Y);
    }
}

fn update_game_tick(mut reader: MessageReader<UpdateGameTick>, mut time_unit: ResMut<TimeUnit>) {
    for msg in reader.read() {
        info!("Game tick updated: {}", msg.0);
        time_unit.0 = msg.0;
    }
}

#[derive(Clone, Copy, Component)]
enum Item {
    Nourriture,
    Linemate,
    Deraumère,
    Sibur,
    Mendiane,
    Phiras,
    Thystame,
}

impl Item {
    fn from_index(index: usize) -> Self {
        match index {
            0 => Item::Nourriture,
            1 => Item::Linemate,
            2 => Item::Deraumère,
            3 => Item::Sibur,
            4 => Item::Mendiane,
            5 => Item::Phiras,
            6 => Item::Thystame,
            _ => panic!("Invalid resource index"),
        }
    }

    fn color(self) -> Color {
        match self {
            Item::Nourriture => Color::srgb(0.8, 0.8, 0.2),
            Item::Linemate => Color::srgb(0.5, 0.5, 0.5),
            Item::Deraumère => Color::srgb(0.2, 0.8, 0.2),
            Item::Sibur => Color::srgb(0.2, 0.2, 0.8),
            Item::Mendiane => Color::srgb(0.8, 0.2, 0.8),
            Item::Phiras => Color::srgb(0.5, 0.2, 0.2),
            Item::Thystame => Color::srgb(0.2, 0.8, 0.8),
        }
    }

    fn delta_vec(self) -> Vec3 {
        // around the center of the tile, using TILE_SIZE
        let delta = TILE_SIZE / 4.;
        match self {
            Item::Nourriture => Vec3::new(-delta, 0., -delta),
            Item::Linemate => Vec3::new(0., 0., -delta),
            Item::Deraumère => Vec3::new(delta, 0., -delta),
            Item::Sibur => Vec3::new(-delta, 0., 0.),
            Item::Mendiane => Vec3::new(delta, 0., 0.),
            Item::Phiras => Vec3::new(-delta, 0., delta),
            Item::Thystame => Vec3::new(delta, 0., delta),
        }
    }
}

#[derive(Resource, Default)]
struct TileStacks(std::collections::HashMap<(usize, usize), [Vec<Entity>; 7]>);

fn update_tile_content(
    mut reader: MessageReader<UpdateTileContent>,
    mut commands: Commands,
    mut stacks: ResMut<TileStacks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for msg in reader.read() {
        info!("Tile ({}, {}) resources: {:?}", msg.x, msg.y, msg.items);
        let tile_pos = (msg.x, msg.y);
        let stack = stacks.0.entry(tile_pos).or_default();
        // Remove existing entities
        for entity in stack.iter_mut().flat_map(|v| v.drain(..)) {
            commands.entity(entity).despawn();
        }
        // Add new resources
        for (index, &count) in msg.items.iter().enumerate() {
            let resource_type = Item::from_index(index);
            for _ in 0..count {
                let delta = resource_type.delta_vec();
                let entity = commands
                    .spawn((
                        resource_type,
                        Transform::from_translation(
                            delta
                                + Vec3::new(
                                    msg.x as f32 * TILE_SIZE,
                                    0.1 + stack[index].len() as f32 * 0.15,
                                    msg.y as f32 * TILE_SIZE,
                                ),
                        ),
                        Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.2).mesh())),
                        MeshMaterial3d(materials.add(resource_type.color())),
                    ))
                    .id();
                stack[index].push(entity);
            }
        }
    }
}

fn add_team(mut reader: MessageReader<TeamName>) {
    for msg in reader.read() {
        info!("Team name: {}", msg.0);
    }
}

#[derive(Component)]
struct Level(u32);

#[derive(Component)]
struct Team(String);

fn add_player(
    mut reader: MessageReader<NewPlayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for msg in reader.read() {
        let rotation = match msg.orientation {
            1 => Quat::from_rotation_y(0.),                           // North
            2 => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2), // East
            3 => Quat::from_rotation_y(std::f32::consts::PI),         // South
            4 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),  // West
            _ => panic!("Invalid orientation"),
        };
        let transform = Transform {
            translation: Vec3::new(msg.x as f32 * TILE_SIZE, 0.75, msg.y as f32 * TILE_SIZE),
            rotation,
            ..Default::default()
        };
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.8, 1.5, 0.8).mesh())),
            MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
            transform,
            Player,
            Level(msg.level),
            Team(msg.team.clone()),
        ));
        info!("Added player #{}", msg.id);
    }
}
