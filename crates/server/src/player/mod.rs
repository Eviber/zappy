mod error;
mod sender;

use alloc::format;

use crate::client::Client;
use crate::client::ClientError;
use crate::state::state;
use crate::state::ObjectClass;
use crate::state::PlayerId;
use crate::state::TeamId;

pub use self::error::*;
pub use self::sender::*;

/// A guard that makes a player leave their team when dropped.
struct PlayerGuard(PlayerId);

impl Drop for PlayerGuard {
    fn drop(&mut self) {
        state().lock().leave(self.0);
    }
}

/// Handles a player connection.
///
/// When this function returns, the client connection is closed.
pub async fn handle(mut client: Client, team_id: TeamId) -> Result<(), ClientError> {
    let player_id = state().lock().try_join_team(&client, team_id)?;
    let _guard = PlayerGuard(player_id);

    finish_handshake(&mut client, team_id).await?;

    loop {
        let line = client.recv_line().await?;
        let (cmd_name, args) = slice_split_once(line, b' ').unwrap_or((line, b""));

        match cmd_name {
            b"avance" => state().lock().move_forward(player_id),
            b"droite" => state().lock().turn_right(player_id),
            b"gauche" => state().lock().turn_left(player_id),
            b"voir" => state().lock().look_around(player_id),
            b"inventaire" => state().lock().inventory(player_id),
            b"prend" => {
                let object = ObjectClass::from_arg(args)
                    .ok_or_else(|| PlayerError::UnknownObjectClass(args.into()))?;
                state().lock().pick_up_object(player_id, object)?;
            }
            b"pose" => {
                let object = ObjectClass::from_arg(args)
                    .ok_or_else(|| PlayerError::UnknownObjectClass(args.into()))?;
                state().lock().drop_object(player_id, object)?;
            }
            b"expulse" => state().lock().knock(player_id),
            b"broadcast" => state().lock().broadcast(player_id, args),
            b"incantation" => state().lock().evolve(player_id),
            b"fork" => state().lock().lay_egg(player_id),
            b"connect_nbr" => unimplemented!(),
            _ => return Err(PlayerError::UnknownCommand(cmd_name.into()).into()),
        }
    }
}

/// Finish the handshake by sending:
/// 1. The number of remaining slots in the team.
/// 2. The dimensions of the world.
async fn finish_handshake(client: &mut Client, team_id: TeamId) -> ft::Result<()> {
    let lock = state().lock();
    let available_slots = lock.available_slots_for(team_id);
    let width = lock.world().width();
    let height = lock.world().height();
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
