//! A simple async executor that uses the `select` system call.

#![no_std]
#![feature(const_binary_heap_constructor)]
#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

extern crate alloc;

mod executor;
pub use executor::{Executor, EXECUTOR};

pub mod futures;
