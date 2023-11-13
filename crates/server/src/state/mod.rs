//! Defines the global state of the server.

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::args::Args;
use crate::client::Client;
use crate::player::{PlayerError, PlayerSender};

mod world;

pub use self::world::*;

/// The ID of a team.
pub type TeamId = usize;

/// Information about the state of a team.
pub struct Team {
    /// The name of the team.
    name: Box<str>,
    /// The number of available slots in the team.
    available_slots: u32,
}

/// The ID of a player.
pub type PlayerId = usize;

/// The state of a player.
pub struct PlayerState {
    /// The ID of the player.
    player_id: PlayerId,
    /// The ID of the team the player is in.
    team_id: TeamId,
    /// The sending half of the channel used to send messages to the player.
    sender: PlayerSender,
}

/// The global state of the server, responsible for managing the clients and the game.
pub struct State {
    /// Information about the teams available in the current game.
    teams: Box<[Team]>,
    /// The list of players currently connected to the server.
    players: Vec<PlayerState>,
    /// The current state of the world.
    world: World,
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

        self.players.push(PlayerState {
            player_id: client.id(),
            team_id,
            sender: PlayerSender::new(client.fd()),
        });

        Ok(client.id())
    }

    /// Removes a player from the server.
    pub fn leave(&mut self, player: PlayerId) {
        let index = self
            .players
            .iter()
            .position(|p| p.player_id == player)
            .expect("no player with the provided ID found");

        let player = self.players.remove(index);
        self.teams[player.team_id].available_slots += 1;
    }

    /// Returns the number of available slots in the specified team.
    #[inline]
    pub fn available_slots_for(&self, team: TeamId) -> u32 {
        self.teams[team].available_slots
    }

    /// Returns the current state of the world.
    #[inline]
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Moves the player forward.
    pub fn move_forward(&mut self, player_id: PlayerId) {
        unimplemented!();
    }

    /// Turns the player to the right.
    pub fn turn_right(&mut self, player_id: PlayerId) {
        unimplemented!();
    }

    /// Turns the player to the left.
    pub fn turn_left(&mut self, player_id: PlayerId) {
        unimplemented!();
    }

    /// Allow the player to look around..
    pub fn look_around(&mut self, player_id: PlayerId) {
        unimplemented!();
    }

    /// Allow the player to look into their inventory.
    pub fn inventory(&mut self, player_id: PlayerId) {
        unimplemented!();
    }

    /// Allow the player to pick up an object.
    pub fn pick_up_object(
        &mut self,
        player_id: PlayerId,
        object: ObjectClass,
    ) -> Result<(), PlayerError> {
        unimplemented!();
    }

    /// Allow the player to drop an object.
    pub fn drop_object(
        &mut self,
        player_id: PlayerId,
        object: ObjectClass,
    ) -> Result<(), PlayerError> {
        unimplemented!();
    }

    /// Allow the player to expel another player.
    pub fn knock(&mut self, player_id: PlayerId) {
        unimplemented!();
    }

    /// Allow the player to broadcast a message.
    pub fn broadcast(&mut self, player_id: PlayerId, message: &[u8]) {
        unimplemented!();
    }

    /// Allow the player to lay an egg.
    pub fn lay_egg(&mut self, player_id: PlayerId) {
        unimplemented!();
    }

    /// Allow the player to evolve.
    pub fn evolve(&mut self, player_id: PlayerId) {
        unimplemented!();
    }

    /// Notifies the state that a whole tick has passed.
    pub fn tick(&mut self) {}

    /// Clears the whole state, deallocating all the resources it uses.
    pub fn clear(&mut self) {
        self.teams = Box::new([]);
    }
}

/// The global state of the server.
static STATE: ft::OnceCell<ft::Mutex<State, ft::sync::mutex::NoBlockLock>> = ft::OnceCell::new();

/// Initializes the global [`State`].
///
/// # Panics
///
/// This function panics if the global state is already initialized.
#[inline]
pub fn set_state(state: State) {
    STATE
        .set(ft::Mutex::new(state))
        .ok()
        .expect("the global state has already been initialized");
}

/// Returns a reference to the global [`State`].
///
/// # Panics
///
/// This function panics if the global state has not been initialized.
#[inline]
pub fn state() -> &'static ft::Mutex<State, ft::sync::mutex::NoBlockLock> {
    STATE
        .get()
        .expect("the global state has not been initialized")
}

/// Registers the `clear_state` function to be called when the program exits.
extern "C" fn setup_clear_state() {
    extern "C" fn clear_state() {
        if let Some(st) = STATE.get() {
            st.lock().clear();
        }
    }

    ft::at_exit(clear_state);
}
ft::ctor!(setup_clear_state);
