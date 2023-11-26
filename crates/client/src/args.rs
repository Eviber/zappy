/// Module for the command line arguments.
use clap::Parser;

/// A Zappy client.
#[derive(Parser, Debug)]
#[clap(disable_help_flag = true, arg_required_else_help = true)]
pub struct Args {
    /// The name of the team
    #[clap(short)]
    pub name: String,
    /// The port of the server
    #[clap(short)]
    pub port: u16,
    /// The hostname of the server
    #[clap(short, default_value = "localhost")]
    pub host: String,
}
