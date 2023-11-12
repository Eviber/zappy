//! The Zappy server.

#![no_std]
#![no_main]
#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

extern crate alloc;

use self::args::Args;
use self::client::Client;
use self::server::Server;

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;

mod args;
mod client;
mod server;

/// The exit code to return in case of success.
const EXIT_SUCCESS: u8 = 0;
/// The exit code to return in case of unexpected error.
const EXIT_FAILURE: u8 = 1;
/// The exit code to return in case of usage error.
const EXIT_USAGE: u8 = 2;

/// This boolean is set to `true` when the server is interrupted by an external signal
/// (such as **SIGINT**).
static INTERRUPTED: AtomicBool = AtomicBool::new(false);

/// The **SIGINT** and **SIGTERM** signal handler.
extern "C" fn interrupt_handler() {
    INTERRUPTED.store(true, Relaxed);
}

fn main(args: &[&ft::CharStar], _env: &[&ft::CharStar]) -> u8 {
    let args = match Args::parse_args(args) {
        Ok(ok) => ok,
        Err(err) => {
            ft::eprintf!(
                core::concat!("\x1B[1;31merror:\x1B[0m {}\n\n", include_str!("usage.txt")),
                err
            );
            return EXIT_USAGE;
        }
    };

    ft_log::trace!("ARGUMENTS:");
    ft_log::trace!("  - port: {}", args.port);
    ft_log::trace!("  - size: {}x{}", args.width, args.height);
    ft_log::trace!("  - teams: {:?}", args.teams);
    ft_log::trace!("  - team slots: {}", args.initial_slot_count);
    ft_log::trace!("  - tick frequency: {}hz", args.tick_frequency);

    ft_log::trace!("setting up the signal handlers...");
    ft::Signal::Interrupt.set_handler(interrupt_handler);
    ft::Signal::Terminate.set_handler(interrupt_handler);

    ft_log::trace!("spawning tasks...");
    ft_async::EXECUTOR.spawn(run_server(args.port));

    ft_log::trace!("running the executor...");
    loop {
        if INTERRUPTED.load(Relaxed) {
            ft_log::trace!("interrupted, exiting...");
            break;
        }

        if ft_async::EXECUTOR.is_empty() {
            ft_log::trace!("no more tasks to run, exiting...");
            break;
        }

        while ft_async::EXECUTOR.run_one_task() {}

        // There are no more tasks to run, we have to wait
        // until at least one task is ready to do some work.
        match ft_async::EXECUTOR.block_until_ready() {
            // We successfully found a task to run, or have been interrupted
            // by a signal. In any case, we can go back to running tasks.
            Ok(()) | Err(ft::Errno::INTERRUPTED) => (),
            // An error occured.
            Err(err) => {
                ft_log::error!("failed to block until a task is ready: {err}");
                return EXIT_FAILURE;
            }
        }
    }

    EXIT_SUCCESS
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
