//! The main Zappy client.

#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(clippy::missing_docs_in_private_items, clippy::must_use_candidate)]

/// Arguments parsing.
mod args;
/// Server communication.
mod server;

use server::{Result, Server};

fn main() -> Result<()> {
    let mut server = Server::new()?;
    loop {
        server.send_command("avance")?;
        server.send_command("gauche")?;
    }
    // Ok(())
}
