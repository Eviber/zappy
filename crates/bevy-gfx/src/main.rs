mod draw;
mod server_message_handlers;
mod user_input;

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Resource)]
struct MapSize {
    width: usize,
    height: usize,
}

#[allow(dead_code)]
#[derive(Resource)]
struct TimeUnit(u32);

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct Player;

const TILE_SIZE: f32 = 5.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(draw::DrawPlugin)
        .add_plugins(server_message_handlers::ServerMessageHandlersPlugin)
        .add_plugins(user_input::UserInputPlugin)
        .add_systems(Startup, setup)
        .run();
}

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
        Transform::from_translation(Vec3 {
            x: -10.0,
            y: 10.0,
            z: -10.0,
        })
        .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((Camera3d::default(), Transform::default()));
}
