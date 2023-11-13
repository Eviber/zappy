use alloc::string::String;
use core::fmt;

use ft_async::sync::channel::{Receiver, Sender};

use crate::client::Client;

/// A message that can be sent from the server to the player.
enum PlayerMsg {}

impl PlayerMsg {
    /// Writes the message to the provided buffer.
    pub fn write(&self, buf: &mut String) -> fmt::Result {
        match *self {}
    }
}

/// The sender side of a channel used to send messages to a player.
#[derive(Clone)]
pub struct PlayerSender(Sender<PlayerMsg>);

impl PlayerSender {
    /// Creates a new [`PlayerSender`] from the provided client.
    ///
    /// This function will spawn a task in the background that will be responsible for
    /// sending the messages to the player when they are dispatched through the returned
    /// channel.
    pub fn new(client: &Client) -> Self {
        let (sender, receiver) = ft_async::sync::channel::make();

        ft_async::EXECUTOR.spawn(send_messages_task(client.fd(), client.id(), receiver));

        Self(sender)
    }
}

/// A task that should run in the background to send the messages dispatched through a
/// channel the player.
async fn send_messages_task(fd: ft::Fd, id: usize, receiver: Receiver<PlayerMsg>) {
    // This buffer will be written to when certain messages need to be formatted.
    let mut buf = String::new();

    while let Some(msg) = receiver.recv().await {
        buf.clear();
        msg.write(&mut buf).expect("failed to format the message");

        ft_async::futures::ready_for_writing(fd).await;
        if let Err(err) = ft_async::futures::write_all(fd, buf.as_bytes()).await {
            ft_log::error!("failed to send message to client #{id}: {}", err);
            break;
        }
    }

    ft_log::trace!("player message sender task (client #{id}) terminated");
}
