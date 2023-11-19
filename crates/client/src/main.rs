//! The main Zappy client.

#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

use std::{io, net::TcpStream};
use io::{Read, Write};
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

#[derive(Debug)]
struct Server {
    stream: TcpStream,
    width: usize,
    height: usize,
}

impl Server {
    fn new() -> io::Result<Self> {
        let args = Args::parse();
        let mut stream = TcpStream::connect((args.host.as_str(), args.port))?;
        let mut buf = [0; 1024];
        let len = stream.read(&mut buf)?;
        print!("in:\n{}", String::from_utf8_lossy(&buf[..len]));
        let mut buf = args.name.as_bytes().to_vec();
        buf.push(b'\n');
        stream.write(&buf)?;
        println!("out:\n{}", args.name);
        let mut buf = [0; 1024];
        let len = stream.read(&mut buf)?;
        let received = String::from_utf8_lossy(&buf[..len]);
        print!("in:\n{}", received);
        let mut info = received
            .split_whitespace()
            .filter(|s| !s.is_empty())
            .map(|s| s.parse());
        let slots = info.next().expect("server should send remaining slots").expect("server should send valid remaining slots");
        let width: usize = info.next().expect("server should send width").expect("server should send valid width");
        let height: usize = info.next().expect("server should send height").expect("server should send valid height");
        println!("slots: {}, width: {}, height: {}", slots, width, height);
        Ok(Self { stream, width, height })
    }

    fn send_command(&mut self, command: &str) -> io::Result<String> {
        println!("> {}", command);
        let mut buf = command.as_bytes().to_vec();
        buf.push(b'\n');
        self.stream.write(&buf)?;
        let mut buf = [0; 1024];
        let len = self.stream.read(&mut buf)?;
        Ok(String::from_utf8_lossy(&buf[..len]).into_owned())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut server = Server::new()?;
    loop {
        println!("< {}", server.send_command("avance")?);
        println!("< {}", server.send_command("gauche")?);
    }
    // Ok(())
}
