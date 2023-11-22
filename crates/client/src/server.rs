/// Commands module
pub mod commands;
/// Server abstraction module

/// Errors module
mod errors;

pub use commands::Command;
use errors::InvalidResponse::MissingValue;
pub use errors::Result;

use crate::args::Args;
use clap::Parser;
use io::{Read, Write};
use std::{io, net::TcpStream};

use self::commands::Response;

/// Abstraction over the server.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Server {
    /// The stream to the server.
    stream: TcpStream,
    /// The width of the map.
    width: usize,
    /// The height of the map.
    height: usize,
}

impl Server {
    /// Creates a new server instance and connects to it.
    pub fn new() -> Result<Self> {
        let args = Args::parse();
        let mut stream = TcpStream::connect((args.host.as_str(), args.port))?;

        let _received = read_from_stream(&mut stream)?; // "BIENVENUE\n"

        stream.write_fmt(format_args!("{}\n", args.name))?;

        let received = read_from_stream(&mut stream)?;
        let mut info = received.split_whitespace().map(|s| s.parse());
        let slots = info.next().ok_or(MissingValue)??;
        let width: usize = info.next().ok_or(MissingValue)??;
        let height: usize = info.next().ok_or(MissingValue)??;
        println!("slots: {}, width: {}, height: {}", slots, width, height);

        Ok(Self {
            stream,
            width,
            height,
        })
    }

    /// Sends a command to the server.
    pub fn send_command(&mut self, command: Command) -> Result<Response> {
        print!("> {}...", command);
        std::io::stdout().flush()?;
        self.stream.write_fmt(format_args!("{}\n", command))?;
        println!(" sent");
        // self.receive()
        Ok(Response::Ok)
    }

    /// Reads a message from the server.
    pub fn receive(&mut self) -> Result<Response> {
        let received = read_from_stream(&mut self.stream)?.trim().parse()?;
        println!("in: {}", received);
        Ok(received)
    }
}

/// Reads a message from a stream.
/// Returns when the message ends with a newline.
fn read_from_stream(stream: &mut TcpStream) -> io::Result<String> {
    let mut res = String::new();
    while !res.ends_with('\n') {
        let mut buf = [0; 1024];
        let len = stream.read(&mut buf)?;
        res.push_str(&String::from_utf8_lossy(&buf[..len]));
    }
    Ok(res)
}
