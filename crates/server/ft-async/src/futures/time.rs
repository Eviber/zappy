use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use crate::EXECUTOR;

/// A future that completes at a particular instant in time.
pub fn sleep(alarm: ft::Instant) -> Sleep {
    Sleep { alarm: Some(alarm) }
}

/// See [`sleep`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Sleep {
    /// The time at which the sleep should end.
    alarm: Option<ft::Instant>,
}

impl Future for Sleep {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        if let Some(alarm) = self.alarm.take() {
            EXECUTOR.wake_me_up_on_alarm(alarm, cx.waker().clone());
            Poll::Pending
        } else {
            Poll::Ready(())
        }
    }
}
