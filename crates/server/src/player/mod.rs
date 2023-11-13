mod error;
mod sender;

use alloc::format;

use crate::client::Client;
use crate::client::ClientError;
use crate::state::state;
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
        let _line = client.recv_line().await?;
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
