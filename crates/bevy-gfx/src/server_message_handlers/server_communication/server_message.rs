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
    PlayerExpulsion(PlayerExpulsion),
    PlayerDeath(PlayerDeath),
    EggNew(NewEgg),
    EggHatch(EggHatch),
    PlayerConnectsFromEgg(PlayerConnectsFromEgg),
    EggDeath(EggDeath),
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
    pub orientation: u8,
    pub level: u32,
    pub team: String,
}

pub struct PlayerPosition {
    pub id: u32,
    pub x: usize,
    pub y: usize,
    pub orientation: u8,
}

pub struct PlayerLevel {
    pub id: u32,
    pub level: u32,
}

pub struct PlayerExpulsion {
    pub id: u32,
}

pub struct PlayerDeath {
    pub id: u32,
}

pub struct NewEgg {
    pub id: u32,
    pub x: usize,
    pub y: usize,
}

pub struct EggHatch {
    pub id: u32,
}

pub struct PlayerConnectsFromEgg {
    pub egg_id: u32,
}

pub struct EggDeath {
    pub id: u32,
}

fn int_parse_error(e: std::num::ParseIntError) -> String {
    e.to_string()
}

impl FromStr for UpdateMapSize {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 3 || parts[0] != "msz" {
            return Err("Invalid map size format".to_string());
        }
        Ok(UpdateMapSize {
            width: parts[1].parse().map_err(int_parse_error)?,
            height: parts[2].parse().map_err(int_parse_error)?,
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
            x: parts[1].parse().map_err(int_parse_error)?,
            y: parts[2].parse().map_err(int_parse_error)?,
            items: [
                parts[3].parse().map_err(int_parse_error)?,
                parts[4].parse().map_err(int_parse_error)?,
                parts[5].parse().map_err(int_parse_error)?,
                parts[6].parse().map_err(int_parse_error)?,
                parts[7].parse().map_err(int_parse_error)?,
                parts[8].parse().map_err(int_parse_error)?,
                parts[9].parse().map_err(int_parse_error)?,
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
            id: parts[1][1..].parse().map_err(int_parse_error)?,
            x: parts[2].parse().map_err(int_parse_error)?,
            y: parts[3].parse().map_err(int_parse_error)?,
            orientation: parts[4].parse().map_err(int_parse_error)?,
            level: parts[5].parse().map_err(int_parse_error)?,
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
            id: parts[1][1..].parse().map_err(int_parse_error)?,
            x: parts[2].parse().map_err(int_parse_error)?,
            y: parts[3].parse().map_err(int_parse_error)?,
            orientation: parts[4].parse().map_err(int_parse_error)?,
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
            id: parts[1][1..].parse().map_err(int_parse_error)?,
            level: parts[2].parse().map_err(int_parse_error)?,
        })
    }
}

impl FromStr for PlayerExpulsion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "pex" {
            return Err("Invalid player expulsion format".to_string());
        }
        Ok(PlayerExpulsion {
            id: parts[1][1..].parse().map_err(int_parse_error)?,
        })
    }
}

impl FromStr for PlayerDeath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "pdi" {
            return Err("Invalid player death format".to_string());
        }
        Ok(PlayerDeath {
            id: parts[1][1..].parse().map_err(int_parse_error)?,
        })
    }
}

impl FromStr for NewEgg {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 4 || parts[0] != "enw" {
            return Err("Invalid new egg format".to_string());
        }
        Ok(NewEgg {
            id: parts[1][1..].parse().map_err(int_parse_error)?,
            x: parts[2].parse().map_err(int_parse_error)?,
            y: parts[3].parse().map_err(int_parse_error)?,
        })
    }
}

impl FromStr for EggHatch {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "eht" {
            return Err("Invalid egg hatch format".to_string());
        }
        Ok(EggHatch {
            id: parts[1][1..].parse().map_err(int_parse_error)?,
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
            egg_id: parts[1][1..].parse().map_err(int_parse_error)?,
        })
    }
}

impl FromStr for EggDeath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 2 || parts[0] != "edi" {
            return Err("Invalid egg death format".to_string());
        }
        Ok(EggDeath {
            id: parts[1][1..].parse().map_err(int_parse_error)?,
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
        Ok(UpdateGameTick(parts[1].parse().map_err(int_parse_error)?))
    }
}

impl FromStr for ServerMessage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let command = &s[..3];
        match command {
            "msz" => Ok(ServerMessage::MapSize(s.parse()?)),
            "bct" => Ok(ServerMessage::TileContent(s.parse()?)),
            "tna" => Ok(ServerMessage::TeamName(s[4..].to_string())),
            "pnw" => Ok(ServerMessage::PlayerNew(s.parse()?)),
            "ppo" => Ok(ServerMessage::PlayerPosition(s.parse()?)),
            "plv" => Ok(ServerMessage::PlayerLevel(s.parse()?)),
            "pin" => Err("Player inventory update not implemented".to_string()),
            "pex" => Ok(ServerMessage::PlayerExpulsion(s.parse()?)),
            "pbc" => Err("Player broadcast not implemented".to_string()),
            "pic" => Err("Incantation start not implemented".to_string()),
            "pie" => Err("Incantation end not implemented".to_string()),
            "pfk" => Err("Player fork not implemented".to_string()),
            "pdr" => Err("Player drop item not implemented".to_string()),
            "pgt" => Err("Player get item not implemented".to_string()),
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
