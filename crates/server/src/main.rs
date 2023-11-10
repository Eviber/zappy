#![no_std]
#![no_main]

fn main(_args: &[&ft::CharStar], _env: &[&ft::CharStar]) -> u8 {
    ft::printf!("Hello, world!");
    0
}

ft::entry_point!(main);
