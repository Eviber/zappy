#![no_std]
#![no_main]

use server::{Connection, Server};

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
        let conn = match server.accept().await {
            Ok(ok) => ok,
            Err(err) => {
                ft_log::error!("failed to accept a connection: {err}");
                continue;
            }
        };

        ft_log::info!(
            "accepted a connection from `\x1B[33m{}\x1B[0m`",
            conn.address(),
        );

        ft_async::EXECUTOR.spawn(handle_connection(conn));
    }
}

/// Handles a connection from a client.
async fn handle_connection(conn: Connection) {
    if let Err(err) = conn.send(b"Hello!\n").await {
        ft_log::error!(
            "failed to send a message to `\x1B[33m{addr}\x1B[0m`: {err}",
            addr = conn.address(),
        );
    }
}
