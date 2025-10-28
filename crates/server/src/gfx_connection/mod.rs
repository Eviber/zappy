//! Defines the state and logic associated with a graphics connection.

use crate::client::{Client, ClientError};

mod commands;

/// Handles a connection to a graphics server.
pub async fn handle(mut client: Client) -> Result<(), ClientError> {
    loop {
        let conn = client.fd();
        let command = client.recv_line().await?;
        self::commands::handle_one_command(conn, command).await?;
    }
}
