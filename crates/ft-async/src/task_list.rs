use alloc::vec::Vec;
use core::cmp::Ordering::*;

use crate::Task;

/// The ID of a task, exists within a [`TaskList`].
pub type TaskId = usize;

/// A list of taks.
pub struct TaskList<'a> {
    /// The list of tasks.
    tasks: Vec<Option<Task<'a>>>,
    /// The index of a slot that is empty but reserved.
    ///
    /// This is used when a task has to be removed from the list, but its
    /// position has to remain reserved (no new task can be added in its place).
    ///
    /// When `usize::MAX`, no task is reserved.
    reserved: usize,
    /// The index of the first slot that is empty.
    ///
    /// If no slot is empty, this is equal to `tasks.len()`.
    first_hole: usize,
}

impl<'a> TaskList<'a> {
    /// Creates an new empty [`TaskList`].
    pub const fn new() -> Self {
        Self {
            tasks: Vec::new(),
            reserved: usize::MAX,
            first_hole: 0,
        }
    }

    /// Attempts to find a hole in the list, starting from the current first hole,
    /// and moving right.
    fn update_hole_rightward(&mut self) {
        while self.first_hole != self.reserved
            && self.tasks.get(self.first_hole).is_some_and(Option::is_some)
        {
            self.first_hole += 1;
        }
    }

    /// Adds a task to the list and returns its ID.
    pub fn insert(&mut self, task: Task<'a>) -> TaskId {
        if let Some(slot) = self.tasks.get_mut(self.first_hole) {
            debug_assert!(slot.is_none());
            *slot = Some(task);
            let id = self.first_hole;
            self.update_hole_rightward();
            id
        } else {
            let id = self.tasks.len();
            self.tasks.push(Some(task));
            self.first_hole = self.tasks.len();
            id
        }
    }

    /// Removes a task from the list, but reserving its slot. This prevents
    /// any new task from being added in its place.
    pub fn remove_reserve(&mut self, id: TaskId) -> Option<Task<'a>> {
        match self.tasks.get_mut(id) {
            Some(slot) => {
                self.reserved = id;
                slot.take()
            }
            None => None,
        }
    }

    /// Puts a task back into the list, in the slot that was reserved for it.
    ///
    /// # Panics
    ///
    /// This function panics if not slot was reserved.
    pub fn restore_reserved(&mut self, task: Task<'a>) {
        assert!(self.reserved != usize::MAX);
        let slot = unsafe { self.tasks.get_unchecked_mut(self.reserved) };
        debug_assert!(slot.is_none());
        *slot = Some(task);
        self.reserved = usize::MAX;
    }

    /// Marks the reserved task as no longer reserved.
    pub fn give_up_reserved(&mut self) {
        debug_assert!(self.reserved != usize::MAX);
        self.reserved = usize::MAX;

        // If the reserved slot is the first hole, we need to update the first hole.
        match self.reserved.cmp(&self.first_hole) {
            Less => self.first_hole = self.reserved,
            Equal => self.update_hole_rightward(),
            Greater => (),
        }
    }
}
