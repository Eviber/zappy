use std::str::FromStr;

use bevy::prelude::*;

#[derive(Message)]
pub enum ServerMessage {
    MapSize(UpdateMapSize),
    GameTick(UpdateGameTick),
    TileContent(UpdateTileContent),
    TeamName(String),
    PlayerNew(NewPlayer),
    PlayerPosition(PlayerPosition),
    PlayerLevel(PlayerLevel),
    PlayerInventory(PlayerInventory),
    PlayerExpulsion(Id),
    PlayerBroadcast(PlayerBroadcast),
    PlayerForking(Id),
    PlayerDropItem(PlayerItemInteraction),
    PlayerGetItem(PlayerItemInteraction),
    PlayerDeath(Id),
    IncantationStart(IncantationStart),
    IncantationEnd(IncantationEnd),
    EggNew(NewEgg),
    EggHatch(Id),
    PlayerConnectsFromEgg(PlayerConnectsFromEgg),
    EggDeath(Id),
    EndGame(String),
    Message(String),
    Error(String),
}

pub struct UpdateMapSize {
    pub width: usize,
    pub height: usize,
}

pub struct UpdateGameTick(pub u32);

pub struct UpdateTileContent {
    pub x: usize,
    pub y: usize,
    pub items: [u32; 7],
}

pub struct NewPlayer {
    pub id: u32,
    pub x: usize,
    pub y: usize,
    pub orientation: u32,
    pub level: u32,
    pub team: String,
}

pub struct PlayerPosition {
    pub id: u32,
    pub x: usize,
    pub y: usize,
    pub orientation: u32,
}

pub struct PlayerLevel {
    pub id: u32,
    pub level: u32,
}

pub struct PlayerInventory {
    pub id: u32,
    pub _x: usize,
    pub _y: usize,
    pub items: [u32; 7],
}

pub struct PlayerItemInteraction {
    pub player_id: u32,
    pub item_id: u32,
}

pub struct Id(pub u32);

pub struct PlayerBroadcast {
    pub id: u32,
    pub message: String,
}

pub struct IncantationStart {
    pub x: usize,
    pub y: usize,
    pub incantation_level: u32,
    pub players: Vec<u32>,
}

pub struct IncantationEnd {
    pub x: usize,
    pub y: usize,
    pub success: bool,
}

pub struct NewEgg {
    pub id: u32,
    pub parent_id: u32,
    pub x: usize,
    pub y: usize,
}

pub struct PlayerConnectsFromEgg {
    pub egg_id: u32,
}

use std::num::ParseIntError;

/// Utility to parse an integer from a string, returning a String error on failure.
fn parse_int<T: FromStr<Err = ParseIntError>>(s: &str) -> Result<T, String> {
    s.parse().map_err(|e: ParseIntError| e.to_string())
}

/// Utility to parse an id prefixed with an optional '#'.
fn parse_id(s: &str) -> Result<u32, String> {
    parse_int(s.strip_prefix('#').unwrap_or(s))
}

impl FromStr for UpdateMapSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 3 || parts[0] != "msz" {
            return Err("Invalid map size format".to_string());
        }
        Ok(UpdateMapSize {
            width: parse_int(parts[1])?,
            height: parse_int(parts[2])?,
        })
    }
}

impl FromStr for UpdateTileContent {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 10 || parts[0] != "bct" {
            return Err("Invalid tile content format".to_string());
        }
        Ok(UpdateTileContent {
            x: parse_int(parts[1])?,
            y: parse_int(parts[2])?,
            items: [
                parse_int(parts[3])?,
                parse_int(parts[4])?,
                parse_int(parts[5])?,
                parse_int(parts[6])?,
                parse_int(parts[7])?,
                parse_int(parts[8])?,
                parse_int(parts[9])?,
            ],
        })
    }
}

impl FromStr for NewPlayer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 7 || parts[0] != "pnw" {
            return Err("Invalid new player format".to_string());
        }
        Ok(NewPlayer {
            id: parse_id(parts[1])?,
            x: parse_int(parts[2])?,
            y: parse_int(parts[3])?,
            orientation: parse_int(parts[4])?,
            level: parse_int(parts[5])?,
            team: parts[6].to_string(),
        })
    }
}

impl FromStr for PlayerPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 5 || parts[0] != "ppo" {
            return Err("Invalid player position format".to_string());
        }
        Ok(PlayerPosition {
            id: parse_id(parts[1])?,
            x: parse_int(parts[2])?,
            y: parse_int(parts[3])?,
            orientation: parse_int(parts[4])?,
        })
    }
}

impl FromStr for PlayerLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 3 || parts[0] != "plv" {
            return Err("Invalid player level format".to_string());
        }
        Ok(PlayerLevel {
            id: parse_id(parts[1])?,
            level: parse_int(parts[2])?,
        })
    }
}

impl FromStr for PlayerInventory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 11 || parts[0] != "pin" {
            return Err("Invalid player inventory format".to_string());
        }
        Ok(PlayerInventory {
            id: parse_id(parts[1])?,
            _x: parse_int(parts[2])?,
            _y: parse_int(parts[3])?,
            items: [
                parse_int(parts[4])?,
                parse_int(parts[5])?,
                parse_int(parts[6])?,
                parse_int(parts[7])?,
                parse_int(parts[8])?,
                parse_int(parts[9])?,
                parse_int(parts[10])?,
            ],
        })
    }
}

impl FromStr for PlayerItemInteraction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 3 {
            return Err("Invalid player item interaction format".to_string());
        }
        Ok(PlayerItemInteraction {
            player_id: parse_id(parts[1])?,
            item_id: parse_int(parts[2])?,
        })
    }
}

impl FromStr for Id {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 {
            return Err("Invalid id format".to_string());
        }
        Ok(Id(parse_id(parts[1])?))
    }
}

impl FromStr for PlayerBroadcast {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 3 || parts[0] != "pbc" {
            return Err("Invalid player broadcast format".to_string());
        }
        Ok(PlayerBroadcast {
            id: parse_id(parts[1])?,
            message: parts[2..].join(" "),
        })
    }
}

impl FromStr for IncantationStart {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() < 5 || parts[0] != "pic" {
            return Err("Invalid incantation start format".to_string());
        }
        let players = parts[4..]
            .iter()
            .map(|p| parse_id(p))
            .collect::<Result<Vec<u32>, String>>()?;
        Ok(IncantationStart {
            x: parse_int(parts[1])?,
            y: parse_int(parts[2])?,
            incantation_level: parse_int(parts[3])?,
            players,
        })
    }
}

impl FromStr for IncantationEnd {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 4 || parts[0] != "pie" {
            return Err("Invalid incantation end format".to_string());
        }
        Ok(IncantationEnd {
            x: parse_int(parts[1])?,
            y: parse_int(parts[2])?,
            success: match parts[3] {
                "1" => true,
                "0" => false,
                _ => return Err("Invalid success value".to_string()),
            },
        })
    }
}

impl FromStr for NewEgg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 5 || parts[0] != "enw" {
            return Err("Invalid new egg format".to_string());
        }
        Ok(NewEgg {
            id: parse_id(parts[1])?,
            parent_id: parse_id(parts[2])?,
            x: parse_int(parts[3])?,
            y: parse_int(parts[4])?,
        })
    }
}

impl FromStr for PlayerConnectsFromEgg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "ebo" {
            return Err("Invalid player connects from egg format".to_string());
        }
        Ok(PlayerConnectsFromEgg {
            egg_id: parse_id(parts[1])?,
        })
    }
}

impl FromStr for UpdateGameTick {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "sgt" {
            return Err("Invalid game tick format".to_string());
        }
        Ok(UpdateGameTick(parse_int(parts[1])?))
    }
}

impl FromStr for ServerMessage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 3 {
            return Err(format!("Unrecognized message format: \"{s}\""));
        }
        let command = &s[..3];
        match command {
            "msz" => Ok(ServerMessage::MapSize(s.parse()?)),
            "bct" => Ok(ServerMessage::TileContent(s.parse()?)),
            "tna" => Ok(ServerMessage::TeamName(s[4..].to_string())),
            "pnw" => Ok(ServerMessage::PlayerNew(s.parse()?)),
            "ppo" => Ok(ServerMessage::PlayerPosition(s.parse()?)),
            "plv" => Ok(ServerMessage::PlayerLevel(s.parse()?)),
            "pin" => Ok(ServerMessage::PlayerInventory(s.parse()?)),
            "pex" => Ok(ServerMessage::PlayerExpulsion(s.parse()?)),
            "pbc" => Ok(ServerMessage::PlayerBroadcast(s.parse()?)),
            "pic" => Ok(ServerMessage::IncantationStart(s.parse()?)),
            "pie" => Ok(ServerMessage::IncantationEnd(s.parse()?)),
            "pfk" => Ok(ServerMessage::PlayerForking(s.parse()?)),
            "pdr" => Ok(ServerMessage::PlayerDropItem(s.parse()?)),
            "pgt" => Ok(ServerMessage::PlayerGetItem(s.parse()?)),
            "pdi" => Ok(ServerMessage::PlayerDeath(s.parse()?)),
            "enw" => Ok(ServerMessage::EggNew(s.parse()?)),
            "eht" => Ok(ServerMessage::EggHatch(s.parse()?)),
            "ebo" => Ok(ServerMessage::PlayerConnectsFromEgg(s.parse()?)),
            "edi" => Ok(ServerMessage::EggDeath(s.parse()?)),
            "sgt" => Ok(ServerMessage::GameTick(s.parse()?)),
            "seg" => Ok(ServerMessage::EndGame(s[4..].to_string())),
            "smg" => Ok(ServerMessage::Message(s[4..].to_string())),
            "suc" => Ok(ServerMessage::Error("Unknown command".to_string())),
            "sbp" => Ok(ServerMessage::Error("Bad parameters".to_string())),
            _ => Err(format!("Unrecognized message format: {s}")),
        }
    }
}
