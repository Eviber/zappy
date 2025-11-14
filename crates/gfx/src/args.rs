use clap::Parser;

/// Command line arguments for the Zappy GFX Client
#[derive(Parser)]
#[command(name = "Zappy GFX Client")]
pub struct Cli {
    /// Server address
    #[clap(short, long, default_value = "localhost")]
    pub server_address: String,
    /// Server port
    #[clap(short, long, default_value = "1234")]
    pub port: u16,
}

/// Get the server address in "address:port" format
pub fn server_address() -> String {
    let cli = Cli::parse();
    format!("{}:{}", cli.server_address, cli.port)
}
