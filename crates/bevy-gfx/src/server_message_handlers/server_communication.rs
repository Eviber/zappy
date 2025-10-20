// initial communications, using stdin and stdout for now, later with tcp

use bevy::prelude::*;
use std::io::{self, BufRead, BufReader};

mod server_message;

pub use server_message::ServerMessage;

pub struct ServerCommunicationPlugin;

impl Plugin for ServerCommunicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_stdin_reader);
        app.add_message::<ServerMessage>();
        app.add_systems(PreUpdate, receive_server_message);
    }
}

#[derive(Resource)]
struct StdinReader {
    reader: BufReader<io::Stdin>,
    buffer: String,
}

pub fn setup_stdin_reader(mut commands: Commands) {
    let stdin = io::stdin();

    // Set stdin to non-blocking mode
    #[cfg(unix)]
    {
        use nix::fcntl::*;
        let flags = fcntl(&stdin, FcntlArg::F_GETFL).unwrap();
        let mut flags = OFlag::from_bits_truncate(flags);
        flags.insert(OFlag::O_NONBLOCK);
        fcntl(&stdin, FcntlArg::F_SETFL(flags)).unwrap();
    }

    commands.insert_resource(StdinReader {
        reader: BufReader::new(stdin),
        buffer: String::new(),
    });
}

fn receive_server_message(
    mut reader: ResMut<StdinReader>,
    mut server_message_writer: MessageWriter<ServerMessage>,
) {
    loop {
        reader.buffer.clear();

        // Split the borrow to avoid multiple mutable borrows
        let StdinReader {
            reader: buf_reader,
            buffer,
        } = &mut *reader;

        match buf_reader.read_line(buffer) {
            Ok(0) => break, // EOF
            Ok(_) => {
                let line = buffer.trim_end().to_string();
                if line.is_empty() {
                    continue;
                }
                if line == "BIENVENUE" {
                    println!("GRAPHIC");
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
                error!("Error reading stdin: {}", e);
                break;
            }
        }
    }
}
