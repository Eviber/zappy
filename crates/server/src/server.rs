//! Provides useful functions to setup a TCP server.

/// A TCP server.
pub struct Server(ft::File);

impl Server {
    /// Opens a TCP server on the provided port.
    pub fn new(port: u16) -> ft::Result<Self> {
        let address = ft::net::SocketAddr::V4([0, 0, 0, 0], port);
        let socket = ft::File::socket(address.family(), ft::net::SocketType::Stream)?;
        socket.bind(&address)?;
        socket.listen(128)?;
        Ok(Self(socket))
    }

    /// Accepts a new connection.
    pub async fn accept(&self) -> ft::Result<(ft::File, ft::net::SocketAddr)> {
        ft_async::futures::ready_for_reading(*self.0).await;
        self.0.accept()
    }
}
