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

pub struct NewEgg {
    pub id: u32,
    pub parent_id: u32,
    pub x: usize,
    pub y: usize,
}

pub struct PlayerConnectsFromEgg {
    pub egg_id: u32,
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

impl FromStr for PlayerInventory {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 11 || parts[0] != "pin" {
            return Err("Invalid player inventory format".to_string());
        }
        Ok(PlayerInventory {
            id: parts[1][1..].parse().map_err(int_parse_error)?,
            _x: parts[2].parse().map_err(int_parse_error)?,
            _y: parts[3].parse().map_err(int_parse_error)?,
            items: [
                parts[4].parse().map_err(int_parse_error)?,
                parts[5].parse().map_err(int_parse_error)?,
                parts[6].parse().map_err(int_parse_error)?,
                parts[7].parse().map_err(int_parse_error)?,
                parts[8].parse().map_err(int_parse_error)?,
                parts[9].parse().map_err(int_parse_error)?,
                parts[10].parse().map_err(int_parse_error)?,
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
            player_id: parts[1][1..].parse().map_err(int_parse_error)?,
            item_id: parts[2].parse().map_err(int_parse_error)?,
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
        Ok(Id(parts[1][1..].parse().map_err(int_parse_error)?))
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
            id: parts[1][1..].parse().map_err(int_parse_error)?,
            message: parts[2..].join(" "),
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
            id: parts[1][1..].parse().map_err(int_parse_error)?,
            parent_id: parts[2][1..].parse().map_err(int_parse_error)?,
            x: parts[3].parse().map_err(int_parse_error)?,
            y: parts[4].parse().map_err(int_parse_error)?,
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
            "pin" => Ok(ServerMessage::PlayerInventory(s.parse()?)),
            "pex" => Ok(ServerMessage::PlayerExpulsion(s.parse()?)),
            "pbc" => Ok(ServerMessage::PlayerBroadcast(s.parse()?)),
            "pic" => Err("Incantation start not implemented".to_string()),
            "pie" => Err("Incantation end not implemented".to_string()),
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
