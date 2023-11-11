//! The main Zappy client.

#![no_std]
#![no_main]
#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

fn main(_args: &[&ft::CharStar], _env: &[&ft::CharStar]) -> u8 {
    ft::printf!("Hello, world!\n");
    0
}

ft::entry_point!(main);
