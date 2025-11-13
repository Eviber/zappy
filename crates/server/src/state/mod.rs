//! Defines the global state of the server.

use {crate::player::PlayerState, core::fmt::Write};
use {
    crate::{player::PlayerId, rng::Rng},
    slotmap::SlotMap,
};
use {alloc::boxed::Box, core::fmt::Display};
use {alloc::vec::Vec, core::time::Duration};

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
    /// The player faces the positive Y direction.
    North,
    /// The player faces the positive X direction.
    East,
    /// The player faces the negative Y direction.
    South,
    /// The player faces the negative X direction.
    West,
}

impl PlayerDirection {
    /// Turns the direction right.
    pub fn turn_right(self) -> Self {
        match self {
            PlayerDirection::North => PlayerDirection::East,
            PlayerDirection::East => PlayerDirection::South,
            PlayerDirection::South => PlayerDirection::West,
            PlayerDirection::West => PlayerDirection::North,
        }
    }

    /// Turns the direction left.
    pub fn turn_left(self) -> Self {
        match self {
            PlayerDirection::North => PlayerDirection::West,
            PlayerDirection::East => PlayerDirection::North,
            PlayerDirection::South => PlayerDirection::East,
            PlayerDirection::West => PlayerDirection::South,
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
pub struct State {
    /// Information about the teams available in the current game.
    pub teams: Box<[Team]>,
    /// The list of players currently connected to the server.
    pub players: SlotMap<PlayerId, PlayerState>,
    /// The current state of the world.
    pub world: World,
    /// The random number generator used by the server.
    pub rng: Rng,
    /// The list of graphics monitors that have subscribed to the server.
    pub gfx_monitors: Vec<ft::Fd>,
    /// The duration between each tick of the world.
    pub tick_duration: Duration,
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
            players: SlotMap::default(),
            world,
            rng: Rng::from_urandom().unwrap_or(Rng::new(0xdeadbeef)),
            gfx_monitors: Vec::new(),
            tick_duration: Duration::from_secs_f32(1.0 / args.tick_frequency),
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

        let player_id = self.players.insert(PlayerState::new_random(
            client,
            team_id,
            &mut self.rng,
            self.world.width,
            self.world.height,
        ));

        Ok(player_id)
    }

    /// Removes a player from the server.
    ///
    /// # Panics
    ///
    /// This function panics if the player does not exist.
    #[track_caller]
    pub fn leave(&mut self, player_id: PlayerId) {
        let player = self
            .players
            .remove(player_id)
            .expect("Attempted to remove non-existent player");
        self.teams[player.team_id()].available_slots += 1;
    }

    /// Returns the number of available slots in the specified team.
    #[inline]
    pub fn available_slots_for(&self, team: TeamId) -> u32 {
        self.teams[team].available_slots
    }

    /// Notifies the state that a whole tick has passed.
    ///
    /// # Arguments
    ///
    /// - `responses` - a list of responses that must be sent to their associated file
    ///   descriptiors.
    pub async fn tick(&mut self) {
        let player_ids: Vec<PlayerId> = self.players.keys().collect();

        for id in player_ids {
            let Some(cmd) = self.players[id].try_unqueue_command() else {
                continue;
            };

            // Execute the command.
            ft_log::trace!("executing command for {}: {:?}", id, cmd);

            if let Err(err) = cmd.execute(id, self).await {
                ft_log::error!("failed to execute command for player {}: {}", id, err);
            }
        }
    }

    /// Broadcasts a message to all registered graphics monitors.
    pub async fn broadcast_to_graphics_monitors(&self, data: &[u8]) {
        for monitor_fd in &self.gfx_monitors {
            if let Err(err) = monitor_fd.async_write_all(data).await {
                ft_log::error!(
                    "failed to broadcast to graphics monitor {}: {}",
                    monitor_fd.to_raw(),
                    err
                );
            };
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
