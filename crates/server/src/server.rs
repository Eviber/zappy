//! Provides useful functions to setup a TCP server.

/// A connection to a TCP client.
pub struct Connection {
    /// The file descriptor of the connection.
    _file: ft::File,
    /// The address of the client.
    address: ft::net::SocketAddr,
}

impl Connection {
    /// Returns the file descriptor of the connection.
    #[inline]
    pub fn address(&self) -> ft::net::SocketAddr {
        self.address
    }
}

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
    pub async fn accept(&self) -> ft::Result<Connection> {
        ft_async::futures::ready_for_reading(*self.0).await;
        match self.0.accept() {
            Ok((file, address)) => Ok(Connection {
                _file: file,
                address,
            }),
            Err(e) => Err(e),
        }
    }
}
