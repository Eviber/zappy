use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::EXECUTOR;

/// Creates a [`Future`] that completes when the provided file descriptor is ready to
/// be read.
///
/// When this future completes, the file descriptor is guaranteed not to block when
/// reading from it.
pub fn ready_for_reading(fd: ft::Fd) -> ReadyForReading {
    ReadyForReading {
        fd,
        waker_registered: false,
    }
}

/// Creates a [`Future`] that completes when the provided file descriptor is ready to
/// be written.
///
/// When this future completes, the file descriptor is guaranteed not to block when
/// writing to it.
pub fn ready_for_writing(fd: ft::Fd) -> ReadyForWriting {
    ReadyForWriting {
        fd,
        waker_registered: false,
    }
}

/// Waits until a file descriptor is ready to be read.
pub struct ReadyForReading {
    fd: ft::Fd,
    waker_registered: bool,
}

impl Future for ReadyForReading {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if self.waker_registered {
            Poll::Ready(())
        } else {
            self.waker_registered = true;
            EXECUTOR.wake_me_up_on_read(self.fd, cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Waits until a file descriptor is ready to be written.
pub struct ReadyForWriting {
    fd: ft::Fd,
    waker_registered: bool,
}

impl Future for ReadyForWriting {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if self.waker_registered {
            Poll::Ready(())
        } else {
            self.waker_registered = true;
            EXECUTOR.wake_me_up_on_write(self.fd, cx.waker().clone());
            Poll::Pending
        }
    }
}
