//! Defines the global state of the server.

use crate::{
    player::{Command, PlayerId},
    rng::Rng,
};
use alloc::vec::Vec;
use {
    crate::player::{PlayerState, Response},
    core::fmt::Write,
};
use {alloc::boxed::Box, core::fmt::Display};

use crate::args::Args;
use crate::client::Client;
use crate::player::PlayerError;

mod world;

pub use self::world::*;

/// The ID of a team.
pub type TeamId = usize;

/// Information about the state of a team.
pub struct Team {
    /// The name of the team.
    pub name: Box<str>,
    /// The number of available slots in the team.
    pub available_slots: u32,
}

/// The ID of a monitor.
pub type MonitorId = usize;

/// A direction in which the player can be facing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerDirection {
    /// The player faces the negative Y direction.
    North,
    /// The player faces the positive Y direction.
    South,
    /// The player faces the negative X direction.
    West,
    /// The player faces the positive X direction.
    East,
}

impl PlayerDirection {
    /// Turns the direction right.
    pub fn turn_right(self) -> Self {
        match self {
            PlayerDirection::North => PlayerDirection::East,
            PlayerDirection::South => PlayerDirection::West,
            PlayerDirection::West => PlayerDirection::North,
            PlayerDirection::East => PlayerDirection::South,
        }
    }

    /// Turns the direction left.
    pub fn turn_left(self) -> Self {
        match self {
            PlayerDirection::North => PlayerDirection::West,
            PlayerDirection::South => PlayerDirection::East,
            PlayerDirection::West => PlayerDirection::South,
            PlayerDirection::East => PlayerDirection::North,
        }
    }
}

impl Display for PlayerDirection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::North => f.write_char('1'),
            Self::East => f.write_char('2'),
            Self::South => f.write_char('3'),
            Self::West => f.write_char('4'),
        }
    }
}

/// The global state of the server, responsible for managing the clients and the game.
#[allow(clippy::vec_box)] // `PlayerState` is a huge struct, copying it around is not a good idea.
pub struct State {
    /// Information about the teams available in the current game.
    pub teams: Box<[Team]>,
    /// The list of players currently connected to the server.
    pub players: Vec<Box<PlayerState>>,
    /// The current state of the world.
    pub world: World,
    /// The random number generator used by the server.
    pub rng: Rng,
}

impl State {
    /// Creates a new instance of [`State`] from the arguments passed to the server.
    pub fn from_args(args: &Args) -> Self {
        let teams = args
            .teams
            .iter()
            .map(|&team| Team {
                name: team.into(),
                available_slots: args.initial_slot_count,
            })
            .collect();

        let world = World::new(args.width, args.height);

        Self {
            teams,
            players: Vec::new(),
            world,
            rng: Rng::from_urandom().unwrap_or(Rng::new(0xdeadbeef)),
        }
    }

    /// Returns the ID of a team from its name.
    pub fn team_id_by_name(&self, name: &str) -> Option<TeamId> {
        self.teams.iter().position(|team| &*team.name == name)
    }

    /// Registers a player to the server, joining the specified team.
    ///
    /// # Arguments
    ///
    /// - `client`: the client to register to the server.
    ///
    /// - `name`: the name of the team the player wants to join.
    #[allow(clippy::unwrap_used)]
    pub fn try_join_team(
        &mut self,
        client: &Client,
        team_id: TeamId,
    ) -> Result<PlayerId, PlayerError> {
        let team = &mut self.teams[team_id];

        if team.available_slots == 0 {
            return Err(PlayerError::TeamFull {
                name: team.name.clone(),
                id: team_id,
            });
        }

        team.available_slots -= 1;

        self.players.push(
            PlayerState::new_random(
                client,
                team_id,
                &mut self.rng,
                self.world.width,
                self.world.height,
            )
            .into(),
        );

        Ok(client.id())
    }

    /// Returns the index of a player in the list of players.
    #[inline]
    fn player_index_by_id(&self, player: PlayerId) -> Option<usize> {
        self.players.iter().position(|p| p.id() == player)
    }

    /// Returns the state of the player with the provided ID.
    ///
    /// # Panics
    ///
    /// This function panics if the player does not exist.
    pub fn player_mut(&mut self, player_id: PlayerId) -> &mut PlayerState {
        self.get_player_mut(player_id)
            .expect("no player with the provided ID")
    }

    /// Returns the state of the player with the provided ID.
    ///
    /// # Panics
    ///
    /// This function panics if the player does not exist.
    pub fn player(&self, player_id: PlayerId) -> &PlayerState {
        self.get_player(player_id)
            .expect("no player with the provided ID")
    }

    /// Returns the state of the player with the provided ID, or `None` if the player
    /// does not exist.
    pub fn get_player(&self, player_id: PlayerId) -> Option<&PlayerState> {
        self.player_index_by_id(player_id)
            .and_then(|i| self.players.get(i).map(|x| &**x))
    }

    /// Returns the state of the player with the provided ID, or `None` if the player
    /// does not exist.
    pub fn get_player_mut(&mut self, player_id: PlayerId) -> Option<&mut PlayerState> {
        self.player_index_by_id(player_id)
            .and_then(|i| self.players.get_mut(i).map(|x| &mut **x))
    }

    /// Removes a player from the server.
    pub fn leave(&mut self, player: PlayerId) {
        let index = self
            .players
            .iter()
            .position(|p| p.id() == player)
            .expect("no player with the provided ID found");

        let player = self.players.remove(index);
        self.teams[player.team_id()].available_slots += 1;
    }

    /// Returns the number of available slots in the specified team.
    #[inline]
    pub fn available_slots_for(&self, team: TeamId) -> u32 {
        self.teams[team].available_slots
    }

    /// Executes a command, returning a response.
    pub fn execute_command(&mut self, command: Command, player: usize) -> Response {
        let player = &mut self.players[player];

        match command {
            Command::TurnLeft => {
                player.turn_left();
                Response::Ok
            }
            Command::TurnRight => {
                player.turn_right();
                Response::Ok
            }
            Command::MoveForward => {
                player.advance_position(self.world.width, self.world.height);
                Response::Ok
            }
            _ => Response::Ok,
        }
    }

    /// Notifies the state that a whole tick has passed.
    ///
    /// # Arguments
    ///
    /// - `responses` - a list of responses that must be sent to their associated file
    ///   descriptiors.
    pub fn tick(&mut self, responses: &mut Vec<(ft::Fd, Response)>) {
        for player_index in 0..self.players.len() {
            let player = &mut self.players[player_index];
            let Some(cmd) = player.try_unqueue_command() else {
                continue;
            };

            // Execute the command.
            ft_log::trace!("executing command for #{}: {:?}", player.id(), cmd);

            let player_conn = player.conn;

            let response = self.execute_command(cmd, player_index);
            responses.push((player_conn, response));
        }
    }
}

/// The global state of the server.
static STATE: ft::Mutex<Option<State>, ft::sync::mutex::NoBlockMutex> = ft::Mutex::new(None);

/// Initializes the global [`State`].
///
/// # Panics
///
/// This function panics if the global state is already initialized.
#[inline]
pub fn set_state(state: State) {
    let mut lock = STATE.lock();
    assert!(
        lock.is_none(),
        "the global state has already been initialized"
    );
    *lock = Some(state);
}

/// Returns a reference to the global [`State`].
///
/// # Panics
///
/// This function panics if the global state has not been initialized.
#[inline]
#[track_caller]
pub fn state() -> ft::sync::mutex::MutexGuard<'static, State, ft::sync::mutex::NoBlockMutex> {
    ft::sync::mutex::MutexGuard::map(STATE.lock(), |opt| {
        opt.as_mut()
            .expect("the global state has not been initialized")
    })
}

/// Registers the `clear_state` function to be called when the program exits.
extern "C" fn setup_clear_state() {
    extern "C" fn clear_state() {
        if let Some(mut st) = STATE.try_lock() {
            *st = None;
        }
    }

    ft::at_exit(clear_state);
}
ft::ctor!(setup_clear_state);
