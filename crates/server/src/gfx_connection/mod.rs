//! Defines the state and logic associated with a graphics connection.

use {
    crate::{
        client::{Client, ClientError},
        state::state,
    },
    alloc::string::String,
    core::fmt::Write,
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

    // Handles monitor connection sequence.
    {
        let mut buf = String::new();
        let st = state();
        _ = writeln!(buf, "msz {} {}", st.world.width, st.world.height);
        _ = writeln!(buf, "sgt {}", st.tick_duration.as_secs_f32());

        let st = state();
        for y in 0..st.world.height {
            for x in 0..st.world.width {
                let cell = st.world.cells[y as usize * st.world.width as usize + x as usize];
                _ = writeln!(
                    buf,
                    "bct {} {} {} {} {} {} {} {} {}",
                    x,
                    y,
                    cell.food,
                    cell.linemate,
                    cell.deraumere,
                    cell.sibur,
                    cell.mendiane,
                    cell.phiras,
                    cell.thystame,
                );
            }
        }

        for team_name in st.teams.iter() {
            _ = writeln!(buf, "tna {}", team_name.name);
        }

        for (player_id, player) in st.players.iter() {
            _ = writeln!(
                buf,
                "pnw {} {} {} {} {} {}",
                player_id,
                player.x,
                player.y,
                player.facing,
                player.level,
                st.teams[player.team_id()].name,
            );
        }

        // TODO: Print the position of all eggs with the `enw` message.

        drop(st);

        client.send_raw(buf.as_bytes()).await?;
    }

    // Start the command loop.
    loop {
        let conn = client.fd();
        let command = client.recv_line().await?;
        self::commands::handle_one_command(conn, command).await?;
    }
}
