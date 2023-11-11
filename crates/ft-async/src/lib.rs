#![no_std]
#![feature(const_binary_heap_constructor)]
#![warn(clippy::must_use_candidate)]

extern crate alloc;

mod executor;
pub use executor::{Executor, EXECUTOR};

pub mod futures;
