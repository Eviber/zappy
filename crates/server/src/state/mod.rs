//! Defines the global state of the server.

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt::Write;

use ft::collections::ArrayVec;

use crate::args::Args;
use crate::client::Client;
use crate::player::PlayerError;
use crate::state::rng::Rng;

mod rng;
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
}

/// A response that can be sent back to a player.
pub enum Response {
    /// The string `"ok"`.
    Ok,
    /// The string `"ko"`.
    Ko,
    /// Inventory of the player
    // todo box where needed
    Inventory(PlayerInventory),
    /// What the player sees
    Sight([Option<WorldCell>; 81]),
    /// The number of available slots in the team.
    ConnectNbr(u32),
}

impl Response {
    /// Sends the response to the specified file descriptor.
    pub async fn send_to(&self, fd: ft::Fd, buf: &mut String) -> ft::Result<()> {
        match self {
            Response::Ok => ft_async::futures::write_all(fd, b"ok\n").await?,
            Response::Ko => ft_async::futures::write_all(fd, b"ko\n").await?,
            Response::Inventory(inventory) => {
                let result = writeln!(
                    buf,
                    "{{nourriture {}, linemate {}, deraumere {}, sibur {}, mendiane {}, phiras {}, thystame {}}}",
                    inventory.food,
                    inventory.linemate,
                    inventory.deraumere,
                    inventory.sibur,
                    inventory.mendiane,
                    inventory.phiras,
                    inventory.thystame,
                );
                debug_assert!(result.is_ok(), "writing to a string should never fail");
                ft_async::futures::write_all(fd, buf.as_bytes()).await?
            }
            Response::Sight(sight) => {
                let mut result = write!(buf, "{{");
                debug_assert!(result.is_ok(), "writing to a string should never fail");
                for i in 0..81 {
                    match sight[i] {
                        Some(cell) => {
                            for _ in 0..cell.food {
                                result = write!(buf, "nourriture ");
                                debug_assert!(
                                    result.is_ok(),
                                    "writing to a string should never fail"
                                );
                            }
                            for _ in 0..cell.linemate {
                                result = write!(buf, "linemate ");
                                debug_assert!(
                                    result.is_ok(),
                                    "writing to a string should never fail"
                                );
                            }
                            for _ in 0..cell.deraumere {
                                result = write!(buf, "deraumere ");
                                debug_assert!(
                                    result.is_ok(),
                                    "writing to a string should never fail"
                                );
                            }
                            for _ in 0..cell.sibur {
                                result = write!(buf, "sibur ");
                                debug_assert!(
                                    result.is_ok(),
                                    "writing to a string should never fail"
                                );
                            }
                            for _ in 0..cell.mendiane {
                                result = write!(buf, "mendiane ");
                                debug_assert!(
                                    result.is_ok(),
                                    "writing to a string should never fail"
                                );
                            }
                            for _ in 0..cell.phiras {
                                result = write!(buf, "phiras ");
                                debug_assert!(
                                    result.is_ok(),
                                    "writing to a string should never fail"
                                );
                            }
                            for _ in 0..cell.thystame {
                                result = write!(buf, "thystame ");
                                debug_assert!(
                                    result.is_ok(),
                                    "writing to a string should never fail"
                                );
                            }
                        }
                        None => break,
                    }
                    if buf.len() > 1 {
                        buf.pop();
                    }
                    result = write!(buf, ", ");
                    debug_assert!(result.is_ok(), "writing to a string should never fail");
                }
                buf.pop();
                buf.pop();
                result = writeln!(buf, "}}");
                debug_assert!(result.is_ok(), "writing to a string should never fail");
                ft_async::futures::write_all(fd, buf.as_bytes()).await?;
            }
            Response::ConnectNbr(nbr) => {
                // NOTE: This cannot fail because writing to a string in this way will panic in case
                // of memory allocation failure instead of returning an error.
                let result = writeln!(buf, "{}", nbr);
                debug_assert!(result.is_ok(), "writing to a string should never fail");
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

#[derive(Debug, Clone, Copy)]
struct PlayerInventory {
    /// Food.
    food: u32,
    /// Linemate.
    linemate: u32,
    /// Deraumere.
    deraumere: u32,
    /// Sibur.
    sibur: u32,
    /// Mendiane.
    mendiane: u32,
    /// Phiras.
    phiras: u32,
    /// Thystame.
    thystame: u32,
}

impl PlayerInventory {
    pub fn new() -> Self {
        Self {
            food: 10,
            linemate: 0,
            deraumere: 0,
            sibur: 0,
            mendiane: 0,
            phiras: 0,
            thystame: 0,
        }
    }
}

/// A direction in which the player can be facing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerDirection {
    /// The player faces the positive Y direction.
    North,
    /// The player faces the negative Y direction.
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
    /// A direction in which the player is facing.
    facing: PlayerDirection,
    /// Current player elevation.
    level: u32,
    /// Current inventory of the player on the inventory axis.
    /// Indices follow world::ObjectClass order...
    inventory: PlayerInventory,
    /// Current position of the player on the horizontal axis.
    x: usize,
    /// Current position of the player on the vertical axis.
    y: usize,
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

    /// Turns the player right.
    #[inline]
    pub fn turn_right(&mut self) {
        self.facing = self.facing.turn_right();
    }

    /// Turns the player left.
    #[inline]
    pub fn turn_left(&mut self) {
        self.facing = self.facing.turn_left();
    }

    /// Advances the player's position based on their current direction.
    pub fn advance_position(&mut self, width: usize, height: usize) {
        match self.facing {
            PlayerDirection::North if self.y == height - 1 => self.y = 0,
            PlayerDirection::North => self.y += 1,
            PlayerDirection::South if self.y == 0 => self.y = height - 1,
            PlayerDirection::South => self.y -= 1,
            PlayerDirection::West if self.x == 0 => self.x = width - 1,
            PlayerDirection::West => self.x -= 1,
            PlayerDirection::East if self.x == width - 1 => self.x = 0,
            PlayerDirection::East => self.x += 1,
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

        let mut rng = Rng::from_urandom().unwrap_or(Rng::new(0xdeadbeef));
        let world = World::new(args.width, args.height, &mut rng);

        Self {
            teams,
            players: Vec::new(),
            world,
            rng,
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

        let x = self.rng.next_u64() as usize % self.world.width;
        let y = self.rng.next_u64() as usize % self.world.height;

        self.world.cells[x + y * self.world.width].player_count += 1;

        // TODO when implementing fork/egg count the time elapsed before actually
        // connecting and adapt food/time to live in inventory
        self.players.push(Box::new(PlayerState {
            player_id: client.id(),
            team_id,
            conn: client.fd(),
            commands: ArrayVec::new(),
            facing: match self.rng.next_u64() % 4 {
                0 => PlayerDirection::North,
                1 => PlayerDirection::South,
                2 => PlayerDirection::East,
                3 => PlayerDirection::West,
                _ => unreachable!(),
            },
            inventory: PlayerInventory::new(),
            level: 1,
            x,
            y,
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
                self.world.cells[player.x + player.y * self.world.width].player_count -= 1;
                player.advance_position(self.world.width, self.world.height);
                self.world.cells[player.x + player.y * self.world.width].player_count += 1;
                Response::Ok
            }
            Command::Inventory => Response::Inventory(player.inventory),
            Command::LookAround => {
                let mut sight = [None; 81];

                sight[0] = Some(self.world.cells[player.x + player.y * self.world.width]);
                let mut level_tool = 1;
                // dir represents 2 2d vectors, the offset dir per level, and the offset dir per case inside that level
                let dir: (i32, i32) = match player.facing {
                    PlayerDirection::North => (0, 1),
                    PlayerDirection::South => (0, -1),
                    PlayerDirection::East => (1, 0),
                    PlayerDirection::West => (-1, 0),
                };
                for i in 1..(player.level + 1) * (player.level + 1) {
                    if level_tool * level_tool < (i + 1) {
                        level_tool += 1;
                    }
                    let level_offset = (level_tool - 1) as i32;
                    let level_index = (i as i32 + 1) - level_offset * level_offset;

                    // trick to reduce tuple size used here
                    let mut x_sight = player.x as i32
                        + level_offset * dir.0
                        + (level_index - level_offset - 1) * dir.1;
                    let mut y_sight = player.y as i32 + level_offset * dir.1
                        - (level_index - level_offset - 1) * dir.0;
                    // these << 4 lines are because if world size is smaller than some number index out of bounds might occur
                    x_sight += (self.world.width as i32) << 4;
                    x_sight %= self.world.width as i32;
                    y_sight += (self.world.height as i32) << 4;
                    y_sight %= self.world.height as i32;
                    sight[i as usize] = Some(
                        self.world.cells[(x_sight + y_sight * self.world.width as i32) as usize],
                    );
                }
                Response::Sight(sight)
            }
            _ => Response::Ko,
        }
    }

    /// Notifies the state that a whole tick has passed.
    ///
    /// # Arguments
    ///
    /// - `responses` - a list of responses that must be sent to their associated file
    ///   descriptiors.
    #[allow(clippy::unwrap_used)]
    pub fn tick(&mut self, responses: &mut Vec<(ft::Fd, Response)>) {
        for player_index in 0..self.players.len() {
            let player = &mut self.players[player_index];
            let Some(command) = player.commands.first_mut() else {
                continue;
            };

            if command.remaining_ticks > 0 {
                command.remaining_ticks -= 1;
                continue;
            }

            // This unwrap can't ever fail because the case where there is no
            // first element is handled above.
            // Also, we can't optimize this with a swap_remove because the
            // order in which commands are inserted matters. Maybe we can use
            // a VecDeque instead, but that would be vastly overkill for those
            // 10 elements.
            let cmd = player.commands.remove(0).unwrap();

            // Execute the command.
            ft_log::trace!(
                "executing command for #{}: {:?}",
                player.player_id,
                cmd.command,
            );

            let player_conn = player.conn;

            let response = self.execute_command(cmd.command, player_index);
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
