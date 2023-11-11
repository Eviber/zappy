#![no_std]
#![feature(const_binary_heap_constructor)]

extern crate alloc;

mod executor;
pub use executor::{Executor, EXECUTOR};
