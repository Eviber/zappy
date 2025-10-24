use super::*;
use bevy::prelude::*;

mod server_communication;
use server_communication::*;

/// Plugin to handle messages from the server
pub(crate) struct ServerMessageHandlersPlugin;

impl Plugin for ServerMessageHandlersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileStacks::default());
        app.add_plugins(ServerCommunicationPlugin::default());
        app.add_systems(
            Update,
            (
                log_server_error,
                log_server_message,
                update_map_size,
                update_game_tick,
            ),
        );
        app.add_systems(
            Update,
            (
                update_tile_content,
                add_team,
                add_player,
                fork_player,
                move_player,
                player_drop_item,
                player_get_item,
                kill_player,
                update_player_level,
                update_player_inventory,
                expulse_player,
                player_broadcast,
                start_incantation,
                end_incantation,
                add_egg,
                hatch_egg,
                remove_egg_on_player_spawn,
                kill_egg,
                on_game_end,
            ),
        );
    }
}

fn update_map_size(
    mut reader: MessageReader<ServerMessage>,
    mut map_size: ResMut<MapSize>,
    mut ground: Single<(&mut Transform, &mut Mesh3d), With<Ground>>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<Ground>)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for msg in reader.read() {
        let ServerMessage::MapSize(msg) = msg else {
            continue;
        };
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

fn update_game_tick(mut reader: MessageReader<ServerMessage>, mut time_unit: ResMut<TimeUnit>) {
    for msg in reader.read() {
        let ServerMessage::GameTick(msg) = msg else {
            continue;
        };
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
    fn try_from_index(index: u32) -> Option<Self> {
        match index {
            0 => Some(Item::Nourriture),
            1 => Some(Item::Linemate),
            2 => Some(Item::Deraumère),
            3 => Some(Item::Sibur),
            4 => Some(Item::Mendiane),
            5 => Some(Item::Phiras),
            6 => Some(Item::Thystame),
            _ => None,
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

impl std::fmt::Display for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Item::Nourriture => "Nourriture",
            Item::Linemate => "Linemate",
            Item::Deraumère => "Deraumère",
            Item::Sibur => "Sibur",
            Item::Mendiane => "Mendiane",
            Item::Phiras => "Phiras",
            Item::Thystame => "Thystame",
        };
        write!(f, "{}", name)
    }
}

#[derive(Resource, Default)]
struct TileStacks(std::collections::HashMap<(usize, usize), [Vec<Entity>; 7]>);

fn update_tile_content(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    mut stacks: ResMut<TileStacks>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for msg in reader.read() {
        let ServerMessage::TileContent(msg) = msg else {
            continue;
        };
        info!("Tile ({}, {}) resources: {:?}", msg.x, msg.y, msg.items);
        let tile_pos = (msg.x, msg.y);
        let stack = stacks.0.entry(tile_pos).or_default();
        // Remove existing entities
        for entity in stack.iter_mut().flat_map(|v| v.drain(..)) {
            commands.entity(entity).despawn();
        }
        // Add new resources
        for (index, &count) in msg.items.iter().enumerate() {
            let Some(resource_type) = Item::try_from_index(index as u32) else {
                warn!("Invalid resource index: {}", index);
                continue;
            };
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

fn add_team(mut reader: MessageReader<ServerMessage>) {
    for msg in reader.read() {
        let ServerMessage::TeamName(msg) = msg else {
            continue;
        };
        info!("Team name: {}", msg);
    }
}

fn log_server_error(mut reader: MessageReader<ServerMessage>) {
    for msg in reader.read() {
        let ServerMessage::Error(msg) = msg else {
            continue;
        };
        error!("Server error message: {}", msg);
    }
}

fn log_server_message(mut reader: MessageReader<ServerMessage>) {
    for msg in reader.read() {
        let ServerMessage::Message(msg) = msg else {
            continue;
        };
        info!("Server message: {}", msg);
    }
}

#[derive(Component)]
struct Level(u32);

#[derive(Component)]
struct Inventory([u32; 7]);

#[derive(Component)]
struct Team(String);

#[derive(Component)]
struct Id(u32);

fn player_transform_from_pos(x: usize, y: usize, orientation: u32) -> Transform {
    let rotation = match orientation {
        1 => Quat::from_rotation_y(0.),                           // North
        2 => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2), // East
        3 => Quat::from_rotation_y(std::f32::consts::PI),         // South
        4 => Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),  // West
        _ => panic!("Invalid orientation"),
    };
    Transform {
        translation: Vec3::new(x as f32 * TILE_SIZE, 0.75, y as f32 * TILE_SIZE),
        rotation,
        ..Default::default()
    }
}

fn add_player(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerNew(msg) = msg else {
            continue;
        };
        let transform = player_transform_from_pos(msg.x, msg.y, msg.orientation);
        commands
            .spawn((
                Mesh3d(meshes.add(Cuboid::new(0.8, 1.5, 0.8).mesh())),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))),
                transform,
                Player,
                Inventory([0; 7]),
                Level(msg.level),
                Team(msg.team.clone()),
                Id(msg.id),
            ))
            .observe(on_player_hover)
            .observe(on_unhover);
        info!("Added player #{}", msg.id);
    }
}

fn move_player(
    mut reader: MessageReader<ServerMessage>,
    mut query: Query<(&Id, &mut Transform), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerPosition(msg) = msg else {
            continue;
        };
        if let Some((_, mut transform)) = query.iter_mut().find(|(id, _)| id.0 == msg.id) {
            let new_transform = player_transform_from_pos(msg.x, msg.y, msg.orientation);
            transform.translation = new_transform.translation;
            transform.rotation = new_transform.rotation;
            info!(
                "Moved player #{} to ({}, {}) with orientation {}",
                msg.id, msg.x, msg.y, msg.orientation
            );
        } else {
            warn!("Received position update for unknown player #{}", msg.id);
        }
    }
}

fn update_player_level(
    mut reader: MessageReader<ServerMessage>,
    mut query: Query<(&Id, &mut Level), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerLevel(msg) = msg else {
            continue;
        };
        if let Some((_, mut level)) = query.iter_mut().find(|(id, _)| id.0 == msg.id) {
            level.0 = msg.level;
            info!("Updated player #{} to level {}", msg.id, msg.level);
        } else {
            warn!("Received level update for unknown player #{}", msg.id);
        }
    }
}

fn update_player_inventory(
    mut reader: MessageReader<ServerMessage>,
    mut inventory: Query<(&Id, &mut Inventory), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerInventory(msg) = msg else {
            continue;
        };
        if let Some((_, mut inventory)) = inventory.iter_mut().find(|(id, _)| id.0 == msg.id) {
            inventory.0 = msg.items;
            info!("Updated inventory for player #{}: {:?}", msg.id, msg.items);
        } else {
            warn!("Received inventory update for unknown player #{}", msg.id);
        }
    }
}

// Don't actually change the world state, as the server will send the proper updates
fn player_drop_item(
    mut reader: MessageReader<ServerMessage>,
    query: Query<(&Id, &Transform), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerDropItem(msg) = msg else {
            continue;
        };
        let Some(item) = Item::try_from_index(msg.item_id) else {
            warn!(
                "Received drop item with invalid item id {} from player #{}",
                msg.item_id, msg.player_id
            );
            continue;
        };
        let Some((_, _transform)) = query.iter().find(|(id, _)| id.0 == msg.player_id) else {
            warn!("Received drop item from unknown player #{}", msg.player_id);
            continue;
        };
        info!("Player #{} dropped item {}", msg.player_id, item);
    }
}

// Same as above
fn player_get_item(
    mut reader: MessageReader<ServerMessage>,
    query: Query<(&Id, &Transform), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerGetItem(msg) = msg else {
            continue;
        };
        let Some(item) = Item::try_from_index(msg.item_id) else {
            warn!(
                "Received get item with invalid item id {} from player #{}",
                msg.item_id, msg.player_id
            );
            continue;
        };
        let Some((_, _transform)) = query.iter().find(|(id, _)| id.0 == msg.player_id) else {
            warn!("Received get item from unknown player #{}", msg.player_id);
            continue;
        };
        info!("Player #{} got item {}", msg.player_id, item);
    }
}

fn expulse_player(
    mut reader: MessageReader<ServerMessage>,
    mut query: Query<(&Id, &Transform), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerExpulsion(msg) = msg else {
            continue;
        };
        if let Some((_, _transform)) = query.iter_mut().find(|(id, _)| id.0 == msg.0) {
            // TODO: add expulsion effect here
            info!("Player #{} has been expelled!", msg.0);
        } else {
            warn!("Received expulsion for unknown player #{}", msg.0);
        }
    }
}

fn fork_player(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    query: Query<(Entity, &Id), With<Player>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerForking(msg) = msg else {
            continue;
        };
        if let Some((entity, _)) = query.iter().find(|(_, id)| id.0 == msg.0) {
            commands.entity(entity).insert(Forking);
            commands
                .entity(entity)
                .insert(MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.2, 0.8, 0.2),
                    emissive: LinearRgba::from(Color::srgb(10.0, 10.0, 10.0)),
                    ..Default::default()
                })));
            info!("Player #{} is forking!", msg.0);
        } else {
            warn!("Received fork notification for unknown player #{}", msg.0);
        }
    }
}

fn kill_player(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    query: Query<(Entity, &Id), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerDeath(msg) = msg else {
            continue;
        };
        if let Some((entity, _)) = query.iter().find(|(_, id)| id.0 == msg.0) {
            commands.entity(entity).despawn();
            info!("Player #{} has died and was removed from the game", msg.0);
        } else {
            warn!("Received death notification for unknown player #{}", msg.0);
        }
    }
}

fn player_broadcast(mut reader: MessageReader<ServerMessage>, query: Query<&Id, With<Player>>) {
    for msg in reader.read() {
        let ServerMessage::PlayerBroadcast(msg) = msg else {
            continue;
        };
        if query.iter().any(|id| id.0 == msg.id) {
            info!("Player #{} broadcasted message: {}", msg.id, msg.message);
        } else {
            warn!(
                "Unknown player #{} broadcasted message: {}",
                msg.id, msg.message
            );
        }
    }
}

#[derive(Component)]
struct Incanting;

fn start_incantation(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    mut players: Query<(Entity, &Id, &mut Transform), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::IncantationStart(msg) = msg else {
            continue;
        };
        for player_id in msg.players.iter() {
            if let Some((entity, _, mut transform)) =
                players.iter_mut().find(|(_, id, _)| id.0 == *player_id)
            {
                commands.entity(entity).insert(Incanting);
                transform.translation.y += 0.5;
                info!(
                    "Player #{} is participating in incantation at ({}, {})",
                    player_id, msg.x, msg.y
                );
            } else {
                warn!(
                    "Player #{} listed in incantation at ({}, {}) not found",
                    player_id, msg.x, msg.y
                );
            }
        }
    }
}

fn end_incantation(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    mut players: Query<(Entity, &mut Transform), (With<Player>, With<Incanting>)>,
) {
    for msg in reader.read() {
        let ServerMessage::IncantationEnd(msg) = msg else {
            continue;
        };
        for (entity, mut transform) in players.iter_mut() {
            let pos_x = transform.translation.x as usize / TILE_SIZE as usize;
            let pos_y = transform.translation.z as usize / TILE_SIZE as usize;
            if pos_x != msg.x || pos_y != msg.y {
                continue;
            }
            commands.entity(entity).remove::<Incanting>();
            transform.translation.y -= 0.5;
        }
        if !msg.success {
            info!(
                "Incantation at ({}, {}) failed. Players return to normal state.",
                msg.x, msg.y
            );
        } else {
            info!(
                "Incantation at ({}, {}) succeeded. Players return to normal state.",
                msg.x, msg.y
            );
        }
    }
}

fn add_egg(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    forking_players: Query<(&Id, Entity, Has<Forking>), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::EggNew(msg) = msg else {
            continue;
        };
        let transform = Transform {
            translation: Vec3::new(msg.x as f32 * TILE_SIZE, 0.25, msg.y as f32 * TILE_SIZE),
            ..Default::default()
        };
        commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(0.25).mesh())),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.8, 0.8))),
                transform,
                Id(msg.id),
                Egg,
            ))
            .observe(on_egg_hover)
            .observe(on_unhover);
        if let Some((id, parent_entity, forking)) = forking_players
            .iter()
            .find(|(id, _, _)| id.0 == msg.parent_id)
        {
            if forking {
                commands.entity(parent_entity).remove::<Forking>();
                commands
                    .entity(parent_entity)
                    .insert(MeshMaterial3d(materials.add(Color::srgb(0.8, 0.2, 0.2))));
            } else {
                warn!("Egg #{} created from non-forking player #{}", msg.id, id.0);
                return;
            }
        } else {
            warn!("New egg #{} from unknown player #{}", msg.id, msg.parent_id);
            return;
        }
        info!("Added egg #{} from player #{}", msg.id, msg.parent_id);
    }
}

fn hatch_egg(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<(Entity, &Id), With<Egg>>,
) {
    for msg in reader.read() {
        let ServerMessage::EggHatch(msg) = msg else {
            continue;
        };
        if let Some((entity, _)) = query.iter().find(|(_, id)| id.0 == msg.0) {
            commands.entity(entity).insert(HatchingEgg);
            commands
                .entity(entity)
                .insert(MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: Color::srgb(0.8, 0.8, 0.2),
                    emissive: LinearRgba::from(Color::srgb(10.0, 10.0, 10.0)),
                    ..Default::default()
                })));
            info!("Egg #{} is hatching", msg.0);
        } else {
            warn!("Received hatch notification for unknown egg #{}", msg.0);
        }
    }
}

fn remove_egg_on_player_spawn(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    query: Query<(Entity, &Id, Has<HatchingEgg>), With<Egg>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerConnectsFromEgg(msg) = msg else {
            continue;
        };
        if let Some((entity, _, hatched)) = query.iter().find(|(_, id, _)| id.0 == msg.egg_id) {
            commands.entity(entity).despawn();
            if hatched {
                info!("Hatching egg #{} has spawned a player", msg.egg_id);
            } else {
                error!("Unhatched egg #{} has spawned a player", msg.egg_id);
            }
        } else {
            warn!("Received player spawn for unknown egg #{}", msg.egg_id);
        }
    }
}

fn kill_egg(
    mut reader: MessageReader<ServerMessage>,
    mut commands: Commands,
    query: Query<(Entity, &Id, Has<HatchingEgg>), With<Egg>>,
) {
    for msg in reader.read() {
        let ServerMessage::EggDeath(msg) = msg else {
            continue;
        };
        if let Some((entity, _, hatched)) = query.iter().find(|(_, id, _)| id.0 == msg.0) {
            commands.entity(entity).despawn();
            if hatched {
                info!("Hatched egg #{} has died", msg.0);
            } else {
                error!("Unhatched egg #{} has died unexpectedly", msg.0);
            }
        } else {
            warn!("Received death notification for unknown egg #{}", msg.0);
        }
    }
}

fn on_game_end(mut reader: MessageReader<ServerMessage>, mut exit_writer: MessageWriter<AppExit>) {
    for msg in reader.read() {
        let ServerMessage::EndGame(msg) = msg else {
            continue;
        };
        info!("Game ended! Winning team: {}", msg);
        exit_writer.write(AppExit::Success);
    }
}

#[derive(Resource)]
pub struct HoverInfo(pub String);

fn on_player_hover(
    over: On<Pointer<Over>>,
    query: Query<(&Id, &Team, &Level, &Inventory, Has<Forking>), With<Player>>,
    mut commands: Commands,
) {
    if let Ok((id, team, level, inventory, forking)) = query.get(over.entity) {
        let info = HoverInfo(format!(
            "Player #{}\nTeam: {}\nLevel: {}\n\nInventory:\n  Nourriture: {}\n  Linemate: {}\n  Deraumère: {}\n  Sibur: {}\n  Mendiane: {}\n  Phiras: {}\n  Thystame: {}{}",
            id.0,
            team.0,
            level.0,
            inventory.0[0],
            inventory.0[1],
            inventory.0[2],
            inventory.0[3],
            inventory.0[4],
            inventory.0[5],
            inventory.0[6],
            if forking { "\n\nForking" } else { "" }
        ));
        commands.insert_resource(info);
        info!("Hovering over player #{}", id.0);
    } else {
        error!("Hovered entity is not a player");
    }
}

fn on_egg_hover(
    over: On<Pointer<Over>>,
    query: Query<(&Id, Has<HatchingEgg>), With<Egg>>,
    mut commands: Commands,
) {
    if let Ok((id, false)) = query.get(over.entity) {
        let info = HoverInfo(format!("Egg #{}", id.0));
        commands.insert_resource(info);
        info!("Hovering over egg #{}", id.0);
    } else if let Ok((id, true)) = query.get(over.entity) {
        let info = HoverInfo(format!("Egg #{}\n(Hatching)", id.0));
        commands.insert_resource(info);
        info!("Hovering over hatching egg #{}", id.0);
    } else {
        error!("Hovered entity is not an egg");
    }
}

fn on_unhover(out: On<Pointer<Out>>, query: Query<&Id>, mut commands: Commands) {
    if let Ok(id) = query.get(out.entity) {
        info!("Stopped hovering over entity #{}", id.0);
        commands.remove_resource::<HoverInfo>();
    }
}
