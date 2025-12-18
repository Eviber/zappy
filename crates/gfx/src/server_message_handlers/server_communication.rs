// Server communications using TCP sockets

use crate::server_message_handlers::Id;
use bevy::prelude::*;
use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

mod server_message;

pub use server_message::ServerMessage;

pub struct ServerCommunicationPlugin;

impl Plugin for ServerCommunicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<ServerMessage>();
        app.add_systems(
            PreUpdate,
            (
                add_connecting_overlay,
                setup_server_connection,
                receive_server_message.run_if(resource_exists::<ServerConnection>),
            )
                .chain(),
        );
    }
}

#[derive(Resource)]
pub struct ServerAddress(String);

impl ServerAddress {
    pub fn new(address: impl ToString) -> Self {
        ServerAddress(address.to_string())
    }
}

#[derive(Resource)]
pub struct ServerConnection {
    stream: TcpStream,
    reader: BufReader<TcpStream>,
    buffer: String,
}

#[derive(Component)]
struct ConnectingOverlay;

fn add_connecting_overlay(
    mut commands: Commands,
    overlay: Option<Single<Entity, With<ConnectingOverlay>>>,
    connected: Option<Res<ServerConnection>>,
) {
    if overlay.is_some() || connected.is_some() {
        return;
    }
    let container = Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        justify_content: JustifyContent::Center,
        ..default()
    };

    let square = (
        BackgroundColor(Color::srgba(0.65, 0.65, 0.65, 0.8)),
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            border: UiRect::all(Val::Px(5.)),
            ..default()
        },
    );

    commands
        .spawn((container, ConnectingOverlay))
        .with_children(|parent| {
            parent.spawn(square).with_children(|parent| {
                parent.spawn((
                    Node {
                        margin: UiRect::all(Val::Auto),
                        ..default()
                    },
                    Text::new("Connecting..."),
                    TextFont::default().with_font_size(24.0),
                    TextColor(Color::BLACK),
                    TextLayout::new_with_justify(Justify::Center),
                ));
            });
        });
}

fn setup_server_connection(
    mut commands: Commands,
    server_address: Res<ServerAddress>,
    server_connection: Option<Res<ServerConnection>>,
    query: Option<Single<Entity, With<ConnectingOverlay>>>,
    id_entities: Query<Entity, With<Id>>,
) {
    if server_connection.is_some() {
        return;
    }
    let Ok(stream) = TcpStream::connect(&server_address.0) else {
        return;
    };

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

    if let Some(overlay_entity) = query {
        commands.entity(*overlay_entity).despawn();
    }
    for entity in id_entities.iter() {
        commands.entity(entity).despawn();
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
