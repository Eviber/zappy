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

/// A command that a player may attempt to execute.
#[derive(Debug)]
pub enum Command {
    /// The `avance` command.
    MoveForward,
    /// The `gauche` command.
    TurnLeft,
    /// The `droite` command.
    TurnRight,
    /// The `voir` command.
    LookAround,
    /// The `inventaire` command.
    Inventory,
    /// The `prend` command.
    PickUpObject(ObjectClass),
    /// The `pose` command.
    DropObject(ObjectClass),
    /// The `expulse` command.
    KnockPlayer,
    /// The `broadcast` command.
    Broadcast(Box<[u8]>),
    /// The `incantation` command.
    Evolve,
    /// The `fork` command.
    LayAnEgg,
    /// The `connect_nbr` command.
    AvailableTeamSlots,
}

/// A command scheduled to be executed in the future.
struct ScheduledCommand {
    /// The player that scheduled the command.
    player: PlayerId,
    /// The command to execute.
    command: Command,
    /// The remaining number of ticks before the command is executed.
    remaining_ticks: u32,
}

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
    /// The commands that have been scheduled to be executed in the future.
    commands: Vec<ScheduledCommand>,
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
            commands: Vec::new(),
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

    /// Returns the index of a player in the list of players.
    #[inline]
    fn player_index_by_id(&self, player: PlayerId) -> Option<usize> {
        self.players.iter().position(|p| p.player_id == player)
    }

    /// Returns the state of the player with the provided ID.
    fn player_mut(&mut self, player: PlayerId) -> &mut PlayerState {
        self.player_index_by_id(player)
            .and_then(|i| self.players.get_mut(i))
            .expect("no player with the provided ID")
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

        // Remove commands scheduled by the player.
        self.commands.retain(|cmd| cmd.player != player.player_id);
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

    /// Schedules a command to be executed in the future.
    pub fn schedule_command(&mut self, player: PlayerId, command: Command) {
        let ticks = match &command {
            Command::MoveForward => 7,
            Command::TurnLeft => 7,
            Command::TurnRight => 7,
            Command::LookAround => 7,
            Command::Inventory => 1,
            Command::PickUpObject(_) => 7,
            Command::DropObject(_) => 7,
            Command::KnockPlayer => 7,
            Command::Broadcast(_) => 7,
            Command::Evolve => 300,
            Command::LayAnEgg => 42,
            Command::AvailableTeamSlots => 0,
        };

        self.commands.push(ScheduledCommand {
            player,
            command,
            remaining_ticks: ticks,
        });
    }

    /// Notifies the state that a whole tick has passed.
    pub async fn tick(&mut self) -> ft::Result<()> {
        let mut i = 0;
        loop {
            let Some(cmd) = self.commands.get_mut(i) else {
                break Ok(());
            };

            if cmd.remaining_ticks > 0 {
                cmd.remaining_ticks -= 1;
                i += 1;
                continue;
            }

            // We can't optimize this with a `swap_remove` because the order of
            // commands must be preserved.
            let cmd = self.commands.remove(i);

            ft_log::trace!("executing command for #{}: {:?}", cmd.player, cmd.command);

            self.player_mut(cmd.player).sender.ok().await?;
        }
    }

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
pub fn state() -> ft::sync::mutex::Guard<'static, State, ft::sync::mutex::NoBlockLock> {
    STATE
        .get()
        .expect("the global state has not been initialized")
        .lock()
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
