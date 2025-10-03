use super::*;
use bevy::prelude::*;

mod server_communication;
use server_communication::*;

/// Plugin to handle messages from the server
pub(crate) struct ServerMessageHandlersPlugin;

impl Plugin for ServerMessageHandlersPlugin {
    fn build(&self, app: &mut App) {
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

pub(crate) fn update_map_size(
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

pub(crate) fn update_game_tick(
    mut reader: MessageReader<UpdateGameTick>,
    mut time_unit: ResMut<TimeUnit>,
) {
    for msg in reader.read() {
        info!("Game tick updated: {}", msg.0);
        time_unit.0 = msg.0;
    }
}

pub(crate) fn update_tile_content(mut reader: MessageReader<UpdateTileContent>) {
    for msg in reader.read() {
        info!("Tile ({}, {}) resources: {:?}", msg.x, msg.y, msg.resources);
    }
}

pub(crate) fn add_team(mut reader: MessageReader<TeamName>) {
    for msg in reader.read() {
        info!("Team name: {}", msg.0);
    }
}

pub(crate) fn add_player(
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
            Player,
        ));
        info!("Added player #{}", msg.id);
    }
}
