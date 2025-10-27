//! Defines the state and logic associated with a graphics connection.

use crate::client::{Client, ClientError};

/// Handles a connection to a graphics server.
pub fn handle(mut client: Client) -> Result<(), ClientError> {
    loop {
        // Read the next message and parse it.
        let tokens = client.recv_line()?.split(u8::is_ascii_whitespace);
    }
}
