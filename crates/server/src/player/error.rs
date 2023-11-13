//! Defines the [`PlayerError`] type.

use alloc::boxed::Box;

use crate::state::TeamId;

/// An error that a player can make.
///
/// This is a usage error coming from them, not an internal error.
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
