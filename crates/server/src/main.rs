#![no_std]
#![no_main]

use server::{Connection, Server};

mod server;

fn main(_args: &[&ft::CharStar], _env: &[&ft::CharStar]) -> u8 {
    ft_async::EXECUTOR.spawn(run_server(1234));

    let err = ft_async::EXECUTOR.run();
    ft::printf!("\x1B[1;31merror\x1B[0m: failed to run the executor: {err}\n");

    0
}

ft::entry_point!(main);

/// Runs the server on the provided port.
async fn run_server(port: u16) {
    ft::printf!("\x1B[1;32minfo\x1B[0m: starting up the server on port {port}\n");

    let server = match Server::new(port) {
        Ok(ok) => ok,
        Err(err) => {
            ft::printf!("\x1B[1;31merror\x1B[0m: failed to create a TCP server: {err}");
            return;
        }
    };

    loop {
        let conn = match server.accept().await {
            Ok(ok) => ok,
            Err(err) => {
                ft::printf!("\x1B[1;31merror\x1B[0m: failed to accept a connection: {err}");
                continue;
            }
        };

        ft::printf!(
            "\x1B[1;32minfo\x1B[0m: accepted a connection from `\x1B[33m{addr}\x1B[0m`\n",
            addr = conn.address(),
        );

        ft_async::EXECUTOR.spawn(handle_connection(conn));
    }
}

/// Handles a connection from a client.
async fn handle_connection(conn: Connection) {
    ft::printf!("sending data\n");
    if let Err(err) = conn.send(b"Hello!\n").await {
        ft::printf!(
            "\x1B[1;31merror\x1B[0m: failed to send a message to `\x1B[33m{addr}\x1B[0m`: {err}\n",
            addr = conn.address(),
        );
    }
}
