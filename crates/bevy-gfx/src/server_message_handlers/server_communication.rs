// initial communications, using stdin and stdout for now, later with tcp

// Symbol Meaning
// X Width or horizontal position
// Y Height or vertical position
// N Team name
// q Quantity
// R Incantation result
// n Player number
// M Message
// O Orientation (N:1, E:2, S:3, O:4)
// i Resource number
// L Player level or incantation level
// e Egg number
// T Time unit

// read one line that should be "BIENVENUE"
// sends "GRAPHIC"
// "msz X Y" is received (map size)
// "sgt T" is received (time unit)
// "bct X Y q q q q q q q q q" is received for each cell of the map
// "tna N" is received for each team
// "pnw #n X Y O L N" is received for each player
// "enw #e X Y" is received for each egg
// read from stdin and parse the initial game state

use bevy::prelude::*;
use std::io::{self, BufRead, BufReader};

pub struct ServerCommunicationPlugin;

impl Plugin for ServerCommunicationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_stdin_reader);
        app.add_message::<UpdateMapSize>();
        app.add_message::<UpdateGameTick>();
        app.add_message::<UpdateTileContent>();
        app.add_message::<TeamName>();
        app.add_message::<NewPlayer>();
        app.add_message::<NewEgg>();
        app.add_systems(PreUpdate, receive_server_message);
    }
}

#[derive(Resource)]
struct StdinReader {
    reader: BufReader<io::Stdin>,
    buffer: String,
}

enum ServerMessage {
    MapSize(UpdateMapSize),
    GameTick(UpdateGameTick),
    TileContent(UpdateTileContent),
    TeamName(String),
    PlayerNew(NewPlayer),
    EggNew(NewEgg),
}

#[derive(Message)]
pub struct UpdateMapSize {
    pub width: usize,
    pub height: usize,
}

#[derive(Message)]
pub struct UpdateGameTick(pub u32);

#[derive(Message)]
pub struct UpdateTileContent {
    pub x: usize,
    pub y: usize,
    pub items: [u32; 7],
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

#[derive(Message)]
pub struct NewEgg {
    pub id: u32,
    pub x: usize,
    pub y: usize,
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
    mut map_size_writer: MessageWriter<UpdateMapSize>,
    mut game_tick_writer: MessageWriter<UpdateGameTick>,
    mut update_tile_content_writer: MessageWriter<UpdateTileContent>,
    mut team_name_writer: MessageWriter<TeamName>,
    mut new_player_writer: MessageWriter<NewPlayer>,
    mut new_egg_writer: MessageWriter<NewEgg>,
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
                match msg {
                    ServerMessage::MapSize(map_size) => {
                        map_size_writer.write(map_size);
                    }
                    ServerMessage::GameTick(game_tick) => {
                        game_tick_writer.write(game_tick);
                    }
                    ServerMessage::TileContent(utc) => {
                        update_tile_content_writer.write(utc);
                    }
                    ServerMessage::TeamName(name) => {
                        team_name_writer.write(TeamName(name));
                    }
                    ServerMessage::PlayerNew(np) => {
                        new_player_writer.write(np);
                    }
                    ServerMessage::EggNew(ne) => {
                        new_egg_writer.write(ne);
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
            ["msz", width, height] => Ok(ServerMessage::MapSize(UpdateMapSize {
                width: width.parse().map_err(int_parse_error)?,
                height: height.parse().map_err(int_parse_error)?,
            })),
            ["sgt", tick] => Ok(ServerMessage::GameTick(UpdateGameTick(
                tick.parse().map_err(int_parse_error)?,
            ))),
            ["bct", x, y, r0, r1, r2, r3, r4, r5, r6] => {
                Ok(ServerMessage::TileContent(UpdateTileContent {
                    x: x.parse().map_err(int_parse_error)?,
                    y: y.parse().map_err(int_parse_error)?,
                    items: [
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
            ["enw", id, x, y] => Ok(ServerMessage::EggNew(NewEgg {
                id: id.parse().map_err(int_parse_error)?,
                x: x.parse().map_err(int_parse_error)?,
                y: y.parse().map_err(int_parse_error)?,
            })),
            _ => Err(format!("Unrecognized message format: {s}")),
        }
    }
}
