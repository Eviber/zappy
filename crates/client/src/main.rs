//! The main Zappy client.

#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

use std::io::{Read, Write};
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(disable_help_flag = true, arg_required_else_help = true)]
struct Args {
    /// The name of the team
    #[clap(short)]
    name: String,
    /// The port of the server
    #[clap(short)]
    port: u16,
    /// The hostname of the server
    #[clap(short, default_value = "localhost")]
    host: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    println!("{:?}", args);
    let mut stream = std::net::TcpStream::connect((args.host.as_str(), args.port))?;
    println!("{:?}", stream);
    let mut buf = [0; 1024];
    let len = stream.read(&mut buf)?;
    print!("in:\n{}", String::from_utf8_lossy(&buf[..len]));
    stream.write_fmt(format_args!("{}\n", args.name))?;
    println!("out:\n{}", args.name);
    buf.fill(0);
    let len = stream.read(&mut buf)?;
    print!("in:\n{}", String::from_utf8_lossy(&buf[..len]));
    let info: Vec<i32> = String::from_utf8_lossy(&buf[..len])
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse())
        .collect::<Result<Vec<i32>, _>>()?;
    println!("{:?}", info);
    Ok(())
}
