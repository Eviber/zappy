//! The Zappy server.

#![no_std]
#![no_main]
#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

extern crate alloc;

use self::args::Args;
use self::client::{Client, ClientError};
use self::player::PlayerError;
use self::server::Server;
use self::state::{set_state, state, State};

use core::sync::atomic::AtomicBool;
use core::sync::atomic::Ordering::Relaxed;
use core::time::Duration;

mod args;
mod client;
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

    ft_log::trace!("initializing the global state...");
    set_state(State::from_args(&args));

    ft_log::trace!("setting up the signal handlers...");
    ft::Signal::Interrupt.set_handler(interrupt_handler);
    ft::Signal::Terminate.set_handler(interrupt_handler);

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
        Err(ClientError::Player(PlayerError::UnknownTeam(team_name))) => {
            ft_log::info!("client #{id} wants to join team `{team_name}`, but it does not exist");
        }
        Err(ClientError::Player(PlayerError::TeamFull { name, id })) => {
            ft_log::info!("client #{id} wants to join team `{name}` (#{id}), but it is full");
        }
        Err(ClientError::Player(PlayerError::InvalidTeamName)) => {
            ft_log::info!("client #{id} sent a team name that is not valid UTF-8");
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
        todo!();
    } else {
        let team_name =
            core::str::from_utf8(team_name).map_err(|_| PlayerError::InvalidTeamName)?;
        let team_id = state()
            .lock()
            .team_id_by_name(team_name)
            .ok_or_else(|| PlayerError::UnknownTeam(team_name.into()))?;
        self::player::handle(client, team_id).await
    }
}

/// Runs ticks on all the clients.
pub async fn run_ticks(freq: f32) {
    if let Err(err) = try_run_ticks(freq).await {
        ft_log::error!("failed to run ticks: {err}");
    }
}

/// See [`run_ticks`].
pub async fn try_run_ticks(freq: f32) -> ft::Result<()> {
    let period = Duration::from_secs_f32(1.0 / freq);
    let mut next_tick = ft::Clock::MONOTONIC.get()?;

    loop {
        // Wait until the next tick.
        ft_async::futures::sleep(next_tick).await;
        next_tick += period;

        // Notify the state.
        state().lock().tick();
    }
}
