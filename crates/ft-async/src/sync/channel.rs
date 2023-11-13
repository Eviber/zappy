//! A channel for sending values between asynchronous tasks.

use core::future::Future;
use core::pin::Pin;
use core::ptr::NonNull;
use core::task::{Context, Poll, Waker};

use alloc::sync::{Arc, Weak};

/// The mutex type used by the executor.
type Mutex<T> = ft::Mutex<T, ft::sync::mutex::NoBlockLock>;

/// Creates a new channel for sending values of type `T`.
///
/// At most one value can be sent through the channel at a time.
#[allow(clippy::must_use_candidate)]
pub fn make<T>() -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(ft::Mutex::new(Shared {
        value: None,
        receiver: None,
        senders: WakerList::new(),
    }));
    let shared_weak = Arc::downgrade(&shared);
    (Sender(shared), Receiver(shared_weak))
}

/// The shared state of a channel.
struct Shared<T> {
    /// The values that have been sent through the channel.
    value: Option<T>,
    /// If the receiver is waiting for a value, this is its waker.
    receiver: Option<Waker>,
    /// If a sender is waiting for the receiver to receive a value, this is its waker.
    senders: WakerList,
}

/// The sending half of a channel.
pub struct Sender<T>(Arc<Mutex<Shared<T>>>);

impl<T> Sender<T> {
    /// Sends a value through the channel.
    ///
    /// If the channel is closed, this will return an error.
    #[inline]
    pub fn send(&self, value: T) -> Send<T> {
        Send {
            shared: &self.0,
            value: Some(value),
            waker_node: None,
            _marker: core::marker::PhantomPinned,
        }
    }
}

impl<T> Clone for Sender<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

/// See [`Sender::send`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Send<'a, T> {
    /// A reference to the shared state.
    shared: &'a Arc<Mutex<Shared<T>>>,

    /// The value to be sent.
    value: Option<T>,

    /// The node in the list of senders.
    ///
    /// If this is `None`, a waker has not been registered yet.
    waker_node: Option<WakerNode>,

    /// This future is not Unpin because `waker_node` needs to remain stable
    /// in memory.
    _marker: core::marker::PhantomPinned,
}

impl<'a, T> Future for Send<'a, T> {
    type Output = Result<(), T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Self::Output> {
        // SAFETY:
        //  We are not moving `self` anywhere, and specifically, we're not
        //  moving `waker_node` anywhere.
        let this = unsafe { self.get_unchecked_mut() };

        // If the channel is already closed, return an error.
        if Arc::weak_count(this.shared) == 0 {
            return Poll::Ready(Err(this
                .value
                .take()
                .expect("future polled after completion")));
        };

        let mut lock = this.shared.lock();

        // If the slot is empty, send the value.
        if lock.value.is_none() {
            lock.value = Some(this.value.take().expect("future polled after completion"));
            return Poll::Ready(Ok(()));
        }

        // Otherwise, register a waker.
        let node_ptr = NonNull::from(this.waker_node.insert(WakerNode {
            waker: cx.waker().clone(),
            next: None,
        }));
        unsafe { lock.senders.push_back(node_ptr) };
        Poll::Pending
    }
}

impl<'a, T> Drop for Send<'a, T> {
    fn drop(&mut self) {
        // Remove the waker from the list of senders.
        let waker_node_ptr = match self.waker_node.as_ref() {
            None => return,
            Some(node) => NonNull::from(node),
        };

        self.shared.lock().senders.remove(waker_node_ptr);
    }
}

/// The receiving half of a channel.
pub struct Receiver<T>(Weak<Mutex<Shared<T>>>);

impl<T> Receiver<T> {
    /// Receives a value from the channel.
    ///
    /// If the channel is closed, this will return `None`.
    #[inline]
    pub fn recv(&self) -> Recv<T> {
        Recv(&self.0)
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Wake the senders waiting to send their value.
        let Some(shared) = self.0.upgrade() else {
            return;
        };

        shared.lock().senders.wake_all();
    }
}

/// See [`Receiver::recv`].
#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct Recv<'a, T>(&'a Weak<Mutex<Shared<T>>>);

impl<'a, T> Future for Recv<'a, T> {
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // If the channel is closed, return `None`.
        let shared = match self.0.upgrade() {
            None => return Poll::Ready(None),
            Some(shared) => shared,
        };

        let mut lock = shared.lock();

        // If a value is already available, return it.
        if let Some(value) = lock.value.take() {
            // If a sender is waiting for the slot to be freed, wake it up.
            if let Some(sender) = lock.senders.pop_front() {
                sender.wake();
            }

            return Poll::Ready(Some(value));
        }

        // Otherwise, register the waker.
        lock.receiver = Some(cx.waker().clone());
        Poll::Pending
    }
}

impl<'a, T> Drop for Recv<'a, T> {
    fn drop(&mut self) {
        let Some(shared) = self.0.upgrade() else {
            return;
        };

        // Remove the waker.
        shared.lock().receiver = None;
    }
}

/// A linked-list of waiters for a channel.
struct WakerList {
    head: Option<NonNull<WakerNode>>,
}

impl WakerList {
    /// Creates a new empty [`WakerList`].
    pub const fn new() -> Self {
        Self { head: None }
    }

    /// Pops the first waker from the list.
    pub fn pop_front(&mut self) -> Option<Waker> {
        match self.head {
            None => None,
            Some(head) => {
                let head = unsafe { head.as_ptr().read() };
                self.head = head.next;
                Some(head.waker)
            }
        }
    }

    /// Pushes a new waker to the list.
    ///
    /// # Safety
    ///
    /// `waker` must remain stable in memory, and must reference a valid [`WakerNode`]
    /// instance.
    pub unsafe fn push_back(&mut self, waker: NonNull<WakerNode>) {
        let mut cur = &mut self.head;

        loop {
            match cur {
                None => {
                    // We reached the end of the linked list.
                    *cur = Some(waker);
                    return;
                }
                Some(node) => {
                    // We haven't reached the end of the linked list yet.
                    cur = unsafe { &mut node.as_mut().next };
                }
            }
        }
    }

    /// Removes a waker node from the list.
    pub fn remove(&mut self, waker: NonNull<WakerNode>) -> Option<Waker> {
        let mut cur = &mut self.head;

        loop {
            match cur {
                None => {
                    // We reached the end of the linked list.
                    return None;
                }
                Some(node) => {
                    // We haven't reached the end of the linked list yet.
                    if *node == waker {
                        let node = unsafe { node.as_ptr().read() };

                        // We found the node.
                        *cur = node.next;
                        return Some(node.waker);
                    }

                    cur = unsafe { &mut node.as_mut().next };
                }
            }
        }
    }

    /// Wake all the waiters in the list, removing them from the list.
    pub fn wake_all(&mut self) {
        let cur = &mut self.head;

        loop {
            // Take the node.
            let Some(node) = cur.take() else {
                // We reached the end of the linked list.
                return;
            };
            let node = unsafe { node.as_ptr().read() };

            // Replace by next node.
            *cur = node.next;

            // Wake the waker.
            node.waker.wake();
        }
    }
}

unsafe impl core::marker::Send for WakerList {}
unsafe impl Sync for WakerList {}

/// A node in the linked list of wakers.
struct WakerNode {
    /// The waker.
    waker: Waker,
    /// The next node in the linked list.
    next: Option<NonNull<WakerNode>>,
}

unsafe impl core::marker::Send for WakerNode {}
unsafe impl Sync for WakerNode {}
