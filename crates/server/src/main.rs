#![no_std]
#![no_main]

use server::Server;

mod server;

fn main(_args: &[&ft::CharStar], _env: &[&ft::CharStar]) -> u8 {
    ft_async::EXECUTOR.spawn(run_server(1234));

    let err = ft_async::EXECUTOR.run();
    ft::printf!("\x1B[1;31merror\x1B[0m: failed to run the executor: {err}\n");

    0
}

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
            "\x1B[1;32minfo\x1B[0m: accepted a connection from {addr}\n",
            addr = conn.address(),
        );
    }
}

ft::entry_point!(main);
