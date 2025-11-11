//! Parsing logic of command-line arguments.

use core::fmt;
use core::str::FromStr;

use alloc::vec;
use alloc::vec::Vec;
use ft::CharStar;

/// An error that can occur while parsing the command-line arguments.
pub enum Error<'a> {
    /// An unexpected positional argument was passed.
    UnexpectedPositional(&'a CharStar),
    /// A flag was passed without a value.
    MissingValue(&'a CharStar),
    /// A flag was passed with an invalid value.
    InvalidNumber(&'a CharStar),
    /// An unknown flag was passed.
    UnknownArgument(&'a CharStar),
    /// A team name was invalid.
    InvalidTeamName(&'a [u8]),
}

impl fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::UnexpectedPositional(arg) => write!(f, "unexpected positional argument: `{arg}`"),
            Self::MissingValue(arg) => write!(f, "missing value for argument: `{arg}`"),
            Self::InvalidNumber(arg) => write!(f, "invalid number for argument: `{arg}`"),
            Self::UnknownArgument(arg) => write!(f, "unknown argument: `{arg}`"),
            Self::InvalidTeamName(name) => write!(
                f,
                "invalid team name: `{}`",
                core::str::from_utf8(name).unwrap_or("<invalid UTF-8>")
            ),
        }
    }
}

/// Describes the command-line arguments passed by the user to the server.
#[derive(Debug, Clone)]
pub struct Args<'a> {
    /// The TCP port to connect to.
    ///
    /// Passed using the `-p` flag.
    ///
    /// **Default:** `1234`
    pub port: u16,
    /// The width of the world.
    ///
    /// Passed using the `-x` flag.
    ///
    /// **Default:** `32`
    pub width: usize,
    /// The height of the world.
    ///
    /// Passed using the `-y` flag.
    ///
    /// **Default:** `32`
    pub height: usize,
    /// The name of the teams that will be playing the game.
    ///
    /// Passed using the `-n` flag.
    ///
    /// **Default:** `["Blue", "Red"]`
    pub teams: Vec<&'a str>,
    /// The initial number of players at the begining of the game, per team.
    ///
    /// Passed using the `-c` flag.
    ///
    /// **Default:** `1`
    pub initial_slot_count: u32,
    /// The number of ticks simulated by the server, per second.
    ///
    /// Passed using the `-t` flag.
    ///
    /// **Default:** `10`
    pub tick_frequency: f32,
}

impl<'a> Args<'a> {
    /// Parses the arguments passed to the program.
    pub fn parse_args(args: &[&'a CharStar]) -> Result<Self, Error<'a>> {
        let mut args = args.iter();

        args.next(); // skip the program name

        let mut result = Args::default();

        while let Some(arg) = args.next() {
            if !arg.starts_with(b"-") {
                return Err(Error::UnexpectedPositional(arg));
            }

            match arg.as_bytes_bounded(4) {
                b"-p" => result.port = parse_number(arg, &mut args)?,
                b"-x" => result.width = parse_number(arg, &mut args)?,
                b"-y" => result.height = parse_number(arg, &mut args)?,
                b"-n" => result.teams = parse_team_names(arg, &mut args)?,
                b"-c" => result.initial_slot_count = parse_number(arg, &mut args)?,
                b"-t" => result.tick_frequency = parse_number(arg, &mut args)?,
                _ => return Err(Error::UnknownArgument(arg)),
            }
        }

        Ok(result)
    }
}

impl Default for Args<'_> {
    fn default() -> Self {
        Self {
            port: 1234,
            width: 32,
            height: 32,
            teams: vec!["Blue", "Red"],
            initial_slot_count: 1,
            tick_frequency: 10.0,
        }
    }
}

/// Parses a number from the given arguments.
fn parse_number<'a, 'b, T: FromStr, I>(arg: &'a CharStar, mut args: I) -> Result<T, Error<'a>>
where
    I: Iterator<Item = &'b &'a CharStar>,
    'a: 'b,
{
    let value = args.next().ok_or(Error::MissingValue(arg))?;

    core::str::from_utf8(value.as_bytes_bounded(32))
        .ok()
        .and_then(|x| x.parse().ok())
        .ok_or(Error::InvalidNumber(value))
}

/// Parses the team names from the given arguments.
fn parse_team_names<'a, 'b, I>(arg: &'a CharStar, mut args: I) -> Result<Vec<&'a str>, Error<'a>>
where
    I: Iterator<Item = &'b &'a CharStar>,
    'a: 'b,
{
    let mut values = Vec::new();

    let teams = args.next().ok_or(Error::MissingValue(arg))?;
    for name in teams.split(b',') {
        let name = core::str::from_utf8(name).map_err(|_| Error::InvalidTeamName(name))?;

        if name == "GRAPHIC" {
            return Err(Error::InvalidTeamName(name.as_bytes()));
        }

        values.push(name);
    }

    if values.is_empty() {
        return Err(Error::MissingValue(arg));
    }

    Ok(values)
}
