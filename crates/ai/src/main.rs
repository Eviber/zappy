//! A simple Zappy artificial intelligence implementation.

use {anyhow::Context, clap::Parser, tokio::net::TcpStream};

mod api;

/// A simple Artificial Intelligence for Zappy
#[derive(Debug, Clone, Parser)]
#[clap(disable_help_flag = true)]
struct Args {
    /// The hostname of the Zappy server to connect to.
    #[clap(short = 'h', default_value = "localhost")]
    hostname: String,
    /// The port number of the Zappy server to connect to.
    #[clap(short = 'p')]
    port: u16,
    /// Name of the team the AI is playing for
    #[clap(short = 'n')]
    team: String,
    /// Print help
    #[clap(short = '?', long = "help", action = clap::ArgAction::HelpLong)]
    help: (),
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> anyhow::Result<()> {
    //
    // Parse the command-line arguments.
    // Note that `Args::parse` will exit the program if `--help` is provided, or if an error
    // occurs during parsing.
    //

    let args = Args::parse();

    //
    // Open a TCP connection to the Zappy server.
    //

    let stream = TcpStream::connect((args.hostname.as_str(), args.port))
        .await
        .context("Failed to connect to the server")?;

    //
    // Initiate the handshake and create the client instance.
    //

    let mut client = api::ZappyClient::new(stream, &args.team)
        .await
        .context("Failed to create Zappy client")?;

    //
    // Start the main loop.
    //

    // TODO: Implement the actual logic of the AI here.

    Ok(())
}
