/// Server abstraction module.
pub mod commands;

mod errors;

pub use commands::Command;
use errors::InvalidMsg::MissingValue;
pub use errors::Result;

use crate::args::Args;
use clap::Parser;
use io::{Read, Write};
use std::{io, net::TcpStream};

use self::commands::Msg;

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
    /// Read buffer.
    buf: String,
}

impl Server {
    /// Creates a new server instance and connects to it.
    pub fn new() -> Result<Self> {
        let args = Args::parse();
        let stream = TcpStream::connect((args.host.as_str(), args.port))?;
        let mut self_ = Self {
            stream,
            width: 0,
            height: 0,
            buf: String::new(),
        };

        let _received = self_.get_line()?;

        self_.stream.write_fmt(format_args!("{}\n", args.name))?;

        let slots: usize = self_.get_line()?.parse()?;
        let line = self_.get_line()?;
        let mut dimensions = line.split_whitespace();
        self_.width = dimensions.next().ok_or(MissingValue)?.parse()?;
        self_.height = dimensions.next().ok_or(MissingValue)?.parse()?;
        println!(
            "slots: {}, width: {}, height: {}",
            slots, self_.width, self_.height
        );

        Ok(self_)
    }

    /// Sends a command to the server.
    pub fn send_command(&mut self, command: Command) -> Result<()> {
        print!("> {}...", command);
        std::io::stdout().flush()?;
        self.stream.write_fmt(format_args!("{}\n", command))?;
        Ok(())
    }

    /// Reads a message from the server.
    pub fn receive(&mut self) -> Result<Msg> {
        let received = self.get_line()?.parse()?;
        println!("in: {}", received);
        Ok(received)
    }

    /// Returns a line read from the server.
    fn get_line(&mut self) -> Result<String> {
        let mut buf = [0; 1024];

        let newline = loop {
            if let Some(newline) = self.buf.find('\n') {
                break newline;
            }
            let len = self.stream.read(&mut buf)?;
            self.buf.push_str(&String::from_utf8_lossy(&buf[..len]));
        };
        let line = self.buf.drain(..newline).collect();
        self.buf.drain(..1);
        Ok(line)
    }
}
