//! Defines the global state of the server.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use ft::collections::ArrayVec;

use crate::args::Args;
use crate::client::Client;
use crate::player::PlayerError;

mod world;

pub use self::world::*;

/// The ID of a team.
pub type TeamId = usize;

/// A command that a player may attempt to execute.
#[derive(Debug)]
#[allow(dead_code)] // FIXME: temporary until all commands are implemented
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

impl Command {
    /// Returns the number of ticks that this command takes to execute.
    pub fn ticks(&self) -> u32 {
        match self {
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
        }
    }

    /// Executes the command, returning the response that must be sent back to the player.
    #[allow(dead_code)]
    pub fn execute(&self, player: &PlayerState, _world: &World, teams: &[Team]) -> Response {
        match self {
            Command::AvailableTeamSlots => {
                let team_id = player.team_id;
                let count = teams[team_id].available_slots;
                Response::ConnectNbr(count)
            }
            _ => Response::Ok,
        }
    }
}

/// A response that can be sent back to a player.
pub enum Response {
    /// The string `"ok"`.
    Ok,
    /// The number of available slots in the team.
    ConnectNbr(u32),
}

impl Response {
    /// Sends the response to the specified file descriptor.
    pub async fn send_to(&self, fd: ft::Fd, buf: &mut String) -> ft::Result<()> {
        match self {
            Response::Ok => ft_async::futures::write_all(fd, b"ok\n").await?,
            Response::ConnectNbr(nbr) => {
                writeln!(buf, "{}", nbr).unwrap();
                ft_async::futures::write_all(fd, buf.as_bytes()).await?
            }
        }

        Ok(())
    }
}

/// A command that has been scheduled to be executed in the future.
#[derive(Debug)]
pub struct ScheduledCommand {
    /// The command that has been scheduled.
    pub command: Command,
    /// The number of ticks remaining before the command is executed.
    pub remaining_ticks: u32,
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
    /// The connection that was open with the player.
    conn: ft::Fd,
    /// The commands that have been buffered for the player.
    commands: ArrayVec<ScheduledCommand, 10>,
}

impl PlayerState {
    /// Schedules a command for this player.
    ///
    /// # Returns
    ///
    /// `true` if the command has been scheduled, `false` if the buffer is full.
    pub fn schedule_command(&mut self, command: Command) -> bool {
        self.commands
            .try_push(ScheduledCommand {
                remaining_ticks: command.ticks(),
                command,
            })
            .is_ok()
    }
}

/// The global state of the server, responsible for managing the clients and the game.
#[allow(clippy::vec_box)] // `PlayerState` is a huge struct, copying it around is not a good idea.
pub struct State {
    /// Information about the teams available in the current game.
    teams: Box<[Team]>,
    /// The list of players currently connected to the server.
    players: Vec<Box<PlayerState>>,
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

        self.players.push(Box::new(PlayerState {
            player_id: client.id(),
            team_id,
            conn: client.fd(),
            commands: ArrayVec::new(),
        }));

        Ok(client.id())
    }

    /// Returns the index of a player in the list of players.
    #[inline]
    fn player_index_by_id(&self, player: PlayerId) -> Option<usize> {
        self.players.iter().position(|p| p.player_id == player)
    }

    /// Returns the state of the player with the provided ID.
    pub fn player_mut(&mut self, player: PlayerId) -> &mut PlayerState {
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
    ///
    /// # Arguments
    ///
    /// - `responses` - a list of responses that must be sent to their associated file
    ///   descriptiors.
    #[allow(clippy::unwrap_used)]
    pub fn tick(&mut self, responses: &mut Vec<(ft::Fd, Response)>) {
        for player in &mut self.players {
            let Some(command) = player.commands.first_mut() else {
                continue;
            };

            if command.remaining_ticks > 0 {
                command.remaining_ticks -= 1;
                continue;
            }

            // This unwrap can ever fail because the case where there is no
            // first element is handled above.
            // Also, we can't optimize this with a swap_remove because the
            // order in which commands are inserted matters. Maybe we can use
            // a VecDeque instead, but that would be vastly overkill for those
            // 10 poor elements.
            let cmd = player.commands.remove(0).unwrap();

            // Execute the command.
            ft_log::trace!(
                "executing command for #{}: {:?}",
                player.player_id,
                cmd.command,
            );

            let response = cmd.command.execute(player, &self.world, &self.teams);
            responses.push((player.conn, response));
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
