use {
    alloc::format,
    core::{fmt::Display, str::FromStr},
    slotmap::new_key_type,
};

use crate::client::{Client, ClientError};
use crate::state::{TeamId, state};

mod command;
pub use self::command::*;

mod state;
pub use self::state::*;

mod error;
pub use self::error::*;

new_key_type! {
    /// The ID of a player.
    pub struct PlayerId;
}

impl PlayerId {
    /// Turns this player ID into a u64.
    #[inline]
    pub fn to_u64(self) -> u64 {
        self.0.as_ffi()
    }

    /// Converts the provided u64 to a player ID.
    #[inline]
    pub fn from_u64(id: u64) -> Self {
        Self(slotmap::KeyData::from_ffi(id))
    }
}

impl FromStr for PlayerId {
    type Err = core::num::ParseIntError;

    #[allow(clippy::unwrap_used, reason = "we know z cannot be parsed as a u64")]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("#") {
            Err("z".parse::<u64>().unwrap_err())
        } else {
            s[1..].parse::<u64>().map(Self::from_u64)
        }
    }
}

impl Display for PlayerId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.to_u64())
    }
}

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
        let cmd = Command::parse(line)?;
        state().players[player_id].schedule_command(cmd);
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
        .fd()
        .async_write_all(format!("{available_slots}\n{width} {height}\n").as_bytes())
        .await
}
