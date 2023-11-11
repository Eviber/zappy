//! The Zappy server.

#![no_std]
#![no_main]
#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

extern crate alloc;

use client::Client;

use self::server::Server;

mod client;
mod server;

fn main(_args: &[&ft::CharStar], _env: &[&ft::CharStar]) -> u8 {
    ft_async::EXECUTOR.spawn(run_server(1234));

    let err = ft_async::EXECUTOR.run();
    ft_log::error!("failed to run the executor: {err}");

    0
}

ft::entry_point!(main);

/// Runs the server on the provided port.
async fn run_server(port: u16) {
    ft_log::info!("starting up the server on port {port}");

    let server = match Server::new(port) {
        Ok(ok) => ok,
        Err(err) => {
            ft_log::error!("failed to create a TCP server: {err}");
            return;
        }
    };

    loop {
        let (conn, address) = match server.accept().await {
            Ok(ok) => ok,
            Err(err) => {
                ft_log::error!("failed to accept a connection: {err}");
                continue;
            }
        };

        ft_log::info!("accepted a connection from `\x1B[33m{address}\x1B[0m`");
        ft_async::EXECUTOR.spawn(handle_connection(conn, address));
    }
}

/// Handles a connection from a client.
async fn handle_connection(conn: ft::File, addr: ft::net::SocketAddr) {
    if let Err(err) = try_handle_connection(conn).await {
        ft_log::error!("failed to handle a connection with `\x1B[33m{addr}\x1B[0m`: {err}");
    }
}

/// See [`handle_connection`].
async fn try_handle_connection(conn: ft::File) -> ft::Result<()> {
    let mut client = Client::new(conn);

    //
    // HANDSHAKE
    //
    // 1. When a new client connects to the server, it must first wait the server for
    //    a `BIENVENUE` message.
    //
    // 2. After this, the client must indicate which team it wants to join. The special
    //    name `GRAPHIC` is reserved for graphical monitors.
    //
    // The rest of the handshake depends on the type of client (player or graphical).
    //

    client.send_raw(b"BIENVENUE\n").await?;
    let team_name = client.recv_line().await?;

    ft_log::trace!(
        "received team name: `{}`",
        core::str::from_utf8(team_name).unwrap_or("<invalid-utf8>")
    );

    Ok(())
}
