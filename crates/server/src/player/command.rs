use {
    super::{PlayerError, PlayerId},
    crate::Vec,
    crate::state::{ObjectClass, PlayerDirection, State},
    alloc::{boxed::Box, format, string::String},
};

/// A command that a player may attempt to execute.
#[derive(Debug)]
pub enum Command {
    /// The `avance` command.
    MoveForward,
    /// The `gauche` command.
    TurnLeft,
    /// The `droite` command.
    TurnRight,
    /// The `voir` command.
    LookAround,
    /// The `inventaire` command.
    Inventory,
    /// The `prend` command.
    PickUpObject(ObjectClass),
    /// The `pose` command.
    DropObject(ObjectClass),
    /// The `expulse` command.
    KnockPlayer,
    /// The `broadcast` command.
    Broadcast(Box<[u8]>),
    /// The `incantation` command.
    Evolve(Vec<PlayerId>),
    /// The `fork` command.
    LayAnEgg,
    /// The `connect_nbr` command.
    AvailableTeamSlots,
}

impl Command {
    /// Returns the number of ticks that this command takes to execute.
    pub fn ticks(&self) -> u32 {
        match self {
            Command::MoveForward => 7,
            Command::TurnLeft => 7,
            Command::TurnRight => 7,
            Command::LookAround => 7,
            Command::Inventory => 1,
            Command::PickUpObject(_) => 7,
            Command::DropObject(_) => 7,
            Command::KnockPlayer => 7,
            Command::Broadcast(_) => 7,
            Command::Evolve(_) => 300,
            Command::LayAnEgg => 42,
            Command::AvailableTeamSlots => 0,
        }
    }

    /// Parses the provided byte string.
    pub async fn parse(
        command: &[u8],
        player_id: PlayerId,
        state: &mut State,
    ) -> Result<Command, PlayerError> {
        let (cmd_name, args) = slice_split_once(command, b' ').unwrap_or((command, b""));

        match cmd_name {
            b"avance" => {
                if state.players[player_id].is_leveling_up {
                    return Err(PlayerError::IsLevelingUp);
                }
                Ok(Self::MoveForward)
            }
            b"droite" => {
                if state.players[player_id].is_leveling_up {
                    return Err(PlayerError::IsLevelingUp);
                }
                Ok(Self::TurnRight)
            }
            b"gauche" => {
                if state.players[player_id].is_leveling_up {
                    return Err(PlayerError::IsLevelingUp);
                }
                Ok(Self::TurnLeft)
            }
            b"voir" => Ok(Self::LookAround),
            b"inventaire" => Ok(Self::Inventory),
            b"prend" => {
                if state.players[player_id].is_leveling_up {
                    return Err(PlayerError::IsLevelingUp);
                }

                let object = ObjectClass::from_arg(args)
                    .ok_or_else(|| PlayerError::UnknownObjectClass(args.into()))?;
                Ok(Self::PickUpObject(object))
            }
            b"pose" => {
                if state.players[player_id].is_leveling_up {
                    return Err(PlayerError::IsLevelingUp);
                }

                let object = ObjectClass::from_arg(args)
                    .ok_or_else(|| PlayerError::UnknownObjectClass(args.into()))?;
                Ok(Self::DropObject(object))
            }
            b"expulse" => {
                if state.players[player_id].is_leveling_up {
                    return Err(PlayerError::IsLevelingUp);
                }

                Ok(Self::KnockPlayer)
            }
            b"broadcast" => {
                if args.contains(&b'\n') {
                    Err(PlayerError::InvalidBroadcast)
                } else {
                    Ok(Self::Broadcast(args.into()))
                }
            }
            b"incantation" => {
                if state.players[player_id].is_leveling_up {
                    return Err(PlayerError::IsLevelingUp);
                }

                let player = &state.players[player_id];
                let cell_x = player.x;
                let cell_y = player.y;
                let cell_index = cell_y * state.world.width + cell_x;
                let cell = &state.world.cells[cell_index];
                let match_level = player.level;

                let players = state
                    .players
                    .iter()
                    .filter(|(_, player)| {
                        player.x == cell_x && player.y == cell_y && player.level == match_level
                    })
                    .map(|(id, _)| id)
                    .collect::<Vec<_>>();

                let (req_players, req_l, req_d, req_s, req_m, req_p, req_t) = match player.level {
                    1 => (1, 1, 0, 0, 0, 0, 0),
                    2 => (2, 1, 1, 1, 0, 0, 0),
                    3 => (2, 2, 0, 1, 0, 2, 0),
                    4 => (4, 1, 1, 2, 0, 1, 0),
                    5 => (4, 1, 2, 1, 3, 0, 0),
                    6 => (6, 1, 2, 3, 0, 1, 0),
                    7 => (6, 2, 2, 2, 2, 2, 1),
                    _ => unreachable!(),
                };

                if players.len() != req_players
                    || cell.linemate < req_l
                    || cell.deraumere < req_d
                    || cell.sibur < req_s
                    || cell.mendiane < req_m
                    || cell.phiras < req_p
                    || cell.thystame < req_t
                {
                    return Err(PlayerError::CantEvolve);
                }

                for &player in players.iter() {
                    let player = &mut state.players[player];

                    player.is_leveling_up = true;

                    if let Err(err) = player.conn.async_write_all(b"elevation en cours").await {
                        return Err(err.into());
                    }
                }

                state.world.cells[cell_index].linemate -= req_l;
                state.world.cells[cell_index].deraumere -= req_d;
                state.world.cells[cell_index].sibur -= req_s;
                state.world.cells[cell_index].mendiane -= req_m;
                state.world.cells[cell_index].phiras -= req_p;
                state.world.cells[cell_index].thystame -= req_t;

                Ok(Self::Evolve(players))
            }
            b"fork" => {
                if state.players[player_id].is_leveling_up {
                    return Err(PlayerError::IsLevelingUp);
                }
                Ok(Self::LayAnEgg)
            }
            b"connect_nbr" => Ok(Self::AvailableTeamSlots),
            _ => Err(PlayerError::UnknownCommand(cmd_name.into())),
        }
    }

    /// Executes the player command on the provided player.
    pub async fn execute(self, player_id: PlayerId, state: &mut State) -> ft::Result<()> {
        match self {
            Command::TurnLeft => {
                let player = &mut state.players[player_id];
                player.turn_left();
                player.conn.async_write_all(b"ok\n").await?;
                broadcast_player_moved(state, player_id).await;
            }
            Command::TurnRight => {
                let player = &mut state.players[player_id];
                player.turn_right();
                player.conn.async_write_all(b"ok\n").await?;
                broadcast_player_moved(state, player_id).await;
            }
            Command::MoveForward => {
                let player = &mut state.players[player_id];
                player.advance_position(state.world.width, state.world.height);
                player.conn.async_write_all(b"ok\n").await?;
                broadcast_player_moved(state, player_id).await;
            }
            Command::Inventory => {
                let player = &state.players[player_id];
                let result = format!(
                    "{{nourriture {}, linemate {}, deraumere {}, sibur {}, mendiane {}, phiras {}, thystame {}}}\n",
                    player.inventory.get_food(),
                    player.inventory.linemate,
                    player.inventory.deraumere,
                    player.inventory.sibur,
                    player.inventory.mendiane,
                    player.inventory.phiras,
                    player.inventory.thystame,
                );
                player.conn.async_write_all(result.as_bytes()).await?;
            }
            Command::PickUpObject(object) => {
                let player = &mut state.players[player_id];
                let cell_index = player.x + player.y * state.world.width;
                if ObjectClass::try_pick_up_object(
                    &mut state.world.cells[cell_index],
                    &mut player.inventory,
                    object,
                ) {
                    player.conn.async_write_all(b"ok\n").await?;
                } else {
                    player.conn.async_write_all(b"ko\n").await?;
                }
                broadcast_inventory_transfer(state, player_id, object).await;
            }
            Command::DropObject(object) => {
                let player = &mut state.players[player_id];
                let cell_index = player.x + player.y * state.world.width;
                if ObjectClass::try_drop_object(
                    &mut player.inventory,
                    &mut state.world.cells[cell_index],
                    object,
                ) {
                    player.conn.async_write_all(b"ok\n").await?;
                } else {
                    player.conn.async_write_all(b"ko\n").await?;
                }
                broadcast_inventory_transfer(state, player_id, object).await;
            }
            Command::LookAround => {
                let player = &state.players[player_id];
                let mut sight = Vec::with_capacity((player.level + 1) * (player.level + 1));

                sight.push(Some(
                    state.world.cells[player.x + player.y * state.world.width],
                ));
                let mut level_tool = 1;
                // dir represents 2 2d vectors, the offset dir per level, and the offset dir per case inside that level
                let dir: (i32, i32) = match player.facing {
                    PlayerDirection::North => (0, 1),
                    PlayerDirection::South => (0, -1),
                    PlayerDirection::East => (1, 0),
                    PlayerDirection::West => (-1, 0),
                };
                for i in 1..(player.level + 1) * (player.level + 1) {
                    if level_tool * level_tool < (i + 1) {
                        level_tool += 1;
                    }
                    let level_offset = (level_tool - 1) as i32;
                    let level_index = (i as i32 + 1) - level_offset * level_offset;

                    // trick to reduce tuple size used here
                    let mut x_sight = player.x as i32
                        + level_offset * dir.0
                        + (level_index - level_offset - 1) * dir.1;
                    let mut y_sight = player.y as i32 + level_offset * dir.1
                        - (level_index - level_offset - 1) * dir.0;
                    // these << 4 lines are because if world size is smaller than some number index out of bounds might occur
                    x_sight += (state.world.width as i32) << 4;
                    x_sight %= state.world.width as i32;
                    y_sight += (state.world.height as i32) << 4;
                    y_sight %= state.world.height as i32;
                    sight.push(Some(
                        state.world.cells[(x_sight + y_sight * state.world.width as i32) as usize],
                    ));
                }
                ft_log::info!("len is {}", sight.len());
                let mut result = String::from("{");
                for s in &sight {
                    match s {
                        Some(cell) => {
                            for _ in 0..cell.food {
                                result.push_str("nourriture ");
                            }
                            for _ in 0..cell.linemate {
                                result.push_str("linemate ");
                            }
                            for _ in 0..cell.deraumere {
                                result.push_str("deraumere ");
                            }
                            for _ in 0..cell.sibur {
                                result.push_str("sibur ");
                            }
                            for _ in 0..cell.mendiane {
                                result.push_str("mendiane ");
                            }
                            for _ in 0..cell.phiras {
                                result.push_str("phiras ");
                            }
                            for _ in 0..cell.thystame {
                                result.push_str("thystame ");
                            }
                            for _ in 0..cell.player_count {
                                result.push_str("player ");
                            }
                        }
                        None => break,
                    }
                    if result.len() > 1 {
                        result.pop();
                    }
                    result.push_str(", ");
                }
                result.pop();
                result.pop();
                result.push_str("}\n");
                ft_log::info!("successfully ended loop {}", result);
                player.conn.async_write_all(result.as_bytes()).await?;
                ft_log::info!("successfully sent message");
            }
            Command::KnockPlayer => {
                let player_x = state.players[player_id].x;
                let player_y = state.players[player_id].y;
                let front_cell =
                    state.players[player_id].get_front_cell(state.world.width, state.world.height);
                let kickee_string = match state.players[player_id].facing {
                    PlayerDirection::North => String::from("deplacement 3\n"),
                    PlayerDirection::East => String::from("deplacement 4\n"),
                    PlayerDirection::South => String::from("deplacement 1\n"),
                    PlayerDirection::West => String::from("deplacement 2\n"),
                };
                for (kickee_id, kickee) in &mut state.players {
                    if kickee.x == player_x && kickee.y == player_y && kickee_id != player_id {
                        kickee.x = front_cell.0;
                        kickee.y = front_cell.1;
                        kickee
                            .conn
                            .async_write_all(kickee_string.as_bytes())
                            .await?;
                    }
                }
                state.players[player_id]
                    .conn
                    .async_write_all(b"ok\n")
                    .await?;
            }
            Command::Broadcast(text) => {
                // Handled when parsing the command in the first place.
                assert!(!text.contains(&b'\n'));

                let source_x = state.players[player_id].x;
                let source_y = state.players[player_id].y;

                let world_width = state.world.width;
                let world_height = state.world.height;

                for (other_player_id, player) in &state.players {
                    if other_player_id == player_id {
                        continue;
                    }

                    let other_x = state.players[other_player_id].x;
                    let other_y = state.players[other_player_id].y;

                    // Calculate the shortest distance considering world wrapping in both
                    // dimensions.

                    fn shortest_wrapped_distance(
                        pos1: usize,
                        pos2: usize,
                        world_size: usize,
                    ) -> isize {
                        let direct = pos2 as isize - pos1 as isize;
                        let wrap_positive = direct + world_size as isize;
                        let wrap_negative = direct - world_size as isize;

                        if direct.abs() <= wrap_positive.abs()
                            && direct.abs() <= wrap_negative.abs()
                        {
                            direct
                        } else if wrap_positive.abs() <= wrap_negative.abs() {
                            wrap_positive
                        } else {
                            wrap_negative
                        }
                    }

                    let dx = shortest_wrapped_distance(source_x, other_x, world_width);
                    let dy = shortest_wrapped_distance(source_y, other_y, world_height);

                    // Determine direction based on dx and dy.
                    let direction = if dx == 0 && dy == 0 {
                        0 // Same position
                    } else if dx > 0 && dy == 0 {
                        1 // Right
                    } else if dx > 0 && dy > 0 {
                        2 // Top right
                    } else if dx == 0 && dy > 0 {
                        3 // Top
                    } else if dx < 0 && dy > 0 {
                        4 // Top left
                    } else if dx < 0 && dy == 0 {
                        5 // Left
                    } else if dx < 0 && dy < 0 {
                        6 // Bottom left
                    } else if dx == 0 && dy < 0 {
                        7 // Bottom
                    } else {
                        8 // Bottom right
                    };

                    // Send the broadcast message to the other player
                    player
                        .conn
                        .async_write_all(format!("message {direction}, ").as_bytes())
                        .await?;
                    player.conn.async_write_all(&text).await?;
                    player.conn.async_write_all(b"\n").await?;
                }
            }
            Command::Evolve(players) => {
                for player_id in players {
                    if let Some(player) = state.players.get_mut(player_id) {
                        player.level += 1;
                        player.is_leveling_up = false;

                        let buf = format!("niveau actuel : {}", player.level);
                        player.conn.async_write_all(buf.as_bytes()).await?;
                    }
                }
            }
            _ => {
                let player = &state.players[player_id];
                player
                    .conn
                    .async_write_all(b"error: not implemented yet\n")
                    .await?;
            }
        }

        Ok(())
    }
}

/// Broadcasts a player's information to all graphics monitors.
async fn broadcast_player_moved(state: &State, player_id: PlayerId) {
    let player = &state.players[player_id];
    state
        .broadcast_to_graphics_monitors(
            format!(
                "ppo {} {} {} {}",
                player_id, player.x, player.y, player.facing,
            )
            .as_bytes(),
        )
        .await;
}

/// Broadcasts a player's inventory transfer from/to a cell of the world (the current
/// cell of the player).
async fn broadcast_inventory_transfer(state: &State, player_id: PlayerId, obj: ObjectClass) {
    let player = &state.players[player_id];
    let cell_index = player.y * state.world.width + player.x;
    let cell_inv = &state.world.cells[cell_index];

    let broadcasted_bytes = format!(
        "\
        pgt #{player_id} {obj:?}\n\
        pin #{player_id} {x} {y} {a1} {b1} {c1} {d1} {e1} {f1} {g1}\n\
        bct {x} {y} {a2} {b2} {c2} {d2} {e2} {f2} {g2}\n\
        ",
        player_id = player_id,
        obj = obj,
        x = player.x,
        y = player.y,
        a1 = player.inventory.get_food(),
        b1 = player.inventory.linemate,
        c1 = player.inventory.deraumere,
        d1 = player.inventory.sibur,
        e1 = player.inventory.mendiane,
        f1 = player.inventory.phiras,
        g1 = player.inventory.thystame,
        a2 = cell_inv.food,
        b2 = cell_inv.linemate,
        c2 = cell_inv.deraumere,
        d2 = cell_inv.sibur,
        e2 = cell_inv.mendiane,
        f2 = cell_inv.phiras,
        g2 = cell_inv.thystame,
    );

    state
        .broadcast_to_graphics_monitors(broadcasted_bytes.as_bytes())
        .await;
}

/// Splits the provided slice into two parts at the first occurrence of the provided delimiter.
fn slice_split_once(slice: &[u8], delim: u8) -> Option<(&[u8], &[u8])> {
    slice
        .iter()
        .position(|&b| b == delim)
        .map(|pos| (&slice[..pos], &slice[pos + 1..]))
}
