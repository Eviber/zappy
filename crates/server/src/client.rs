//! A simple client abstraction for the Zappy server.

use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::Relaxed;

use ft::collections::ReadBuffer;

use crate::player::PlayerError;

/// Represents a client connected to the server.
///
/// This type provides a simple abstraction over the TCP connection and allows sending
/// messages to the client in the way defined by the subject.
pub struct Client {
    /// The unique identifier of the client.
    ///
    /// This is used for debugging purposes.
    id: usize,
    /// The file descriptor of the client.
    conn: ft::File,
    /// The read buffer used to read data from the client.
    read_buf: ReadBuffer,
}

impl Client {
    /// Creates a new [`Client`] from the provided file descriptor.
    pub fn new(conn: ft::File) -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

        Self {
            id: NEXT_ID.fetch_add(1, Relaxed),
            conn,
            read_buf: ReadBuffer::new(),
        }
    }

    /// Returns the ID of the client.
    #[inline]
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the file descriptor of the client.
    #[inline]
    pub fn fd(&self) -> ft::Fd {
        *self.conn
    }

    /// Sends the provided buffer to the client.
    pub async fn send_raw(&mut self, buf: &[u8]) -> ft::Result<()> {
        ft_async::futures::ready_for_writing(*self.conn).await;
        ft_async::futures::write_all(*self.conn, buf).await
    }

    /// Reads an entire line from the client, returning it.
    pub async fn recv_line(&mut self) -> ft::Result<&[u8]> {
        ft_async::futures::ready_for_reading(*self.conn).await;
        ft_async::futures::read_line(*self.conn, &mut self.read_buf).await
    }
}

/// An error that might occur while handling a client connection (player or monitor).
pub enum ClientError {
    /// An unexpected error occurred while handling the client.
    Unexpected(ft::Errno),
    /// The player disconnected from the server unexpectedly.
    Disconnected,
    /// The player made a mistake.
    Player(PlayerError),
}

impl From<ft::Errno> for ClientError {
    fn from(value: ft::Errno) -> Self {
        match value {
            ft::Errno::CONNRESET => Self::Disconnected,
            _ => Self::Unexpected(value),
        }
    }
}

impl From<PlayerError> for ClientError {
    #[inline]
    fn from(value: PlayerError) -> Self {
        Self::Player(value)
    }
}
