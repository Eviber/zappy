use alloc::boxed::Box;
use alloc::string::String;
use core::fmt::Write as _;

use ft_async::sync::channel::{Receiver, Sender};

use crate::state::TeamId;

/// An error that a player can make.
pub enum PlayerError {
    /// The player provided an invalid team name (usually it's not valid UTF-8).
    InvalidTeamName,
    /// The player provided a team name that does not exist.
    UnknownTeam(Box<str>),
    /// The team that the player wants to join is full.
    TeamFull {
        /// The name of the team.
        name: Box<str>,
        /// The ID of the team.
        id: TeamId,
    },
}

/// A player currently connected to the server.
pub struct Player {
    /// The channel used to send messages to the player.
    pub sender: Sender<PlayerMsg>,
    /// The ID of the team the player belongs to.
    pub team_id: TeamId,
}

/// A message that can be sent from the server to the player.
pub enum PlayerMsg {
    /// The message sent to the player immediately after the handshake is finished.
    HandshakeFinished {
        /// The number of slots remaining in the team the player joined.
        remaining_slots: u32,
        /// The width of the map.
        width: u32,
        /// The height of the map.
        height: u32,
    },
}

/// A task responsible for sending messages to the player.
///
/// Messages are gathered from the provided channel and sent to the player
/// through the network in the order they are received.
pub async fn player_sender_task(conn: ft::Fd, receiver: Receiver<PlayerMsg>) {
    let mut buf = String::new();

    while let Some(response) = receiver.recv().await {
        buf.clear();

        match response {
            PlayerMsg::HandshakeFinished {
                remaining_slots,
                width,
                height,
            } => {
                write!(buf, "{}\n{} {}\n", remaining_slots, width, height).expect(FORMAT_ERROR);
            }
        }

        ft_async::futures::ready_for_writing(conn).await;
        if let Err(err) = ft_async::futures::write_all(conn, buf.as_bytes()).await {
            ft_log::error!("failed to send message to player: {}", err);
            return;
        }
    }

    ft_log::trace!("player message sender task terminated")
}

/// The error message displayed when the server fails to format a message.
const FORMAT_ERROR: &str = "failed to format a message";
