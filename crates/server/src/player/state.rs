use {
    super::Command,
    crate::{
        client::Client,
        rng::Rng,
        state::{ObjectClass, PlayerDirection, TeamId},
    },
    core::ops::{Index, IndexMut},
    ft::collections::ArrayVec,
};

/// A command that has been scheduled to be executed in the future.
#[derive(Debug)]
struct ScheduledCommand {
    /// The command that has been scheduled.
    pub command: Command,
    /// The number of ticks remaining before the command is executed.
    pub remaining_ticks: u32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct PlayerInventory {
    /// Food.
    pub time_to_live: u32,
    /// Linemate.
    pub linemate: u32,
    /// Deraumere.
    pub deraumere: u32,
    /// Sibur.
    pub sibur: u32,
    /// Mendiane.
    pub mendiane: u32,
    /// Phiras.
    pub phiras: u32,
    /// Thystame.
    pub thystame: u32,
}

impl Index<ObjectClass> for PlayerInventory {
    type Output = u32;

    fn index(&self, object: ObjectClass) -> &Self::Output {
        match object {
            ObjectClass::Food => &self.time_to_live,
            ObjectClass::Linemate => &self.linemate,
            ObjectClass::Deraumere => &self.deraumere,
            ObjectClass::Sibur => &self.sibur,
            ObjectClass::Mendiane => &self.mendiane,
            ObjectClass::Phiras => &self.phiras,
            ObjectClass::Thystame => &self.thystame,
        }
    }
}

impl IndexMut<ObjectClass> for PlayerInventory {
    fn index_mut(&mut self, object: ObjectClass) -> &mut Self::Output {
        match object {
            ObjectClass::Food => &mut self.time_to_live,
            ObjectClass::Linemate => &mut self.linemate,
            ObjectClass::Deraumere => &mut self.deraumere,
            ObjectClass::Sibur => &mut self.sibur,
            ObjectClass::Mendiane => &mut self.mendiane,
            ObjectClass::Phiras => &mut self.phiras,
            ObjectClass::Thystame => &mut self.thystame,
        }
    }
}

impl PlayerInventory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            time_to_live: 1260,
            linemate: 0,
            deraumere: 0,
            sibur: 0,
            mendiane: 0,
            phiras: 0,
            thystame: 0,
        }
    }

    pub fn get_food(&self) -> u32 {
        self.time_to_live / 126
    }
}

/// Defines the state that is kept per-player.
pub struct PlayerState {
    /// The ID of the team the player is in.
    team_id: TeamId,
    /// The commands that have been buffered for the player.
    commands: ArrayVec<ScheduledCommand, 10>,

    /// The connection that was open with the player.
    pub conn: ft::Fd,

    /// The direction in which the player is facing.
    pub facing: PlayerDirection,
    /// Current position of the player on the horizontal axis.
    pub x: usize,
    /// Current position of the player on the vertical axis.
    pub y: usize,
    /// Items currently held by the player
    pub inventory: PlayerInventory,
    /// The current level of the player.
    pub level: usize,

    /// Indicates that the player is currently leveling up.
    ///
    /// When `true`, the player cannot do anything.
    pub is_leveling_up: bool,
}

impl PlayerState {
    /// Creates a new player state.
    ///
    /// # Parameters
    ///
    /// * `player_id` - The ID of the player.
    /// * `team_id` - The ID of the team the player is in.
    /// * `conn` - The connection that was open with the player.
    /// * `rng` - The random number generator to use.
    /// * `width` - The width of the world (maximum X coordinate).
    /// * `height` - The height of the world (maximum Y coordinate).
    pub fn new_random(
        client: &Client,
        team_id: TeamId,
        rng: &mut Rng,
        width: usize,
        height: usize,
    ) -> Self {
        Self {
            team_id,
            commands: ArrayVec::new(),
            conn: client.fd(),
            facing: match rng.next_u64() % 4 {
                0 => PlayerDirection::North,
                1 => PlayerDirection::East,
                2 => PlayerDirection::South,
                3 => PlayerDirection::West,
                _ => unreachable!(),
            },
            x: rng.next_u64() as usize % width,
            y: rng.next_u64() as usize % height,
            level: 1,
            inventory: PlayerInventory::new(),
            is_leveling_up: false,
        }
    }

    /// Returns the ID of the team the player is in.
    #[inline]
    pub fn team_id(&self) -> TeamId {
        self.team_id
    }

    /// Attempts to pop a command from the command queue.
    ///
    /// If the top-level command is ready to run, then it is returned. Otherwise, this function
    /// returns `None`.
    pub fn try_unqueue_command(&mut self) -> Option<Command> {
        let scheduled_command = self.commands.first_mut()?;

        if scheduled_command.remaining_ticks > 0 {
            scheduled_command.remaining_ticks -= 1;
            return None;
        }

        self.commands.remove(0).map(|x| x.command)
    }

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

    pub fn get_front_cell(&self, width: usize, height: usize) -> (usize, usize) {
        match self.facing {
            PlayerDirection::North if self.y == height - 1 => (self.x, 0),
            PlayerDirection::North => (self.x, self.y + 1),
            PlayerDirection::South if self.y == 0 => (self.x, height - 1),
            PlayerDirection::South => (self.x, self.y - 1),
            PlayerDirection::West if self.x == 0 => (width - 1, self.y),
            PlayerDirection::West => (self.x - 1, self.y),
            PlayerDirection::East if self.x == width - 1 => (0, self.y),
            PlayerDirection::East => (self.x + 1, self.y),
        }
    }

    /// Advances the player's position based on their current direction.
    pub fn advance_position(&mut self, width: usize, height: usize) {
        let front_cell = self.get_front_cell(width, height);
        self.x = front_cell.0;
        self.y = front_cell.1;
    }
}
