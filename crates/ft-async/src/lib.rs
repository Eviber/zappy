#![no_std]
#![feature(const_binary_heap_constructor)]

extern crate alloc;

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, Waker};

use alloc::boxed::Box;
use task_list::TaskId;
use task_waker::TaskWaker;
use tasks::Tasks;
use waker::waker_from_task_id;

mod task_list;
mod task_waker;
mod tasks;
mod waker;

/// The mutex type used by the executor.
type Mutex<T> = ft::Mutex<T, ft::sync::mutex::NoBlockLock>;

/// A boxed future, supported by the executor.
type Task<'a> = Pin<Box<dyn 'a + Send + Future<Output = ()>>>;

/// The executor keeping track of which tasks is ready to be polled.
pub struct Executor<'a> {
    //// The list of tasks managed by the executor.
    tasks: Mutex<Tasks<'a>>,
    /// The manager taking care of keeping track of what task waits for what.
    waker: Mutex<TaskWaker>,
}

impl<'a> Executor<'a> {
    /// Creates a new empty [`Executor`].
    const fn new() -> Self {
        Self {
            tasks: Mutex::new(Tasks::new()),
            waker: Mutex::new(TaskWaker::new()),
        }
    }

    /// Spawns a new task onto the executor.
    pub fn spawn<F>(&self, future: F)
    where
        F: Send + Future<Output = ()> + 'a,
    {
        self.tasks.lock().insert(Box::pin(future));
    }

    /// Registers a task to be woken up when the provided alarm expires.
    ///
    /// Note that it is likely that the task will be woken up *some very small
    /// amount of time* after the alarm expires.
    #[inline]
    pub fn wake_me_up_on_alarm(&self, alarm: ft::Instant, waker: Waker) {
        self.waker.lock().register_alarm(alarm, waker);
    }

    /// Registers a task to be woken up when the provided file descriptor is
    /// ready to be read.
    ///
    /// In other words, when reading the file descriptor becomes guaranteed not
    /// to block, the task will be woken up.
    #[inline]
    pub fn wake_me_up_on_read(&self, fd: ft::Fd, waker: Waker) {
        self.waker.lock().register_read(fd, waker);
    }

    /// Registers a task to be woken up when the provided file descriptor is
    /// ready to be written to.
    ///
    /// In other words, when writing to the file descriptor becomes guaranteed
    /// not to block, the task will be woken up.
    #[inline]
    pub fn wake_me_up_on_write(&self, fd: ft::Fd, waker: Waker) {
        self.waker.lock().register_write(fd, waker);
    }

    /// Wakes a task up.
    #[inline]
    fn wake_up(&self, id: TaskId) {
        self.tasks.lock().set_ready(id);
    }

    /// Runs all the tasks  that are currently ready.
    fn run_all_ready_tasks(&self) {
        while let Some((id, mut task)) = self.tasks.lock().take_ready() {
            let waker = waker_from_task_id(id);
            let mut context = Context::from_waker(&waker);
            match task.as_mut().poll(&mut context) {
                Poll::Ready(()) => self.tasks.lock().now_ready(),
                Poll::Pending => self.tasks.lock().now_pending(task),
            }
        }
    }

    /// Runs the executor until an error occurs.
    pub fn run(&self) -> ft::Errno {
        loop {
            match self.waker.lock().block_until_ready() {
                Ok(()) => (),
                Err(err) => break err,
            }

            self.run_all_ready_tasks();
        }
    }
}

/// The global executor.
pub static EXECUTOR: Executor<'static> = Executor::new();
