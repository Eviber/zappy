use {
    super::{Command, PlayerId},
    crate::{
        client::Client,
        rng::Rng,
        state::{PlayerDirection, TeamId},
    },
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

/// Defines the state that is kept per-player.
pub struct PlayerState {
    /// The ID of the player.
    ///
    /// This is a unique identifier that is assigned to each player when they connect to the
    /// server.
    player_id: PlayerId,
    /// The ID of the team the player is in.
    team_id: TeamId,
    /// The commands that have been buffered for the player.
    commands: ArrayVec<ScheduledCommand, 10>,

    /// The connection that was open with the player.
    pub conn: ft::Fd,

    /// The direction in which the player is facing.
    pub facing: PlayerDirection,
    /// Current position of the player on the horizontal axis.
    pub x: u32,
    /// Current position of the player on the vertical axis.
    pub y: u32,
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
        width: u32,
        height: u32,
    ) -> Self {
        Self {
            player_id: client.id(),
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
            x: rng.next_u64() as u32 % width,
            y: rng.next_u64() as u32 % height,
        }
    }

    /// Returns the ID of the player.
    #[inline]
    pub fn id(&self) -> PlayerId {
        self.player_id
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

    /// Advances the player's position based on their current direction.
    pub fn advance_position(&mut self, width: u32, height: u32) {
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
