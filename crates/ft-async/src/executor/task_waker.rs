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

/// Contains the state required to perform a [`ft::select`] system call.
struct Select {
    /// The list of tasks that are waiting for reads to become non-blocking.
    read: Vec<BlockedByIo>,
    /// The list of tasks that are waiting for writes to become non-blocking.
    write: Vec<BlockedByIo>,

    //
    // Those two fields are only used to avoid allocating a whole set on the
    // stack every time we call `select`.
    //
    /// The list of file descriptors that we are waiting for reads to become
    /// non-blocking.
    read_fdset: ft::fd::FdSet,
    /// The list of file descriptors that we are waiting for writes to become
    /// non-blocking.
    write_fdset: ft::fd::FdSet,
}

impl Select {
    /// Creates a new [`Select`] instance.
    pub const fn new() -> Self {
        Self {
            read: Vec::new(),
            write: Vec::new(),
            read_fdset: ft::fd::FdSet::new(),
            write_fdset: ft::fd::FdSet::new(),
        }
    }

    /// Sets the file descriptors that we are waiting for, and returns the
    /// highest file descriptor.
    fn setup_fdset(list: &[BlockedByIo], set: &mut ft::fd::FdSet) -> ft::Fd {
        let mut max = ft::Fd::from_raw(-1);

        set.clear();
        for task in list {
            set.insert(task.fd);

            if task.fd > max {
                max = task.fd;
            }
        }

        max
    }

    /// Wakes up all the tasks that are part of the provided set, removing them from
    /// the list of waiting tasks.
    fn wake_up_tasks_for(list: &mut Vec<BlockedByIo>, set: &ft::fd::FdSet) {
        let mut i = 0;
        while let Some(task) = list.get(i) {
            if set.contains(task.fd) {
                list.swap_remove(i).waker.wake();
            } else {
                i += 1;
            }
        }
    }

    /// Registers a task to be woken up when the provided file descriptor becomes
    /// non-blocking for reads.
    #[inline]
    pub fn register_read(&mut self, fd: ft::Fd, waker: Waker) {
        self.read.push(BlockedByIo { fd, waker });
    }

    /// Registers a task to be woken up when the provided file descriptor becomes
    /// non-blocking for writes.
    #[inline]
    pub fn register_write(&mut self, fd: ft::Fd, waker: Waker) {
        self.write.push(BlockedByIo { fd, waker });
    }

    /// Returns whether there are currently any tasks waiting for I/O.
    #[inline]
    pub fn anybody_waiting(&self) -> bool {
        !self.read.is_empty() || !self.write.is_empty()
    }

    /// Performs the [`ft::select`] system call, waking up tasks that are
    /// waiting for I/O.
    ///
    /// Note: this function will block if no tasks are waiting for I/O.
    pub fn select(&mut self, timeout: Option<Duration>) -> ft::Result<()> {
        let mut maxfd = ft::Fd::from_raw(-1);
        maxfd = maxfd.max(Self::setup_fdset(&self.read, &mut self.read_fdset));
        maxfd = maxfd.max(Self::setup_fdset(&self.write, &mut self.write_fdset));

        ft::fd::select(
            maxfd,
            Some(&mut self.read_fdset),
            Some(&mut self.write_fdset),
            None,
            timeout,
        )?;

        Self::wake_up_tasks_for(&mut self.read, &self.read_fdset);
        Self::wake_up_tasks_for(&mut self.write, &self.write_fdset);

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
