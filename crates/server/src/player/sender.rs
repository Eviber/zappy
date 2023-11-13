use core::fmt;

use alloc::string::String;

/// The sender side of a channel used to send messages to a player.
#[derive(Clone)]
pub struct PlayerSender {
    /// A buffer used to store formatted messages before sending them to the player.
    buffer: String,
    /// The file descriptor to which the messages are sent.
    conn: ft::Fd,
}

impl PlayerSender {
    /// Creates a new [`PlayerSender`] from the provided client.
    pub fn new(fd: ft::Fd) -> Self {
        Self {
            buffer: String::new(),
            conn: fd,
        }
    }

    async fn send_raw(&mut self, buf: &[u8]) -> ft::Result<()> {
        ft_async::futures::ready_for_writing(self.conn).await;
        ft_async::futures::write_all(self.conn, buf).await
    }

    /// Sends the provided formatted message to the player.
    async fn send_fmt(&mut self, args: fmt::Arguments<'_>) -> ft::Result<()> {
        self.buffer.clear();
        fmt::write(&mut self.buffer, args).expect("failed to format message");
        ft_async::futures::ready_for_writing(self.conn).await;
        ft_async::futures::write_all(self.conn, self.buffer.as_bytes()).await
    }

    /// Send "ok" to the player.
    pub async fn ok(&mut self) -> ft::Result<()> {
        self.send_raw(b"ok\n").await
    }

    /// Send "ko" to the player.
    pub async fn ko(&mut self) -> ft::Result<()> {
        self.send_raw(b"ko\n").await
    }

    /// Send "mort" to the player.
    pub async fn dead(&mut self) -> ft::Result<()> {
        self.send_raw(b"mort\n").await
    }

    /// Send "elevation en cours" to the player.
    pub async fn elevation_in_progress(&mut self) -> ft::Result<()> {
        self.send_raw(b"elevation en cours\n").await
    }

    /// Send "niveau actuel : {level}" to the player.
    pub async fn current_level(&mut self, level: u32) -> ft::Result<()> {
        self.send_fmt(format_args!("niveau actuel : {}\n", level))
            .await
    }

    /// Send "{case1, case2, ...}" to the player.
    pub async fn see(&mut self, tiles: &[&str]) -> ft::Result<()> {
        self.send_fmt(format_args!("voir {}\n", FormatSeeResponse(tiles)))
            .await
    }
}

/// A formatted response to a [`see`] request.
struct FormatSeeResponse<'a>(&'a [&'a str]);

impl<'a> fmt::Display for FormatSeeResponse<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{");
        for (i, tile) in self.0.iter().enumerate() {
            f.write_str(tile)?;
            f.write_str(", ")?;
        }
        f.write_str("}");

        Ok(())
    }
}
