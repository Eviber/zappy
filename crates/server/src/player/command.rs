use {
    super::{PlayerError, PlayerId},
    crate::state::{ObjectClass, State},
    alloc::{boxed::Box, format},
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
    Evolve,
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
            Command::Evolve => 300,
            Command::LayAnEgg => 42,
            Command::AvailableTeamSlots => 0,
        }
    }

    /// Parses the provided byte string.
    pub fn parse(command: &[u8]) -> Result<Command, PlayerError> {
        let (cmd_name, args) = slice_split_once(command, b' ').unwrap_or((command, b""));

        match cmd_name {
            b"avance" => Ok(Self::MoveForward),
            b"droite" => Ok(Self::TurnRight),
            b"gauche" => Ok(Self::TurnLeft),
            b"voir" => Ok(Self::LookAround),
            b"inventaire" => Ok(Self::Inventory),
            b"prend" => {
                let object = ObjectClass::from_arg(args)
                    .ok_or_else(|| PlayerError::UnknownObjectClass(args.into()))?;
                Ok(Self::PickUpObject(object))
            }
            b"pose" => {
                let object = ObjectClass::from_arg(args)
                    .ok_or_else(|| PlayerError::UnknownObjectClass(args.into()))?;
                Ok(Self::DropObject(object))
            }
            b"expulse" => Ok(Self::KnockPlayer),
            b"broadcast" => Ok(Self::Broadcast(args.into())),
            b"incantation" => Ok(Self::Evolve),
            b"fork" => Ok(Self::LayAnEgg),
            b"connect_nbr" => Ok(Self::AvailableTeamSlots),
            _ => Err(PlayerError::UnknownCommand(cmd_name.into())),
        }
    }

    /// Executes the player command on the provided player.
    pub async fn execute(self, player_id: PlayerId, state: &mut State) -> ft::Result<()> {
        let player = &mut state.players[player_id];

        match self {
            Command::TurnLeft => {
                player.turn_left();
                player.conn.async_write_all(b"ok\n").await?;
                broadcast_player_moved(state, player_id).await;
            }
            Command::TurnRight => {
                player.turn_right();
                player.conn.async_write_all(b"ok\n").await?;
                broadcast_player_moved(state, player_id).await;
            }
            Command::MoveForward => {
                player.advance_position(state.world.width, state.world.height);
                player.conn.async_write_all(b"ok\n").await?;
                broadcast_player_moved(state, player_id).await;
            }
            Command::Inventory => {
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
            _ => {
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
