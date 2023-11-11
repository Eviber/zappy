use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::EXECUTOR;

/// Returns a future that completes when the `buf` has been completely written to the
/// provided file descriptor.
pub fn write_all(fd: ft::Fd, buf: &[u8]) -> WriteAll {
    WriteAll { fd, buf }
}

/// See [`write_all`].
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
