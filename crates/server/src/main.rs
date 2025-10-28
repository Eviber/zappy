//! The Zappy server.

#![no_std]
#![no_main]
#![feature(maybe_uninit_as_bytes)]
#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]
#![allow(dead_code)] // FIXME: Remove this

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use self::args::Args;
use self::client::{Client, ClientError};
use self::player::PlayerError;
use self::server::Server;
use self::state::{State, set_state, state};

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;
use core::time::Duration;

mod args;
mod client;
mod gfx_connection;
mod player;
mod server;
mod state;

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
extern "C" fn interrupt_handler(_: ft::Signal) {
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

    ft_log::trace!("initializing the global state...");
    set_state(State::from_args(&args));

    ft_log::trace!("setting up the signal handlers...");
    ft::Signal::INT.set_handler(ft::process::SigHandler::from_fn(interrupt_handler));
    ft::Signal::TERM.set_handler(ft::process::SigHandler::from_fn(interrupt_handler));

    ft_log::trace!("spawning tasks...");
    ft_async::EXECUTOR.spawn(run_server(args.port));
    ft_async::EXECUTOR.spawn(run_ticks(args.tick_frequency));

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
            Ok(()) | Err(ft::Errno::INTR) => (),
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

        ft_async::EXECUTOR.spawn(handle_connection(conn, address));
    }
}

/// Handles a connection from a client.
async fn handle_connection(conn: ft::File, addr: ft::net::SocketAddr) {
    let client = Client::new(conn);
    let id = client.id();

    ft_log::info!("accepted a connection from `{addr}` (#{id})");

    match try_handle_connection(client).await {
        Ok(()) => (),
        Err(ClientError::Disconnected) => {
            ft_log::info!("client #{id} disconnected");
        }
        Err(ClientError::Unexpected(err)) => {
            ft_log::error!("failed to handle client #{id}: {err}");
        }
        Err(ClientError::Player(err)) => {
            ft_log::info!("player #{id} behaved badly: {err}");
        }
    }
}

/// See [`handle_connection`].
async fn try_handle_connection(mut client: Client) -> Result<(), ClientError> {
    let id = client.id();

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

    if team_name == b"GRAPHIC" {
        ft_log::trace!("client #{id} is a graphical monitor");
        self::gfx_connection::handle(client).await
    } else {
        let team_name =
            core::str::from_utf8(team_name).map_err(|_| PlayerError::InvalidTeamName)?;
        let team_id = state()
            .team_id_by_name(team_name)
            .ok_or_else(|| PlayerError::UnknownTeam(team_name.into()))?;
        self::player::handle(client, team_id).await
    }
}

/// Runs ticks on all the clients.
async fn run_ticks(freq: f32) {
    if let Err(err) = try_run_ticks(freq).await {
        ft_log::error!("failed to run ticks: {err}");
    }
}

/// See [`run_ticks`].
async fn try_run_ticks(freq: f32) -> ft::Result<()> {
    let period = Duration::from_secs_f32(1.0 / freq);
    let mut next_tick = ft::Clock::MONOTONIC.get();

    let mut responses = Vec::new();
    let mut send_buf = String::new();

    loop {
        // Wait until the next tick.
        ft_async::futures::sleep(next_tick).await;
        next_tick += period;

        // Notify the state.
        responses.clear();
        state().tick(&mut responses);

        // Send the responses to the clients.
        // TODO: optimize this by sending the responses concurrently.
        // There's two ways to do this:
        //  1. Create a proper future that sends all the responses concurrently.
        //  2. Spawn a task per message, but that leaves no good way to re-use buffers.
        //     This might not be a big problem though.
        for (conn, response) in responses.iter() {
            send_buf.clear();
            response.send_to(*conn, &mut send_buf).await?;
        }
    }
}
