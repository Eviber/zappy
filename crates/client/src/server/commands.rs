/// This module defines the commands that can be sent to the server.
use std::fmt::Display;
use std::str::FromStr;

use super::errors::InvalidResponse;

#[allow(dead_code)]
/// Enum representing a command that can be sent to the server.
pub enum Command<'a> {
    /// Move forward.
    Forward,
    /// Turn right.
    Right,
    /// Turn left.
    Left,
    /// Look around.
    Look,
    /// Inventory.
    Inventory,
    /// Take an object.
    Take(Object),
    /// Drop an object.
    Drop(Object),
    /// Kick a player from the square.
    Kick,
    /// Broadcast a message.
    Broadcast(&'a str),
    /// Begin an incantation.
    Incantation,
    /// Fork a new player.
    Fork,
    /// Ask for the number of free slots in the team.
    ConnectNbr,
}

impl Display for Command<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Forward => write!(f, "avance"),
            Command::Right => write!(f, "droite"),
            Command::Left => write!(f, "gauche"),
            Command::Look => write!(f, "voir"),
            Command::Inventory => write!(f, "inventaire"),
            Command::Take(obj) => write!(f, "prend {}", obj),
            Command::Drop(obj) => write!(f, "pose {}", obj),
            Command::Kick => write!(f, "expulse"),
            Command::Broadcast(msg) => write!(f, "broadcast {}", msg),
            Command::Incantation => write!(f, "incantation"),
            Command::Fork => write!(f, "fork"),
            Command::ConnectNbr => write!(f, "connect_nbr"),
        }
    }
}

/// Enum representing a response from the server.
#[derive(Debug, Clone)]
pub enum Response {
    /// Success.
    Ok,
    /// Error.
    Ko,
    /// The contents of the squares seen by the player.
    Seen(Vec<Vec<Object>>),
    /// Inventory.
    Inventory(Vec<(Object, u8)>), // TODO: use a HashMap instead?
    /// Acknowledgement of an incantation.
    Elevating,
    /// The level of the player after an incantation.
    Elevated(u8),
    /// The number of free slots in the team.
    FreeSlots(u8),
    /// The player has died.
    Dead,
}

impl Display for Response {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Response::Ok => write!(f, "ok"),
            Response::Ko => write!(f, "ko"),
            Response::Seen(seen) => {
                write!(f, "{{")?;
                for (i, row) in seen.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    for (j, obj) in row.iter().enumerate() {
                        if j != 0 {
                            write!(f, " ")?;
                        }
                        write!(f, "{}", obj)?;
                    }
                }
                write!(f, "}}")
            }
            Response::Inventory(inventory) => {
                write!(f, "{{")?;
                for (i, (obj, n)) in inventory.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} {}", obj, n)?;
                }
                write!(f, "}}")
            }
            Response::Elevating => write!(f, "elevation en cours"),
            Response::Elevated(level) => write!(f, "niveau actuel : {}", level),
            Response::FreeSlots(slots) => write!(f, "{}", slots),
            Response::Dead => write!(f, "mort"),
        }
    }
}

use Response::{Dead, Elevated, Elevating, FreeSlots, Inventory, Ko, Seen};

impl FromStr for Response {
    type Err = InvalidResponse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lvl = "niveau actuel : ";
        match s {
            "ok" => Ok(Response::Ok),
            "ko" => Ok(Ko),
            "mort" => Ok(Dead),
            "elevation en cours" => Ok(Elevating),
            s if s.starts_with(lvl) => Ok(Elevated(s[lvl.len()..].parse()?)),
            // s if let Ok(n) = s.parse() => Ok(FreeSlots(n)), // unstable :(
            s if s.parse::<u8>().is_ok() => Ok(FreeSlots(s.parse()?)),
            s if s.starts_with('{') && s.ends_with('}') => {
                let s = &s[1..s.len() - 1];
                let mut list = s.split(", ").peekable();
                // This is assuming that the command 'voir' never returns
                // an empty list.
                let Some(first) = list.peek() else {
                    return Ok(Inventory(Vec::new()));
                };
                // This is assuming that the response to 'voir' can not contain
                // any number and that the response to 'inventaire' always does.
                let is_inventory = first.chars().any(|c| c.is_ascii_digit());
                if is_inventory {
                    parse_inventory(list)
                } else {
                    parse_seen(list)
                }
            }
            _ => Err(InvalidResponse::ParsingError),
        }
    }
}

/// Tries to parse an inventory from a list of objects.
fn parse_inventory<'a, I>(list: I) -> Result<Response, InvalidResponse>
where
    I: Iterator<Item = &'a str>,
{
    let mut inventory = Vec::new();
    for obj_pair in list {
        let mut obj_pair = obj_pair.split(' ');
        let obj = obj_pair.next().unwrap_or("").parse()?;
        let n = obj_pair
            .next()
            .unwrap_or("")
            .parse()
            .map_err(|_| InvalidResponse::ParsingError)?;
        inventory.push((obj, n));
    }
    Ok(Inventory(inventory))
}

/// Tries to parse a list of objects seen by the player.
fn parse_seen<'a, I>(list: I) -> Result<Response, InvalidResponse>
where
    I: Iterator<Item = &'a str>,
{
    let mut seen = Vec::new();
    for row in list {
        let mut row_vec = Vec::new();
        for obj in row.split(' ') {
            row_vec.push(obj.parse()?);
        }
        seen.push(row_vec);
    }
    Ok(Seen(seen))
}

/// Enum representing all the objects that can be found in the game.
#[derive(Debug, Clone, Copy)]
pub enum Object {
    /// Food.
    Food,
    /// Linemate.
    Linemate,
    /// Deraumere.
    Deraumere,
    /// Sibur.
    Sibur,
    /// Mendiane.
    Mendiane,
    /// Phiras.
    Phiras,
    /// Thystame.
    Thystame,
    /// A player.
    Player,
}

impl Display for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Object::Food => write!(f, "nourriture"),
            Object::Linemate => write!(f, "linemate"),
            Object::Deraumere => write!(f, "deraumere"),
            Object::Sibur => write!(f, "sibur"),
            Object::Mendiane => write!(f, "mendiane"),
            Object::Phiras => write!(f, "phiras"),
            Object::Thystame => write!(f, "thystame"),
            Object::Player => write!(f, "player"),
        }
    }
}

impl FromStr for Object {
    type Err = InvalidResponse;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nourriture" => Ok(Object::Food),
            "linemate" => Ok(Object::Linemate),
            "deraumere" => Ok(Object::Deraumere),
            "sibur" => Ok(Object::Sibur),
            "mendiane" => Ok(Object::Mendiane),
            "phiras" => Ok(Object::Phiras),
            "thystame" => Ok(Object::Thystame),
            "joueur" => Ok(Object::Player),
            _ => Err(InvalidResponse::ParsingError),
        }
    }
}
