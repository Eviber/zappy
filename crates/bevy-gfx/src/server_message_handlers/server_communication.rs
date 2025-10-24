// Server communications using TCP sockets

use bevy::prelude::*;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

mod server_message;

pub use server_message::ServerMessage;

pub struct ServerCommunicationPlugin {
    pub server_address: String,
}

impl Default for ServerCommunicationPlugin {
    fn default() -> Self {
        Self {
            server_address: "127.0.0.1:1234".to_string(),
        }
    }
}

impl Plugin for ServerCommunicationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ServerAddress(self.server_address.clone()));
        app.add_systems(Startup, setup_server_connection);
        app.add_message::<ServerMessage>();
        app.add_systems(PreUpdate, receive_server_message);
    }
}

#[derive(Resource)]
pub struct ServerAddress(String);

#[derive(Resource)]
pub struct ServerConnection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    buffer: String,
}

// impl ServerConnection {
//     pub fn send(&mut self, message: &str) -> io::Result<()> {
//         self.stream.write_all(message.as_bytes())?;
//         self.stream.write_all(b"\n")?;
//         self.stream.flush()
//     }
// }

pub fn setup_server_connection(mut commands: Commands, server_address: Res<ServerAddress>) {
    match TcpStream::connect(&server_address.0) {
        Ok(stream) => {
            // Set socket to non-blocking mode
            if let Err(e) = stream.set_nonblocking(true) {
                error!("Failed to set socket to non-blocking: {}", e);
                return;
            }

            // Clone the stream for both reading and writing
            let reader_stream = match stream.try_clone() {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to clone stream: {}", e);
                    return;
                }
            };

            info!("Connected to server at {}", server_address.0);

            commands.insert_resource(ServerConnection {
                stream,
                reader: BufReader::new(reader_stream),
                buffer: String::new(),
            });
        }
        Err(e) => {
            error!("Failed to connect to server at {}: {}", server_address.0, e);
        }
    }
}

fn receive_server_message(
    connection: Option<ResMut<ServerConnection>>,
    mut server_message_writer: MessageWriter<ServerMessage>,
    mut commands: Commands,
) {
    let Some(mut conn) = connection else {
        // Not connected yet or connection failed
        return;
    };

    loop {
        let ServerConnection {
            reader: buf_reader,
            buffer,
            stream,
        } = &mut *conn;

        match buf_reader.read_line(buffer) {
            Ok(0) => {
                // EOF - server closed connection
                warn!("Server closed connection");
                commands.remove_resource::<ServerConnection>();
                break;
            }
            Ok(_) => {
                if !buffer.ends_with('\n') {
                    // Incomplete line, keep it in buffer and wait for more data
                    break;
                }

                let line = buffer.trim_end().to_string();
                buffer.clear();

                if line.is_empty() {
                    continue;
                }

                if line == "BIENVENUE" {
                    if let Err(e) = stream.write_all(b"GRAPHIC\n") {
                        error!("Failed to send GRAPHIC response: {}", e);
                    } else if let Err(e) = stream.flush() {
                        error!("Failed to flush stream: {}", e);
                    }
                    // TODO: wipe state?
                    continue;
                }

                let msg = match line.parse::<ServerMessage>() {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to parse server message: {}: {}", line, e);
                        continue;
                    }
                };
                server_message_writer.write(msg);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                // No data available right now, that's fine
                break;
            }
            Err(e) => {
                error!("Error reading from server: {}", e);
                break;
            }
        }
    }
}
