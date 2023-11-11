use core::task::Waker;
use core::time::Duration;

use alloc::collections::BinaryHeap;
use alloc::vec::Vec;

/// An task currently blocked because of an I/O operation.
struct BlockedByIo {
    /// The waker to `.wake()` when the operation becomes non-blocking.
    waker: Waker,
    /// The file descriptor that we are waiting on.
    fd: ft::Fd,
}

/// A list of tasks that are blocked because they are waiting for an event.
struct EventSet {
    /// The list of tasks that are waiting to become non-blocking.
    list: Vec<BlockedByIo>,
    /// An [`ft::fd::FdSet`] to avoid allocating a new one every time we call
    /// [`ft::select`].
    set: ft::fd::FdSet,
}

impl EventSet {
    /// Creates a new [`EventSet`] instance.
    const fn new() -> Self {
        Self {
            list: Vec::new(),
            set: ft::fd::FdSet::new(),
        }
    }

    /// Sets the file descriptors that we are waiting for, and returns the
    /// highest file descriptor.
    fn setup_fdset(&mut self) -> ft::Fd {
        let mut max = ft::Fd::from_raw(-1);

        self.set.clear();
        for task in &self.list {
            self.set.insert(task.fd);

            if task.fd > max {
                max = task.fd;
            }
        }

        max
    }

    /// Wakes up all the tasks, removing them from the list of waiting tasks.
    fn wake_up_tasks(&mut self) {
        let mut i = 0;
        while let Some(task) = self.list.get(i) {
            if self.set.contains(task.fd) {
                self.list.swap_remove(i).waker.wake();
            } else {
                i += 1;
            }
        }
    }

    /// Returns a mutable reference to the [`ft::fd::FdSet`] used to perform
    /// [`ft::select`].
    #[inline]
    fn set_mut(&mut self) -> &mut ft::fd::FdSet {
        &mut self.set
    }

    /// Returns whether there are currently any tasks waiting.
    #[inline]
    fn anybody_waiting(&self) -> bool {
        !self.list.is_empty()
    }

    /// Registers a task to be woken up when the provided file descriptor becomes
    /// non-blocking.
    #[inline]
    fn register(&mut self, fd: ft::Fd, waker: Waker) {
        self.list.push(BlockedByIo { waker, fd });
    }

}

/// Contains the state required to perform a [`ft::select`] system call.
struct Select {
    /// The list of tasks that are waiting for reads to become non-blocking.
    read: EventSet,
    /// The list of tasks that are waiting for writes to become non-blocking.
    write: EventSet,
}

impl Select {
    /// Creates a new [`Select`] instance.
    pub const fn new() -> Self {
        Self {
            read: EventSet::new(),
            write: EventSet::new(),
        }
    }

    /// Registers a task to be woken up when the provided file descriptor becomes
    /// non-blocking for reads.
    #[inline]
    pub fn register_read(&mut self, fd: ft::Fd, waker: Waker) {
        self.read.register(fd, waker);
    }

    /// Registers a task to be woken up when the provided file descriptor becomes
    /// non-blocking for writes.
    #[inline]
    pub fn register_write(&mut self, fd: ft::Fd, waker: Waker) {
        self.write.register(fd, waker);
    }

    /// Returns whether there are currently any tasks waiting for I/O.
    #[inline]
    pub fn anybody_waiting(&self) -> bool {
        !self.read.anybody_waiting() && !self.write.anybody_waiting()
    }

    /// Performs the [`ft::select`] system call, waking up tasks that are
    /// waiting for I/O.
    ///
    /// Note: this function will block if no tasks are waiting for I/O.
    pub fn select(&mut self, timeout: Option<Duration>) -> ft::Result<()> {
        let maxfd = self.read.setup_fdset().max(self.write.setup_fdset());

        ft::fd::select(
            maxfd,
            Some(self.read.set_mut()),
            Some(self.write.set_mut()),
            None,
            timeout,
        )?;

        self.read.wake_up_tasks();
        self.write.wake_up_tasks();

        Ok(())
    }
}

/// A task that is blocked because it is waiting for a certain amount of time.
struct BlockedByTime {
    /// The waker to `.wake()` when the `alarm` expires.
    waker: Waker,
    /// The instant at which the alarm expires.
    alarm: ft::Instant,
}

impl PartialEq for BlockedByTime {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.alarm == other.alarm
    }
}

impl Eq for BlockedByTime {}

impl PartialOrd for BlockedByTime {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BlockedByTime {
    #[inline]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        // We need to reverse the ordering, making earlier alarms bigger than
        // later alarms.
        //
        // This is needed because the `BinaryHeap` is a max-heap, and we want
        // the earliest alarm to be at the top.
        other.alarm.cmp(&self.alarm)
    }
}

/// The list of tasks that are blocked because they are waiting for time to pass.
struct Sleepers {
    /// The list of tasks that are waiting for time to pass.
    list: BinaryHeap<BlockedByTime>,
}

impl Sleepers {
    /// Creates a new [`Sleepers`] instance.
    pub const fn new() -> Self {
        Self {
            list: BinaryHeap::new(),
        }
    }

    /// Registers a task to be woken up when the provided alarm expires.
    #[inline]
    pub fn register(&mut self, alarm: ft::Instant, waker: Waker) {
        self.list.push(BlockedByTime { alarm, waker });
    }

    /// Returns the earliest alarm in the list, if any.
    #[inline]
    pub fn earliest(&self) -> Option<ft::Instant> {
        self.list.peek().map(|sleeper| sleeper.alarm)
    }

    /// Wakes up tasks that are ready to be polled.
    #[allow(clippy::unwrap_used)]
    pub fn wake_up_tasks(&mut self) -> ft::Result<()> {
        let now = ft::Clock::MONOTONIC.get()?;
        while let Some(sleeper) = self.list.peek() {
            if sleeper.alarm <= now {
                self.list.pop().unwrap().waker.wake();
            } else {
                break;
            }
        }
        Ok(())
    }
}

/// Contains the state required to wake up tasks that are waiting for some
/// external event to occur.
pub struct TaskWaker {
    /// Tasks blocked by I/O.
    select: Select,
    /// Tasks blocked by time.
    sleepers: Sleepers,
}

impl TaskWaker {
    /// Creates a new [`TaskWaker`].
    pub const fn new() -> Self {
        Self {
            select: Select::new(),
            sleepers: Sleepers::new(),
        }
    }

    /// Registers a task to be woken up when the provided file descriptor becomes
    /// non-blocking for reads.
    #[inline]
    pub fn register_read(&mut self, fd: ft::Fd, waker: Waker) {
        self.select.register_read(fd, waker);
    }

    /// Registers a task to be woken up when the provided file descriptor becomes
    /// non-blocking for writes.
    #[inline]
    pub fn register_write(&mut self, fd: ft::Fd, waker: Waker) {
        self.select.register_write(fd, waker);
    }

    /// Registers a task to be woken up when the provided alarm expires.
    #[inline]
    pub fn register_alarm(&mut self, alarm: ft::Instant, waker: Waker) {
        self.sleepers.register(alarm, waker);
    }

    /// Blocks the current thread until some of the tasks managed by this [`TaskWaker`]
    /// are ready to be polled.
    pub fn block_until_ready(&mut self) -> ft::Result<()> {
        let timeout = match self.sleepers.earliest() {
            Some(earliest) => {
                let now = ft::Clock::MONOTONIC.get()?;
                Some(earliest.saturating_sub(now))
            }
            None => None,
        };

        if self.select.anybody_waiting() || timeout.is_some() {
            self.select.select(timeout)?;
        }

        self.sleepers.wake_up_tasks()?;
        Ok(())
    }
}
