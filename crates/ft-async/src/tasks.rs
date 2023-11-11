use alloc::collections::VecDeque;

use crate::task_list::{TaskId, TaskList};
use crate::Task;

/// The part of the [`Executor`] that is responsible for storing up tasks and keeping
/// track of which ones are ready to be polled.
pub struct Tasks<'a> {
    /// The list of all tasks managed by the executor.
    tasks: TaskList<'a>,
    /// The list of tasks that are ready to be polled.
    ready: VecDeque<TaskId>,
}

impl<'a> Tasks<'a> {
    /// Creates a new empty [`Tasks`].
    pub const fn new() -> Self {
        Self {
            tasks: TaskList::new(),
            ready: VecDeque::new(),
        }
    }

    /// Returns a task that is ready to be polled, if any.
    pub fn take_ready(&mut self) -> Option<(usize, Task<'a>)> {
        while let Some(id) = self.ready.pop_front() {
            if let Some(task) = self.tasks.remove_reserve(id) {
                return Some((id, task));
            }
        }

        None
    }

    /// Marks the task taken with [`take_ready`](Self::take_ready) as pending.
    #[inline]
    pub fn now_pending(&mut self, task: Task<'a>) {
        self.tasks.restore_reserved(task);
    }

    /// Marks the task taken with [`take_ready`](Self::take_ready) as finished, removing
    /// it from the list.
    #[inline]
    pub fn now_ready(&mut self) {
        self.tasks.give_up_reserved();
    }

    /// Adds a task to the list and returns its ID.
    ///
    /// New tasks are always scheduled to be polled.
    pub fn insert(&mut self, task: Task<'a>) -> TaskId {
        let id = self.tasks.insert(task);
        self.ready.push_back(id);
        id
    }

    /// Sets a task as ready to be polled.
    #[inline]
    pub fn set_ready(&mut self, id: TaskId) {
        self.ready.push_back(id);
    }
}
