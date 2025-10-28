//! Defines the state and logic associated with a graphics connection.

use crate::{
    client::{Client, ClientError},
    state::state,
};

mod commands;

/// The guard responsible for unsubscribing a graphics monitor from the state when
/// dropped.
struct GfxMonitorGuard(ft::Fd);

impl Drop for GfxMonitorGuard {
    fn drop(&mut self) {
        let mut state = state();

        let idx = state
            .gfx_monitors
            .iter()
            .position(|x| *x == self.0)
            .expect("Graphics monitor not found or removed before the end of the handler");

        state.gfx_monitors.swap_remove(idx);
    }
}

/// Handles a connection to a graphics server.
pub async fn handle(mut client: Client) -> Result<(), ClientError> {
    state().gfx_monitors.push(client.fd());
    let _guard = GfxMonitorGuard(client.fd());

    loop {
        let conn = client.fd();
        let command = client.recv_line().await?;
        self::commands::handle_one_command(conn, command).await?;
    }
}
