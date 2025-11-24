//! The main Zappy client.

#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

mod args;
mod server;

use server::commands::Msg;
use server::{Command, Result, Server};

/// Generate a random 64-bit integer.
fn rand64() -> u64 {
    use std::hash::{BuildHasher, Hasher};
    std::hash::RandomState::new().build_hasher().finish()
}

/// Send a random command to the server.
fn random_command() -> Command<'static> {
    use Command::*;
    let random_number = rand64();
    match random_number % 12 {
        0 => Forward,
        1 => Right,
        2 => Left,
        3 => Look,
        4 => Inventory,
        5 => Take(server::commands::Object::Food),
        6 => Drop(server::commands::Object::Food),
        7 => Kick,
        8 => Broadcast("Hello, world!"),
        9 => Incantation,
        10 => Fork,
        11 => ConnectNbr,
        _ => unreachable!(),
    }
}

fn main() -> Result<()> {
    let mut server = Server::new()?;
    loop {
        //server.send_command(random_command())?;
        //while let Msg::Notif(_) = server.receive()? {}
        server.send_command(Command::Inventory)?;
        while let Msg::Notif(_) = server.receive()? {}
        server.send_command(Command::Look)?;
        while let Msg::Notif(_) = server.receive()? {}
        server.send_command(Command::Forward)?;
        while let Msg::Notif(_) = server.receive()? {}
        server.send_command(Command::Forward)?;
        while let Msg::Notif(_) = server.receive()? {}
        server.send_command(Command::Forward)?;
        while let Msg::Notif(_) = server.receive()? {}
        server.send_command(Command::Forward)?;
        while let Msg::Notif(_) = server.receive()? {}
        server.send_command(Command::Left)?;
        while let Msg::Notif(_) = server.receive()? {}
    }
}
