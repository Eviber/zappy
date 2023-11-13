//! Defines the [`PlayerError`] type.

use core::fmt;

use alloc::boxed::Box;

use crate::state::TeamId;

/// An error that a player can make.
///
/// This is a usage error coming from them, not an internal error.
#[derive(Debug)]
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
    /// The player sent an unknown command.
    UnknownCommand(Box<[u8]>),
    /// The player sent an unknown object class.
    UnknownObjectClass(Box<[u8]>),
}

impl fmt::Display for PlayerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PlayerError::UnknownTeam(ref team_name) => {
                write!(f, "unknown team `{}`", team_name)
            }
            PlayerError::TeamFull { ref name, id } => {
                write!(f, "team `{}` (#{}) is full", name, id)
            }
            PlayerError::InvalidTeamName => {
                write!(f, "invalid team name")
            }
            PlayerError::UnknownCommand(ref cmd_name) => {
                write!(
                    f,
                    "unknown command `{}`",
                    core::str::from_utf8(cmd_name).unwrap_or("<invalid UTF-8>")
                )
            }
            PlayerError::UnknownObjectClass(ref obj_class) => {
                write!(
                    f,
                    "unknown object class `{}`",
                    core::str::from_utf8(obj_class).unwrap_or("<invalid UTF-8>")
                )
            }
        }
    }
}
