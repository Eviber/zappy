//! A simple logging facade.

#![no_std]
#![deny(clippy::unwrap_used, unsafe_op_in_unsafe_fn)]
#![warn(missing_docs, clippy::must_use_candidate)]

use core::fmt::Arguments;
use core::sync::atomic::AtomicU8;
use core::sync::atomic::Ordering::Relaxed;

use Verbosity::*;

/// A logging verbosity level.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Verbosity {
    /// The message is simply tracing some event in the program. This is generally not
    /// useful for the end user.
    Trace = 1,
    /// The message provides some insight about the current state of the program. This is
    /// generally useful to display, but not necessary.
    Info = 2,
    /// A debug message.
    ///
    /// Messages with this verbosity level should generally not stay in the code for
    /// long.
    Debug = 4,
    /// A warning message. Messages with this verbosity level should generally be displayed
    /// to the user, but they are not critical.
    Warning = 8,
    /// An error message. Messages with this verbosity level should always be displayed to
    /// the user.
    Error = 16,
}

/// A logging verbosity filter.
pub struct VerbosityFilter(AtomicU8);

impl VerbosityFilter {
    /// Creates a new [`VerbosityFilter`] that allows all messages.
    const fn new() -> Self {
        Self(AtomicU8::new(0xFF))
    }

    /// Inserts a new verbosity level into the filter.
    #[inline]
    pub fn insert(&self, level: Verbosity) {
        self.0.fetch_or(level as u8, Relaxed);
    }

    /// Removes a verbosity level from the filter.
    #[inline]
    pub fn remove(&self, level: Verbosity) {
        self.0.fetch_and(!(level as u8), Relaxed);
    }

    /// Returns whether the filter contains the provided verbosity level.
    pub fn contains(&self, level: Verbosity) -> bool {
        self.0.load(Relaxed) & (level as u8) != 0
    }
}

/// A message to log to the console.
pub struct Message<'a> {
    /// The verbosity level of the message to log.
    pub verbosity: Verbosity,
    /// The message to write.
    pub message: Arguments<'a>,
}

/// Creates a new [`Message`] at the provided verbosity level.
///
/// The message is created with provenance information for the location of the
/// macro invocation.
#[macro_export]
macro_rules! message {
    ($v:expr, $($args:tt)*) => {
        $crate::Message {
            verbosity: $v,
            message: ::core::format_args!($($args)*),
        }
    };
}

/// The verbosity level filter for all messages.
pub static VERBOSITY: VerbosityFilter = VerbosityFilter::new();

/// Logs the provided message.
#[inline]
pub fn log(message: &Message) {
    if VERBOSITY.contains(message.verbosity) {
        log_unchecked(message);
    }
}

/// Logs the provided message without checking whether the global verbosity level
/// allows it.
fn log_unchecked(message: &Message) {
    let Message { message, verbosity } = message;

    let (prefix, suffix) = match verbosity {
        Trace => ("   \x1B[1;2mtrace\x1B[0m\x1B[2m  ", "\x1B[0m"),
        Info => ("    \x1B[1;92minfo\x1B[0m  ", ""),
        Debug => ("   \x1B[1;95mdebug\x1B[0m  ", ""),
        Warning => (" \x1B[1;93mwarning\x1B[0m\x1B[93m  ", "\x1B[0m"),
        Error => ("   \x1B[1;31merror\x1B[0m\x1B[91m  ", "\x1B[0m"),
    };

    ft::printf!("{prefix}{message}{suffix}\n");
}

/// Logs a message with the [`Trace`] verbosity level.
#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {
        $crate::log(&$crate::message!($crate::Verbosity::Trace, $($args)*));
    };
}

/// Logs a message with the [`Info`] verbosity level.
#[macro_export]
macro_rules! info {
    ($($args:tt)*) => {
        $crate::log(&$crate::message!($crate::Verbosity::Info, $($args)*));
    };
}

/// Logs a message with the [`Debug`] verbosity level.
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {
        $crate::log(&$crate::message!($crate::Verbosity::Debug, $($args)*));
    };
}

/// Logs a message with the [`Warning`] verbosity level.
#[macro_export]
macro_rules! warning {
    ($($args:tt)*) => {
        $crate::log(&$crate::message!($crate::Verbosity::Warning, $($args)*));
    };
}

/// Logs a message with the [`Error`] verbosity level.
#[macro_export]
macro_rules! error {
    ($($args:tt)*) => {
        $crate::log(&$crate::message!($crate::Verbosity::Error, $($args)*));
    };
}
