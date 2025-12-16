//! An implementation of the Zappy API.

use {
    anyhow::Context,
    parking_lot::{Mutex, RwLock},
    std::{
        ops::{Add, Deref, DerefMut},
        str::FromStr,
        sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        },
        time::Duration,
    },
    tokio::{
        io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
        net::{
            TcpStream,
            tcp::{OwnedReadHalf, OwnedWriteHalf},
        },
    },
};

/// The maximum number of pending commands the server can accept before starting to drop requests.
pub const MAX_PENDING_COMMANDS: usize = 10;

/// A direction in which a broadcasted message can be received.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BroadcastDirection {
    Center,
    Front,
    FrontLeft,
    FrontRight,
    BackLeft,
    BackRight,
    Left,
    Right,
    Back,
}

/// Describes the content of a cell.
#[derive(Debug, Default, Clone, Copy)]
pub struct CellContent {
    /// The number of players in the cell, not including the current player.
    pub player: u32,
    /// The number of food in the cell.
    pub food: u32,
    /// The number of linemate in the cell.
    pub linemate: u32,
    /// The number of deraumere in the cell.
    pub deraumere: u32,
    /// The number of sibur in the cell.
    pub sibur: u32,
    /// The number of mendiane in the cell.
    pub mendiane: u32,
    /// The number of phiras in the cell.
    pub phiras: u32,
    /// The number of thystame in the cell.
    pub thystame: u32,
}

impl Add for CellContent {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            player: self.player + other.player,
            food: self.food + other.food,
            linemate: self.linemate + other.linemate,
            deraumere: self.deraumere + other.deraumere,
            sibur: self.sibur + other.sibur,
            mendiane: self.mendiane + other.mendiane,
            phiras: self.phiras + other.phiras,
            thystame: self.thystame + other.thystame,
        }
    }
}

impl FromStr for CellContent {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut output = Self::default();

        for part in s.split_ascii_whitespace() {
            match part {
                "player" => output.player += 1,
                "food" => output.food += 1,
                "linemate" => output.linemate += 1,
                "deraumere" => output.deraumere += 1,
                "sibur" => output.sibur += 1,
                "mendiane" => output.mendiane += 1,
                "phiras" => output.phiras += 1,
                "thystame" => output.thystame += 1,
                _ => anyhow::bail!("Invalid cell element name: {}", part),
            }
        }

        Ok(output)
    }
}

/// The type of an item.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ItemType {
    Food,
    Linemate,
    Deraumere,
    Sibur,
    Mendiane,
    Phiras,
    Thystame,
}

impl ItemType {
    /// Returns the name of the item type.
    pub const fn name(self) -> &'static str {
        match self {
            ItemType::Food => "nourriture",
            ItemType::Linemate => "linemate",
            ItemType::Deraumere => "deraumere",
            ItemType::Sibur => "sibur",
            ItemType::Mendiane => "mendiane",
            ItemType::Phiras => "phiras",
            ItemType::Thystame => "thystame",
        }
    }
}

/// An event received from the server.
#[derive(Debug, Clone)]
pub enum Event {
    /// A message has been broadcasted from a particular direction.
    BroadcastMessage {
        /// The direction from which the message was received.
        direction: BroadcastDirection,
        /// The content of the message.
        content: Box<[u8]>,
    },
}

/// A direction a player can face.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerDirection {
    Front,
    Back,
    Right,
    Left,
}

impl PlayerDirection {
    /// Returns the direction rotated to the left.
    pub fn rotated_left(self) -> Self {
        match self {
            PlayerDirection::Front => PlayerDirection::Left,
            PlayerDirection::Back => PlayerDirection::Right,
            PlayerDirection::Right => PlayerDirection::Back,
            PlayerDirection::Left => PlayerDirection::Front,
        }
    }

    /// Returns the direction rotated to the right.
    pub fn rotated_right(self) -> Self {
        match self {
            PlayerDirection::Front => PlayerDirection::Right,
            PlayerDirection::Back => PlayerDirection::Left,
            PlayerDirection::Right => PlayerDirection::Front,
            PlayerDirection::Left => PlayerDirection::Back,
        }
    }

    /// Rotates the provided vector by the direction.
    pub fn rotate_vector(self, v: (i32, i32)) -> (i32, i32) {
        match self {
            PlayerDirection::Front => (v.0, v.1),
            PlayerDirection::Back => (-v.0, -v.1),
            PlayerDirection::Right => (v.1, -v.0),
            PlayerDirection::Left => (-v.1, v.0),
        }
    }

    /// Returns a vector in the direction.
    ///
    /// `front` is positive X.
    pub fn to_vector(self) -> (i32, i32) {
        self.rotate_vector((1, 0))
    }
}

/// Returns the current state of the game.
#[derive(Debug, Clone)]
pub struct GameState {
    /// The width of the map.
    pub width: u32,
    /// The height of the map.
    pub height: u32,
    /// The remaining number of players that can join the game in our team.
    pub available_team_slots: u32,

    /// The current level of the player.
    pub player_level: u32,

    /// The number of food items we have.
    pub food_count: u32,
    /// The number of linemates we have.
    pub linemate_count: u32,
    /// The number of deraumeres we have.
    pub deraumere_count: u32,
    /// The number of sibur we have.
    pub sibur_count: u32,
    /// The number of mendeianes we have.
    pub mendiane_count: u32,
    /// The number of phiras we have.
    pub phiras_count: u32,
    /// The number of thystame we have.
    pub thystame_count: u32,

    /// The current direction of the player, relative to the player's initial direction.
    pub player_direction: PlayerDirection,

    /// The content of the world.
    ///
    /// # Remarks
    ///
    /// The origin of the world is specified as our own initial position and orientation.
    /// The initial orientation is in the positive X direction.
    ///
    /// The size of this world does not depend on the real world's width and height. Instead, it's
    /// a square of the world's largest side. This does mean that some of the cells may be
    /// stored multiple times in multiple states, but this is unavoidable given the nature of the
    /// information we receive from the server.
    pub world_contents: Box<[CellContent]>,

    /// The position of the player, relative to the player's initial position.
    pub player_position_x: i32,
    /// The position of the player, relative to the player's initial position.
    pub player_position_y: i32,
}

impl GameState {
    /// Initializes a new [`GameState`] instance from the provided handshake.
    fn from_handshake(handshake: &Handshake) -> Self {
        let max_side = handshake.width.max(handshake.height);
        let world_contents = std::iter::repeat_with(CellContent::default)
            .take(max_side as usize * max_side as usize)
            .collect();

        Self {
            width: handshake.width,
            height: handshake.height,
            available_team_slots: handshake.available_team_slots,
            player_level: 1,
            food_count: 0,
            linemate_count: 0,
            deraumere_count: 0,
            sibur_count: 0,
            mendiane_count: 0,
            phiras_count: 0,
            thystame_count: 0,
            player_direction: PlayerDirection::Front,
            player_position_x: 0,
            player_position_y: 0,
            world_contents,
        }
    }

    /// Gets a mutable reference to a cell.
    pub fn get_cell_mut(&mut self, x: i32, y: i32) -> &mut CellContent {
        let max_side = self.width.max(self.height) as usize;
        let wrapped_x = x.rem_euclid(max_side as i32) as usize;
        let wrapped_y = y.rem_euclid(max_side as i32) as usize;
        &mut self.world_contents[wrapped_x + wrapped_y * max_side]
    }

    /// Gets a reference to a cell.
    pub fn get_cell(&self, x: i32, y: i32) -> &CellContent {
        let max_side = self.width.max(self.height) as usize;
        let wrapped_x = x.rem_euclid(max_side as i32) as usize;
        let wrapped_y = y.rem_euclid(max_side as i32) as usize;
        &self.world_contents[wrapped_x + wrapped_y * max_side]
    }

    /// Gets a mutable reference to a cell, relative to the current player's position
    /// and direction.
    pub fn get_cell_relative_mut(&mut self, mut dx: i32, mut dy: i32) -> &mut CellContent {
        (dx, dy) = self.player_direction.rotate_vector((dx, dy));
        self.get_cell_mut(self.player_position_x + dx, self.player_position_y + dy)
    }

    /// Gets a reference to a cell, relative to the current player's position
    /// and direction.
    pub fn get_cell_relative(&self, dx: i32, dy: i32) -> &CellContent {
        let (dx, dy) = self.player_direction.rotate_vector((dx, dy));
        self.get_cell(self.player_position_x + dx, self.player_position_y + dy)
    }
}

/// The type of a request. This is used to interpret responses sent by the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestType {
    #[doc(alias = "avance")]
    MoveForward,
    #[doc(alias = "droite")]
    TurnRight,
    #[doc(alias = "gauche")]
    TurnLeft,
    #[doc(alias = "voir")]
    See,
    #[doc(alias = "inventaire")]
    Inventory,
    #[doc(alias = "prend")]
    Pickup,
    #[doc(alias = "pose")]
    Drop,
    #[doc(alias = "expulse")]
    Kick,
    Broadcast,
    #[doc(alias = "incantation")]
    Incantation,
    Fork,
    #[doc(alias = "connect_nbr")]
    AvailableTeamSlots,
}

/// The state that is shared between the reader and writer halves of the [`ZappyClient`].
#[derive(Debug)]
struct SharedState {
    /// The list of events that haven't been handled yet.
    pub unhandled_events: Mutex<Vec<Event>>,
    /// The current state of the game.
    pub game_state: RwLock<GameState>,
    /// A boolean indicating that the client has been dropped and that the reader task should
    /// terminate.
    pub is_dropped: AtomicBool,
}

/// Contains the state to interact with a Zappy server.
pub struct ZappyClient {
    /// The list of requests that were sent to the server currently expecting a response.
    pending_request_sender: tokio::sync::mpsc::Sender<RequestType>,

    /// The open connection to the server.
    writer: OwnedWriteHalf,
    /// The shared state between the reader and writer halves.
    state: Arc<SharedState>,

    /// A temporary buffer for storing unhandled events while they are being processed
    /// by the user.
    local_unhandled_events: Vec<Event>,
}

impl ZappyClient {
    /// Creates a new Zappy client using the provided connected stream.
    pub async fn new(stream: TcpStream, team_name: &str) -> anyhow::Result<Self> {
        let mut stream = BufReader::new(stream);

        //
        // Perform the initial server handshake to get information about the current state of the
        // server.
        //
        let handshake = perform_handshake(&mut stream, team_name).await?;

        //
        // Split the stream into a reader and a writer half. The reader will go on a separate
        // task to handle server responses.
        //
        let (reader, writer) = stream.into_inner().into_split();

        //
        // Create the shared state.
        //
        let state = Arc::new(SharedState {
            unhandled_events: Mutex::new(Vec::new()),
            game_state: RwLock::new(GameState::from_handshake(&handshake)),
            is_dropped: AtomicBool::new(false),
        });

        let (pending_request_sender, pending_request_receiver) =
            tokio::sync::mpsc::channel(MAX_PENDING_COMMANDS);

        //
        // Spawn the child task for the reader half.
        //
        tokio::spawn(run_reader_task(
            reader,
            state.clone(),
            pending_request_receiver,
        ));

        Ok(Self {
            pending_request_sender,
            writer,
            state,
            local_unhandled_events: Vec::new(),
        })
    }

    /// Polls the list of unhandled events received from the server since the last
    /// call to this method.
    pub fn poll_unhandled_events(&mut self) -> impl Iterator<Item = Event> {
        self.local_unhandled_events
            .append(&mut self.state.unhandled_events.lock());
        self.local_unhandled_events.drain(..)
    }

    /// Returns a reference to the current game state.
    ///
    /// The result of this method is a read guard to the game state. No `.await` point should
    /// exist within the scope of the returned guard.
    ///
    /// # Remarks
    ///
    /// This function returns a read guard which locks the game state for the whole process. Try
    /// to keep the scope of the returned guard as short as possible to avoid blocking other
    /// threads.
    pub fn game_state(&self) -> impl Deref<Target = GameState> {
        self.state.game_state.read()
    }

    /// Returns a mutable reference to the current game state.
    ///
    /// The game state is normally updated automatically based on the responses received
    /// from the server. However, it is still possible to update the game state manually
    /// with this method.
    ///
    /// # Remarks
    ///
    /// This function returns a write guard which locks the game state for the whole process. Try
    /// to keep the scope of the returned guard as short as possible to avoid blocking other
    /// threads.
    pub fn game_state_mut(&self) -> impl DerefMut<Target = GameState> {
        self.state.game_state.write()
    }

    /// Requests the server to advance by one square.
    #[doc(alias = "avancer")]
    pub async fn move_forward(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender
            .send(RequestType::MoveForward)
            .await?;
        self.writer.write_all(b"avance\n").await?;
        Ok(())
    }

    /// Requests the server to turn right.
    #[doc(alias = "droite")]
    pub async fn turn_right(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender
            .send(RequestType::TurnRight)
            .await?;
        self.writer.write_all(b"droite\n").await?;
        Ok(())
    }

    /// Requests the server to turn left.
    #[doc(alias = "gauche")]
    pub async fn turn_left(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender
            .send(RequestType::TurnLeft)
            .await?;
        self.writer.write_all(b"gauche\n").await?;
        Ok(())
    }

    /// Requests the server to send the surroundings of the player.
    #[doc(alias = "voir")]
    pub async fn see(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender.send(RequestType::See).await?;
        self.writer.write_all(b"voir\n").await?;
        Ok(())
    }

    /// Requests the server to send the inventory of the player.
    #[doc(alias = "inventaire")]
    pub async fn refresh_inventory(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender
            .send(RequestType::Inventory)
            .await?;
        self.writer.write_all(b"inventaire\n").await?;
        Ok(())
    }

    /// Requests the server to pick us up an item.
    pub async fn pickup_item(&mut self, item_name: ItemType) -> anyhow::Result<()> {
        self.pending_request_sender
            .send(RequestType::Pickup)
            .await?;
        self.writer.write_all(b"prend ").await?;
        self.writer.write_all(item_name.name().as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        Ok(())
    }

    /// Requests the server to drop an item on the ground.
    pub async fn drop_item(&mut self, item_name: ItemType) -> anyhow::Result<()> {
        self.pending_request_sender.send(RequestType::Drop).await?;
        self.writer.write_all(b"pose ").await?;
        self.writer.write_all(item_name.name().as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        Ok(())
    }

    /// Requests the server to kick the player in front of us.
    pub async fn kick(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender.send(RequestType::Kick).await?;
        self.writer.write_all(b"expulse\n").await?;
        Ok(())
    }

    /// Requests the server to broadcast the provided message to everyone.
    pub async fn broadcast(&mut self, message: &[u8]) -> anyhow::Result<()> {
        debug_assert!(!message.contains(&b'\n'));
        self.pending_request_sender
            .send(RequestType::Broadcast)
            .await?;
        self.writer.write_all(b"broadcast ").await?;
        self.writer.write_all(message).await?;
        self.writer.write_all(b"\n").await?;
        Ok(())
    }

    /// Requests the server to start the leveling up process.
    pub async fn incantation(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender
            .send(RequestType::Incantation)
            .await?;
        self.writer.write_all(b"incantation\n").await?;
        Ok(())
    }

    /// Requests the server to fork
    pub async fn fork(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender.send(RequestType::Fork).await?;
        self.writer.write_all(b"fork\n").await?;
        Ok(())
    }

    /// Requests the server to refresh the number of remaining team slots.
    pub async fn refresh_available_team_slots(&mut self) -> anyhow::Result<()> {
        self.pending_request_sender
            .send(RequestType::AvailableTeamSlots)
            .await?;
        self.writer.write_all(b"connect_nbr\n").await?;
        Ok(())
    }
}

impl Drop for ZappyClient {
    fn drop(&mut self) {
        self.state.is_dropped.store(true, Ordering::Relaxed);
    }
}

/// The result of the handshake operation with the server.
#[derive(Debug, Clone)]
struct Handshake {
    /// The width of the world we are playing in.
    pub width: u32,
    /// The height of the world we are playing in.
    pub height: u32,
    /// The number of connections the server can still accept for the provided team.
    pub available_team_slots: u32,
}

/// Performs the handshake with the server, providing the team name.
///
/// This should be the first function to invoke when starting the client.
async fn perform_handshake(
    stream: &mut BufReader<TcpStream>,
    team_name: &str,
) -> anyhow::Result<Handshake> {
    let mut buffer = Vec::new();

    tokio::time::timeout(Duration::from_secs(5), async move {
        //
        // Read the first line. This should be the welcome message. This is always
        // `BIENVENUE\n`.
        //

        buffer.clear();
        stream.read_until(b'\n', &mut buffer).await?;
        anyhow::ensure!(
            buffer == b"BIENVENUE\n",
            "Invalid handshake message received from the server",
        );

        //
        // Send the name of the team we want to join.
        //

        stream.write_all(team_name.as_bytes()).await?;
        stream.write_all(b"\n").await?;

        //
        // Receive the number of slots still available in our team.
        //

        buffer.clear();
        stream.read_until(b'\n', &mut buffer).await?;
        let available_team_slots: u32 = str::from_utf8(&buffer)?.parse()?;

        //
        // Receive the dimensions of the world.
        //

        buffer.clear();
        stream.read_until(b'\n', &mut buffer).await?;
        let mut coordinates = str::from_utf8(&buffer)?
            .split_ascii_whitespace()
            .map(|x| x.parse::<u32>().context("Failed to parse coordinate"));
        let width: u32 = coordinates
            .next()
            .context("Missing width of the world")
            .flatten()?;
        let height: u32 = coordinates
            .next()
            .context("Missing height of the world")
            .flatten()?;

        //
        // We're done with the handshake!
        //

        anyhow::Result::Ok(Handshake {
            width,
            height,
            available_team_slots,
        })
    })
    .await
    .map_err(|_| anyhow::anyhow!("Handshake timed out"))
    .flatten()
}

/// The task responsible for running the reader half of the stream.
async fn run_reader_task(
    reader: OwnedReadHalf,
    state: Arc<SharedState>,
    mut pending_request_receiver: tokio::sync::mpsc::Receiver<RequestType>,
) {
    let mut buffer = Vec::new();
    let mut reader = BufReader::new(reader);

    while !state.is_dropped.load(Ordering::Relaxed) {
        if let Err(err) = try_run_reader_task_iteration(
            &mut buffer,
            &mut reader,
            &state,
            &mut pending_request_receiver,
        )
        .await
        {
            eprintln!("Error: {err}");
        }
    }
}

/// Reads one message from the reader and processes it.
async fn try_run_reader_task_iteration(
    buffer: &mut Vec<u8>,
    reader: &mut BufReader<OwnedReadHalf>,
    state: &SharedState,
    pending_request_receiver: &mut tokio::sync::mpsc::Receiver<RequestType>,
) -> anyhow::Result<()> {
    buffer.clear();
    reader.read_until(b'\n', buffer).await?;

    let mut buffer = buffer.trim_ascii();

    //
    // If the received message starts with `message`, then we are listening to a broadcasted
    // message.
    //

    if let Some(mut broadcast_payload) = buffer.strip_prefix(b"message") {
        broadcast_payload = broadcast_payload.trim_ascii_start();

        let comma = broadcast_payload
            .iter()
            .position(|&c| c == b',')
            .context("Found no `,` character in broadcast payload")?;

        anyhow::ensure!(
            comma == 1,
            "Invalid broadcast direction: \"{}\"",
            broadcast_payload[0..comma].escape_ascii()
        );

        let direction = match broadcast_payload[0] {
            b'0' => BroadcastDirection::Center,
            b'1' => BroadcastDirection::Right,
            b'2' => BroadcastDirection::FrontRight,
            b'3' => BroadcastDirection::Front,
            b'4' => BroadcastDirection::FrontLeft,
            b'5' => BroadcastDirection::Left,
            b'6' => BroadcastDirection::BackLeft,
            b'7' => BroadcastDirection::Back,
            b'8' => BroadcastDirection::BackRight,
            _ => anyhow::bail!(
                "Invalid broadcast direction: {}",
                broadcast_payload[0].escape_ascii(),
            ),
        };

        let content: Box<[u8]> = Box::from(&broadcast_payload[comma + 1..]);

        state
            .unhandled_events
            .lock()
            .push(Event::BroadcastMessage { direction, content });

        return Ok(());
    }

    //
    // If the message is `mort`, then the server is notifying us that a player died.
    //

    if buffer == b"mort" {
        // TODO: Not sure what to do with this. We don't even know if the player was in our team
        // or not.
        println!("A player died.");
        return Ok(());
    }

    //
    // If the message starts with `displacement`, then we have been moved around.
    //

    if let Some(direction) = buffer.strip_prefix(b"displacement") {
        let mut game_state = state.game_state.write();

        let mut dir = match direction.trim_ascii() {
            b"1" => (1, 0),
            b"3" => (0, 1),
            b"5" => (-1, 0),
            b"7" => (0, -1),
            _ => anyhow::bail!(
                "Invalid direction received for `displacement`: \"{}\"",
                direction.escape_ascii(),
            ),
        };

        dir = game_state.player_direction.rotate_vector(dir);
        game_state.player_position_x += dir.0;
        game_state.player_position_y += dir.1;
    }

    //
    // Otherwise, the message must be a response to some request we made.
    //

    let matched_request = pending_request_receiver.try_recv().with_context(|| {
        format!(
            "No pending request found to match with message: \"{}\"",
            buffer.escape_ascii(),
        )
    })?;

    match matched_request {
        RequestType::MoveForward => {
            anyhow::ensure!(
                buffer == b"ok",
                "Expected `ok` as a response to `avance`, got \"{}\"",
                buffer.escape_ascii()
            );

            {
                let mut game_state = state.game_state.write();
                let (dx, dy) = game_state.player_direction.to_vector();
                game_state.player_position_x += dx;
                game_state.player_position_y += dy;
            }
        }
        RequestType::TurnLeft => {
            anyhow::ensure!(
                buffer == b"ok",
                "Expected `ok` as a response to `gauche`, got \"{}\"",
                buffer.escape_ascii()
            );

            {
                let mut game_state = state.game_state.write();
                game_state.player_direction = game_state.player_direction.rotated_left();
            }
        }
        RequestType::TurnRight => {
            anyhow::ensure!(
                buffer == b"ok",
                "Expected `ok` as a response to `droite`, got \"{}\"",
                buffer.escape_ascii()
            );

            {
                let mut game_state = state.game_state.write();
                game_state.player_direction = game_state.player_direction.rotated_right();
            }
        }
        RequestType::See => {
            anyhow::ensure!(
                buffer.len() >= 3,
                "Invalid response to `voir`: \"{}\"",
                buffer.escape_ascii()
            );

            anyhow::ensure!(
                buffer.starts_with(b"{"),
                "Expected '{{' as the first character of the response to `voir`, got \"{}\"",
                buffer[0].escape_ascii()
            );

            anyhow::ensure!(
                buffer.ends_with(b"}"),
                "Expected '}}' as the last character of the response to `voir`, got \"{}\"",
                buffer[buffer.len() - 1].escape_ascii()
            );

            buffer = &buffer[1..buffer.len() - 1];

            let mut game_state = state.game_state.write();

            let expected_iterator_size =
                game_state.player_level as usize * game_state.player_level as usize;
            let mut actual_iterator_size = 0;

            let mut dy = 0;
            let mut dx = 0;
            let mut amplitude = 1;
            for cell in buffer
                .split(|&c| c == b',')
                .map(|s| str::from_utf8(s)?.parse::<CellContent>())
            {
                *game_state.get_cell_mut(dx, dy) = cell.context("Failed to parse cell content")?;

                dx += 1;
                if dx == amplitude {
                    amplitude += 1;
                    dx = -amplitude + 1;
                    dy += 1;
                }

                actual_iterator_size += 1;
            }

            if actual_iterator_size != expected_iterator_size {
                eprintln!(
                    "warning: Expected {} cells for the current level, got {}",
                    expected_iterator_size, actual_iterator_size,
                );
            }
        }
        RequestType::Inventory => {
            anyhow::ensure!(
                buffer.len() >= 3,
                "Invalid response to `inventaire`: \"{}\"",
                buffer.escape_ascii(),
            );

            anyhow::ensure!(
                buffer.starts_with(b"{"),
                "Expected '{{' as the first character of the response to `inventaire`, got \"{}\"",
                buffer[0].escape_ascii(),
            );

            anyhow::ensure!(
                buffer.ends_with(b"}"),
                "Expected '}}' as the last character of the response to `inventaire`, got \"{}\"",
                buffer[buffer.len() - 1].escape_ascii(),
            );

            buffer = &buffer[1..buffer.len() - 1];

            let mut game_state = state.game_state.write();
            for slot in buffer.split(|&c| c == b',').map(parse_inventory_slot) {
                let (name, count) = slot.context("Can't parse inventory slot")?;

                match name {
                    b"food" => game_state.food_count += count,
                    b"linemate" => game_state.linemate_count += count,
                    b"deraumere" => game_state.deraumere_count += count,
                    b"sibur" => game_state.sibur_count += count,
                    b"mendiane" => game_state.mendiane_count += count,
                    b"phiras" => game_state.phiras_count += count,
                    b"thystame" => game_state.thystame_count += count,
                    _ => anyhow::bail!("Unknown inventory item type: {}", name.escape_ascii()),
                }
            }
        }
        RequestType::Pickup => match buffer {
            b"ok" => {}
            b"ko" => {}
            _ => anyhow::bail!(
                "Invalid response to `prendre`: \"{}\"",
                buffer.escape_ascii(),
            ),
        },
        RequestType::Drop => match buffer {
            b"ok" => {}
            b"ko" => {}
            _ => anyhow::bail!("Invalid response to `pose`: \"{}\"", buffer.escape_ascii(),),
        },
        RequestType::Kick => match buffer {
            b"ok" => {}
            b"ko" => {}
            _ => anyhow::bail!(
                "Invalid response to `expulse`: \"{}\"",
                buffer.escape_ascii(),
            ),
        },
        RequestType::Broadcast => {
            anyhow::ensure!(
                buffer == b"ok",
                "Expected `ok` as a response to `broadcast`, got \"{}\"",
                buffer.escape_ascii()
            );
        }
        RequestType::Incantation => {
            let new_level = buffer.strip_prefix(b"niveau actuel :").with_context(|| {
                format!(
                    "Invalid response to `incantation`: \"{}\"",
                    buffer.escape_ascii()
                )
            })?;

            let new_level: u32 = str::from_utf8(new_level)
                .map_err(anyhow::Error::from)
                .and_then(|x| x.parse().map_err(anyhow::Error::from))
                .context("Failed to parse new player level")?;

            let mut game_state = state.game_state.write();

            anyhow::ensure!(
                new_level == game_state.player_level + 1,
                "Expected new level to be {}, got {}",
                game_state.player_level + 1,
                new_level
            );

            game_state.player_level = new_level;
        }
        RequestType::Fork => {
            anyhow::ensure!(
                buffer == b"ok",
                "Expected `ok` as a response to `fork`, got \"{}\"",
                buffer.escape_ascii()
            );
        }
        RequestType::AvailableTeamSlots => {
            let team_slots: u32 = str::from_utf8(buffer)
                .map_err(anyhow::Error::from)
                .and_then(|x| x.parse().map_err(anyhow::Error::from))
                .context("Failed to parse available team slots")?;
            state.game_state.write().available_team_slots = team_slots;
        }
    }

    Ok(())
}

fn parse_inventory_slot(slot: &[u8]) -> anyhow::Result<(&[u8], u32)> {
    let space = slot
        .iter()
        .position(|&c| c == b' ')
        .context("Invalid inventory slot format")?;
    let name = &slot[0..space];
    let count = str::from_utf8(&slot[space + 1..])?.parse()?;
    Ok((name, count))
}
