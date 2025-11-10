use clap::Parser;

/// Command line arguments for the Zappy GFX Client
#[derive(Parser)]
#[command(name = "Zappy GFX Client")]
pub struct Cli {
    /// Server address in the format host:port
    #[clap(short, long, default_value = "localhost")]
    pub server_address: String,
    #[clap(short, long, default_value = "1234")]
    pub port: u16,
}

pub fn server_address() -> String {
    let cli = Cli::parse();
    format!("{}:{}", cli.server_address, cli.port)
}
