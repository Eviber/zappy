//! Defines the global state of the server.

use alloc::boxed::Box;
use alloc::vec::Vec;
use ft_async::sync::channel::Sender;

use crate::args::Args;
use crate::player::{Player, PlayerError, PlayerMsg};

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

/// The global state of the server, responsible for managing the clients and the game.
pub struct State {
    /// Information about the teams available in the current game.
    teams: Box<[Team]>,
    /// The players currently connected to the server.
    players: Vec<Player>,
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

    /// Registers a player to the server, joining the specified team.
    ///
    /// # Arguments
    ///
    /// - `conn`: the file descriptor of the client.
    ///
    /// - `sender`: the channel used to send messages to the player.
    ///
    /// - `name`: the name of the team the player wants to join.
    pub fn try_join_team(
        &mut self,
        sender: Sender<PlayerMsg>,
        name: &str,
    ) -> Result<TeamId, PlayerError> {
        let team_id = self
            .teams
            .iter()
            .position(|team| &*team.name == name)
            .ok_or_else(|| PlayerError::UnknownTeam(name.into()))?;

        let team = &mut self.teams[team_id];

        if team.available_slots == 0 {
            return Err(PlayerError::TeamFull {
                name: team.name.clone(),
                id: team_id,
            });
        }

        team.available_slots -= 1;

        self.players.push(Player { sender, team_id });

        Ok(team_id)
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
