use alloc::format;

use crate::client::{Client, ClientError};
use crate::state::{Command, ObjectClass, PlayerId, TeamId, state};

mod error;

pub use self::error::*;

/// A guard that makes a player leave their team when dropped.
struct PlayerGuard(PlayerId);

impl Drop for PlayerGuard {
    fn drop(&mut self) {
        state().leave(self.0);
    }
}

/// Handles a player connection.
///
/// When this function returns, the client connection is closed.
pub async fn handle(mut client: Client, team_id: TeamId) -> Result<(), ClientError> {
    let player_id = state().try_join_team(&client, team_id)?;
    let _guard = PlayerGuard(player_id);

    finish_handshake(&mut client, team_id).await?;

    loop {
        let line = client.recv_line().await?;
        let (cmd_name, args) = slice_split_once(line, b' ').unwrap_or((line, b""));

        let cmd = match cmd_name {
            b"avance" => Command::MoveForward,
            b"droite" => Command::TurnRight,
            b"gauche" => Command::TurnLeft,
            b"voir" => Command::LookAround,
            b"inventaire" => Command::Inventory,
            b"prend" => {
                let object = ObjectClass::from_arg(args)
                    .ok_or_else(|| PlayerError::UnknownObjectClass(args.into()))?;
                Command::PickUpObject(object)
            }
            b"pose" => {
                let object = ObjectClass::from_arg(args)
                    .ok_or_else(|| PlayerError::UnknownObjectClass(args.into()))?;
                Command::DropObject(object)
            }
            b"expulse" => Command::KnockPlayer,
            b"broadcast" => Command::Broadcast(args.into()),
            b"incantation" => Command::Evolve,
            b"fork" => Command::LayAnEgg,
            b"connect_nbr" => Command::AvailableTeamSlots,
            _ => return Err(PlayerError::UnknownCommand(cmd_name.into()).into()),
        };

        state().player_mut(player_id).schedule_command(cmd);
    }
}

/// Finish the handshake by sending:
/// 1. The number of remaining slots in the team.
/// 2. The dimensions of the world.
async fn finish_handshake(client: &mut Client, team_id: TeamId) -> ft::Result<()> {
    let lock = state();
    let available_slots = lock.available_slots_for(team_id);
    let width = lock.world.width;
    let height = lock.world.height;
    drop(lock);

    client
        .send_raw(format!("{available_slots}\n{width} {height}\n").as_bytes())
        .await
}

/// Splits the provided slice into two parts at the first occurrence of the provided delimiter.
fn slice_split_once(slice: &[u8], delim: u8) -> Option<(&[u8], &[u8])> {
    slice
        .iter()
        .position(|&b| b == delim)
        .map(|pos| (&slice[..pos], &slice[pos + 1..]))
}
