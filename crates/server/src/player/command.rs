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

/// Splits the provided slice into two parts at the first occurrence of the provided delimiter.
fn slice_split_once(slice: &[u8], delim: u8) -> Option<(&[u8], &[u8])> {
    slice
        .iter()
        .position(|&b| b == delim)
        .map(|pos| (&slice[..pos], &slice[pos + 1..]))
}
