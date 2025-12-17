use super::*;
use bevy::prelude::*;

mod server_communication;
pub use server_communication::ServerAddress;
use server_communication::*;

mod dust_cloud;

/// Plugin to handle messages from the server
pub(crate) struct ServerMessageHandlersPlugin;

impl Plugin for ServerMessageHandlersPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileStacks::default());
        app.add_plugins(ServerCommunicationPlugin);
        app.add_plugins(dust_cloud::DustExplosionPlugin);
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
                add_team,
                add_player,
                fork_player,
                move_player,
                ((player_drop_item, player_get_item), update_tile_content).chain(),
                animate_moving_items,
                kill_player,
                update_player_level,
                update_player_inventory,
                expulse_player,
                (update_broadcasts, player_broadcast, follow_entities).chain(),
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
    mut commands: Commands,
    mut reader: MessageReader<ServerMessage>,
    mut map_size: ResMut<MapSize>,
    mut ground_tiles: Query<Entity, With<Ground>>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<Ground>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut msz_msg = None;
    for msg in reader.read() {
        let ServerMessage::MapSize(msg) = msg else {
            continue;
        };
        msz_msg = Some(msg);
    }
    let Some(msg) = msz_msg else {
        return;
    };
    for ground_entity in ground_tiles.iter_mut() {
        commands.entity(ground_entity).despawn();
    }
    info!("Map size updated: {}x{}", msg.width, msg.height);
    map_size.width = msg.width;
    map_size.height = msg.height;
    let mesh = Mesh3d(meshes.add(Plane3d::default().mesh().size(TILE_SIZE, TILE_SIZE)));
    let material = MeshMaterial3d(materials.add(Color::srgb(0.3, 0.5, 0.3)));
    for x in 0..map_size.width {
        let x = x as f32 * TILE_SIZE;
        for y in 0..map_size.height {
            let y = y as f32 * TILE_SIZE;
            let pos = Vec3 { x, y: 0., z: y };
            commands.spawn((
                mesh.clone(),
                material.clone(),
                Transform::from_translation(pos),
                Ground,
            ));
        }
    }
    // reposition camera to still look at center of the map
    let plane_width = map_size.width as f32 * TILE_SIZE;
    let plane_height = map_size.height as f32 * TILE_SIZE;
    let delta_x = plane_width / 2. - TILE_SIZE / 2.;
    let delta_y = plane_height / 2. - TILE_SIZE / 2.;
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

fn spawn_resource(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    item: Item,
    position: Vec3,
) -> Entity {
    commands
        .spawn((
            item,
            Transform::from_translation(position),
            Mesh3d(meshes.add(Cuboid::new(0.2, 0.1, 0.2).mesh())),
            MeshMaterial3d(materials.add(item.color())),
        ))
        .id()
}

const STACK_OFFSET: f32 = 0.1;
const STACK_GAP: f32 = 0.15;

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
                let offset = item_stack_offset(
                    Vec3::new(msg.x as f32 * TILE_SIZE, 0., msg.y as f32 * TILE_SIZE),
                    stack[index].len(),
                );
                let position = delta + offset;
                let entity = spawn_resource(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    resource_type,
                    position,
                );
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
struct Id(u64);

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
        let main_color = bevy::color::palettes::css::RED;
        let main_color = Color::srgb(main_color.red, main_color.green, main_color.blue);
        let main_material = materials.add(main_color);

        let spheres_material = materials.add(Color::srgb(0.1, 0.1, 0.1));
        let spheres_radius = 0.1;

        commands
            .spawn((
                Mesh3d(meshes.add(Capsule3d::new(0.4, 1.2).mesh())),
                MeshMaterial3d(main_material),
                transform,
                Player,
                Inventory([0; 7]),
                Level(msg.level),
                Team(msg.team.clone()),
                Id(msg.id),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(meshes.add(Sphere::new(spheres_radius).mesh())),
                    MeshMaterial3d(spheres_material.clone()),
                    Transform::from_translation(Vec3 {
                        x: -0.2,
                        y: 0.3,
                        z: 0.3,
                    }),
                ));
                parent.spawn((
                    Mesh3d(meshes.add(Sphere::new(spheres_radius).mesh())),
                    MeshMaterial3d(spheres_material),
                    Transform::from_translation(Vec3 {
                        x: 0.2,
                        y: 0.3,
                        z: 0.3,
                    }),
                ));
            })
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

#[derive(Component)]
struct MovingItem {
    /// Starting position of the item
    start_pos: Vec3,
    /// Target position of the item
    target_pos: Vec3,
    /// Time since the animation started
    progress: f32,
    /// Total duration of the animation in seconds
    duration: f32,
}

/// System to animate moving items
fn animate_moving_items(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Transform, &mut MovingItem)>,
) {
    for (e, mut transform, mut moving_item) in query.iter_mut() {
        moving_item.progress += time.delta_secs();
        let t = (moving_item.progress / moving_item.duration).min(1.0);
        transform.translation = moving_item.start_pos.lerp(moving_item.target_pos, t);
        if t >= 1.0 {
            commands.entity(e).despawn();
        }
    }
}

const ANIMATION_DURATION: f32 = 0.5;

// Don't actually change the world state, as the server will send the proper updates
fn player_drop_item(
    mut commands: Commands,
    mut reader: MessageReader<ServerMessage>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stacks: ResMut<TileStacks>,
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
        let Some((_, player_transform)) = query.iter().find(|(id, _)| id.0 == msg.player_id) else {
            warn!("Received drop item from unknown player #{}", msg.player_id);
            continue;
        };
        info!("Player #{} dropped item {}", msg.player_id, item);
        let player_translation = player_transform.translation;
        let tile_x = (player_translation.x / TILE_SIZE).round() as usize;
        let tile_y = (player_translation.z / TILE_SIZE).round() as usize;
        let stack_size = stacks.0.entry((tile_x, tile_y)).or_default()[msg.item_id as usize].len();
        let delta = item.delta_vec();
        let offset = item_stack_offset(player_translation, stack_size);
        let target_pos = delta + offset;
        let entity = spawn_resource(
            &mut commands,
            &mut meshes,
            &mut materials,
            item,
            player_translation,
        );
        commands.entity(entity).insert(MovingItem {
            start_pos: player_translation,
            target_pos,
            progress: 0.0,
            duration: ANIMATION_DURATION,
        });
    }
}

fn item_stack_offset(player_translation: Vec3, stack_size: usize) -> Vec3 {
    Vec3::new(
        player_translation.x,
        STACK_OFFSET + stack_size as f32 * STACK_GAP,
        player_translation.z,
    )
}

// Same as above
fn player_get_item(
    mut commands: Commands,
    mut reader: MessageReader<ServerMessage>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut stacks: ResMut<TileStacks>,
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
        let Some((_, transform)) = query.iter().find(|(id, _)| id.0 == msg.player_id) else {
            warn!("Received get item from unknown player #{}", msg.player_id);
            continue;
        };
        info!("Player #{} got item {}", msg.player_id, item);
        let tile_x = (transform.translation.x / TILE_SIZE).round() as usize;
        let tile_y = (transform.translation.z / TILE_SIZE).round() as usize;
        let stack = stacks.0.entry((tile_x, tile_y)).or_default();
        let delta = item.delta_vec();
        let offset = item_stack_offset(
            Vec3::new(tile_x as f32 * TILE_SIZE, 0., tile_y as f32 * TILE_SIZE),
            stack[msg.item_id as usize].len() - 1,
        );
        let item_position = delta + offset;
        let entity = stack[msg.item_id as usize].pop().unwrap_or_else(|| {
            warn!(
                "No item {} found on tile ({}, {}) for player #{} to get",
                item, tile_x, tile_y, msg.player_id
            );
            spawn_resource(
                &mut commands,
                &mut meshes,
                &mut materials,
                item,
                item_position,
            )
        });
        let player_translation = transform.translation;
        commands.entity(entity).insert(MovingItem {
            start_pos: item_position,
            target_pos: player_translation,
            progress: 0.0,
            duration: ANIMATION_DURATION,
        });
    }
}

fn expulse_player(
    mut commands: Commands,
    dust_assets: Res<dust_cloud::DustExplosionAssets>,
    mut reader: MessageReader<ServerMessage>,
    mut query: Query<(&Id, &Transform), With<Player>>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerExpulsion(msg) = msg else {
            continue;
        };
        if let Some((_, transform)) = query.iter_mut().find(|(id, _)| id.0 == msg.0) {
            info!("Player #{} has been expelled!", msg.0);
            dust_cloud::spawn_dust_explosion(&mut commands, &dust_assets, *transform);
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

/// Component to make a node follow an entity
#[derive(Component)]
pub struct FollowEntity(pub Entity);

/// Component to destroy the entity after some time
#[derive(Component)]
pub struct DestroyAfter(pub Timer);

fn player_broadcast(
    mut commands: Commands,
    mut reader: MessageReader<ServerMessage>,
    players: Query<(Entity, &Id), With<Player>>,
    current_nodes: Query<(Entity, &FollowEntity)>,
) {
    for msg in reader.read() {
        let ServerMessage::PlayerBroadcast(msg) = msg else {
            continue;
        };
        if let Some((player_e, _)) = players.iter().find(|(_, id)| id.0 == msg.id) {
            info!("Player #{} broadcasted message: {}", msg.id, msg.message);
            for (node_e, follow_entity) in current_nodes.iter() {
                if follow_entity.0 == player_e {
                    commands.entity(node_e).despawn();
                }
            }
            commands.spawn((
                Node { ..default() },
                Text::new(&msg.message),
                TextColor(Color::srgba(0., 0., 0., 0.)),
                FollowEntity(player_e),
                DestroyAfter(Timer::from_seconds(2.0, TimerMode::Once)),
            ));
        } else {
            warn!(
                "Unknown player #{} broadcasted message: {}",
                msg.id, msg.message
            );
        }
    }
}

fn update_broadcasts(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut DestroyAfter)>,
) {
    for (entity, mut destroy_after) in query.iter_mut() {
        destroy_after.0.tick(time.delta());
        if destroy_after.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn follow_entities(
    mut commands: Commands,
    camera: Single<(&Camera, &GlobalTransform)>,
    mut query: Query<
        (
            Entity,
            &FollowEntity,
            &ComputedNode,
            &mut Node,
            &mut TextColor,
        ),
        Without<Player>,
    >,
    players: Query<&Transform, With<Player>>,
) {
    let (camera, camera_transform) = *camera;
    for (entity, follow_entity, computed_node, mut node, mut color) in query.iter_mut() {
        let size = computed_node.size();
        if size.x == 0.0 && size.y == 0.0 {
            // not yet computed
            continue;
        }
        let Ok(target_transform) = players.get(follow_entity.0) else {
            info!(
                "Followed entity {:?} not found, despawning follower",
                follow_entity.0
            );
            commands.entity(entity).despawn();
            continue;
        };
        let target = target_transform.translation + Vec3::new(0., 3., 0.);
        let screen_pos = camera.world_to_viewport(camera_transform, target);
        let Ok(screen_pos) = screen_pos else {
            continue;
        };
        node.left = Val::Px(screen_pos.x - size.x / 2.);
        node.top = Val::Px(screen_pos.y);
        *color = TextColor(Color::BLACK);
    }
}

#[derive(Component)]
struct Incanting;

const INCANTATION_RISE_HEIGHT: f32 = 0.5;

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
                transform.translation.y += INCANTATION_RISE_HEIGHT;
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
            let pos_x = (transform.translation.x / TILE_SIZE).round() as usize;
            let pos_y = (transform.translation.z / TILE_SIZE).round() as usize;
            if pos_x != msg.x || pos_y != msg.y {
                continue;
            }
            commands.entity(entity).remove::<Incanting>();
            transform.translation.y -= INCANTATION_RISE_HEIGHT;
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
                continue;
            }
        } else {
            warn!("New egg #{} from unknown player #{}", msg.id, msg.parent_id);
            continue;
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
