use {
    crate::state::ObjectClass,
    alloc::{boxed::Box, string::String},
    core::fmt::Write,
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
}

/// A response that can be sent back to a player.
pub enum Response {
    /// The string `"ok"`.
    Ok,
    /// The number of available slots in the team.
    ConnectNbr(u32),
}

impl Response {
    /// Sends the response to the specified file descriptor.
    pub async fn send_to(&self, fd: ft::Fd, buf: &mut String) -> ft::Result<()> {
        match self {
            Response::Ok => ft_async::futures::write_all(fd, b"ok\n").await?,
            Response::ConnectNbr(nbr) => {
                // NOTE: This cannot fail because writing to a string in this way will panic in case
                // of memory allocation failure instead of returning an error.
                let result = writeln!(buf, "{}", nbr);
                debug_assert!(result.is_ok(), "writing to a string should never fail");
                ft_async::futures::write_all(fd, buf.as_bytes()).await?
            }
        }

        Ok(())
    }
}
