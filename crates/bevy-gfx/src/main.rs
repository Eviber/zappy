#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

mod args;
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

#[derive(Component)]
struct Egg;

#[derive(Component)]
struct HatchingEgg;

#[derive(Component)]
struct Forking;

const TILE_SIZE: f32 = 5.0;

use server_message_handlers::ServerAddress;

fn main() {
    let server_address = args::server_address();
    App::new()
        .insert_resource(ServerAddress::new(server_address))
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
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        bevy::light::SunDisk::EARTH,
        Transform::from_translation(Vec3 {
            x: -1000.0,
            y: 1000.0,
            z: -1000.0,
        })
        .looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        bevy::post_process::bloom::Bloom::NATURAL,
        bevy::pbr::Atmosphere::default(),
        Transform::default(),
    ));
}
