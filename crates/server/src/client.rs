//! A simple client abstraction for the Zappy server.

use ft::collections::ReadBuffer;

/// Represents a client connected to the server.
///
/// This type provides a simple abstraction over the TCP connection and allows sending
/// messages to the client in the way defined by the subject.
pub struct Client {
    /// The file descriptor of the client.
    conn: ft::File,
    /// The read buffer used to read data from the client.
    buf: ReadBuffer,
}

impl Client {
    /// Creates a new [`Client`] from the provided file descriptor.
    pub fn new(conn: ft::File) -> Self {
        Self {
            conn,
            buf: ReadBuffer::new(),
        }
    }

    /// Sends the provided buffer to the client.
    pub async fn send_raw(&mut self, buf: &[u8]) -> ft::Result<()> {
        ft_async::futures::ready_for_writing(*self.conn).await;
        ft_async::futures::write_all(*self.conn, buf).await
    }

    /// Reads an entire line from the client, returning it.
    pub async fn recv_line(&mut self) -> ft::Result<&[u8]> {
        ft_async::futures::ready_for_reading(*self.conn).await;
        ft_async::futures::read_line(*self.conn, &mut self.buf).await
    }
}
