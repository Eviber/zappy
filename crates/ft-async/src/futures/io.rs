use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use ft::collections::ReadBuffer;

use crate::EXECUTOR;

/// Returns a future that completes when the `buf` has been completely written to the
/// provided file descriptor.
pub fn write_all(fd: ft::Fd, buf: &[u8]) -> WriteAll {
    WriteAll { fd, buf }
}

/// Returns a future that completes when a complete line (delimited by `\n`) has been read
/// from the provided file descriptor.
///
/// # Remarks
///
/// If the end of file is reached before the end of a line, the future will complete with
/// an error (`ft::Errno::CONNECTION_RESET`).
///
/// # Returns
///
/// An error, or the line without the final delimiter.
pub fn read_line(fd: ft::Fd, buf: &mut ReadBuffer) -> ReadLine {
    ReadLine { fd, buf }
}

/// See [`write_all`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteAll<'a> {
    fd: ft::Fd,
    buf: &'a [u8],
}

impl<'a> Future for WriteAll<'a> {
    type Output = ft::Result<()>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let n = match self.fd.write(self.buf) {
            Ok(n) => n,
            Err(err) => return Poll::Ready(Err(err)),
        };

        self.buf = unsafe { self.buf.get_unchecked(n..) };

        if self.buf.is_empty() {
            Poll::Ready(Ok(()))
        } else {
            EXECUTOR.wake_me_up_on_write(self.fd, cx.waker().clone());
            Poll::Pending
        }
    }
}

/// See [`read_line`].
pub struct ReadLine<'a> {
    fd: ft::Fd,
    buf: &'a mut ReadBuffer,
}

impl<'a> ReadLine<'a> {
    /// Checks whether the buffer contains a complete line.
    ///
    /// If so, this function takes care of consuming the line
    /// to ensure that it won't be returned again.
    pub fn check_line(&mut self) -> Option<&'a [u8]> {
        let pending = self.buf.pending();

        // Try to find the index of the delimiter.
        let Some(mut index) = pending.iter().position(|&byte| byte == b'\n') else {
            return None;
        };

        // Convert the index into the index of the delimiter *within* the pending
        // buffer; not just within the added bytes.
        index += pending
            .len()
            .wrapping_neg()
            .wrapping_add(self.buf.pending().len());

        // Consume and return the line.

        // SAFETY:
        //  `index + 1` is at most `added.len()` which ensures that we won't overflow
        //  the size of the pending block in the buffer.
        unsafe {
            let consumed = self.buf.pending().as_ptr();
            self.buf.consume_unchecked(index + 1);
            Some(core::slice::from_raw_parts(consumed, index))
        }
    }
}

impl<'a> Future for ReadLine<'a> {
    type Output = ft::Result<&'a [u8]>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        let fd = self.fd;

        // Check if the buffer doesn't already contain a line.
        if let Some(line) = self.check_line() {
            return Poll::Ready(Ok(line));
        }

        // Make sure that the buffer has enough space to read at least 64 bytes.
        match self.buf.reserve(64) {
            Ok(()) => (),
            Err(err) => return Poll::Ready(Err(err.into())),
        }

        // Try to read from the file descriptor.
        match self.buf.fill_with_fd(fd) {
            Ok([]) => return Poll::Ready(Err(ft::Errno::CONNECTION_RESET)),
            Ok(_) => (),
            Err(err) => return Poll::Ready(Err(err)),
        };

        match self.check_line() {
            Some(line) => Poll::Ready(Ok(line)),
            None => {
                EXECUTOR.wake_me_up_on_read(fd, cx.waker().clone());
                Poll::Pending
            }
        }
    }
}
