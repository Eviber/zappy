use bevy::prelude::*;
use std::io::{self, BufRead, BufReader};

pub struct ServerCommunication;

impl Plugin for ServerCommunication {
    fn build(&self, app: &mut App) {
        // app.add_systems(Startup, setup_stdin_reader);
        app.add_systems(PreUpdate, receive_server_message);
    }
}

#[derive(Resource)]
struct StdinReader {
    reader: BufReader<io::Stdin>,
    buffer: String,
}

enum ServerMessage {
    TileContent(UpdateTileContent),
    TeamName(String),
    PlayerNew(NewPlayer),
}

#[derive(Message)]
pub struct UpdateTileContent {
    pub x: usize,
    pub y: usize,
    pub resources: [u32; 7],
}

#[derive(Message)]
pub struct TeamName(pub String);

#[derive(Message)]
pub struct NewPlayer {
    pub id: u32,
    pub x: usize,
    pub y: usize,
    pub orientation: u8,
    pub level: u32,
    pub team: String,
}

pub fn setup_stdin_reader(mut commands: Commands) {
    let stdin = io::stdin();

    // Set stdin to non-blocking mode
    #[cfg(unix)]
    {
        use nix::fcntl::{fcntl, FcntlArg, OFlag};
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
    mut update_tile_content_writer: MessageWriter<UpdateTileContent>,
    mut team_name_writer: MessageWriter<TeamName>,
    mut new_player_writer: MessageWriter<NewPlayer>,
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
                let msg = match line.parse::<ServerMessage>() {
                    Ok(msg) => msg,
                    Err(e) => {
                        error!("Failed to parse server message: {}: {}", line, e);
                        continue;
                    }
                };
                match msg {
                    ServerMessage::TileContent(utc) => {
                        update_tile_content_writer.write(utc);
                    }
                    ServerMessage::TeamName(name) => {
                        team_name_writer.write(TeamName(name));
                    }
                    ServerMessage::PlayerNew(np) => {
                        new_player_writer.write(np);
                    }
                }
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

impl std::str::FromStr for ServerMessage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let int_parse_error = |e: std::num::ParseIntError| e.to_string();
        let parts: Vec<&str> = s.split_whitespace().collect();
        match parts.as_slice() {
            ["bct", x, y, r0, r1, r2, r3, r4, r5, r6] => {
                Ok(ServerMessage::TileContent(UpdateTileContent {
                    x: x.parse().map_err(int_parse_error)?,
                    y: y.parse().map_err(int_parse_error)?,
                    resources: [
                        r0.parse().map_err(int_parse_error)?,
                        r1.parse().map_err(int_parse_error)?,
                        r2.parse().map_err(int_parse_error)?,
                        r3.parse().map_err(int_parse_error)?,
                        r4.parse().map_err(int_parse_error)?,
                        r5.parse().map_err(int_parse_error)?,
                        r6.parse().map_err(int_parse_error)?,
                    ],
                }))
            }
            ["tna", team_name] => Ok(ServerMessage::TeamName(team_name.to_string())),
            ["pnw", id, x, y, orientation, level, team] => {
                Ok(ServerMessage::PlayerNew(NewPlayer {
                    id: id.parse().map_err(int_parse_error)?,
                    x: x.parse().map_err(int_parse_error)?,
                    y: y.parse().map_err(int_parse_error)?,
                    orientation: orientation.parse().map_err(int_parse_error)?,
                    level: level.parse().map_err(int_parse_error)?,
                    team: team.to_string(),
                }))
            }
            _ => Err(format!("Unrecognized message format: {}", s)),
        }
    }
}
