//! Implementation of `Vec` backed by a static const-sized array

use errchain::prelude::*;

/// Various errors that [`StackVec`] functions can result in
#[derive(Copy, Clone, Debug)]
pub enum StackVecError {
    /// [`StackVec`] is currently full
    Full,
}

/// Vec backed by a const-sized array
#[derive(Debug)]
pub struct StackVec<T: Copy, const N: usize> {
    /// Array holding the data
    data: [Option<T>; N],

    /// Index to the next available entry in the vector
    index: usize,
}

#[allow(dead_code)]
impl<T: Copy, const N: usize> StackVec<T, { N }> {
    /// Create a new vector
    ///
    /// Example:
    ///
    /// ```
    /// let mut available_memory = StackVec::<MemoryEntry, 64>::new();
    /// ```
    pub fn new() -> Self {
        Self {
            data: [None; N],
            index: 0
        }
    }

    /// Get the length of the `StackVec`
    pub fn _len(&self) -> usize {
        self.index
    }

    /// Return the current slice of used data in the vector
    pub fn data(&self) -> &[Option<T>] {
        &self.data[..self.index]
    }

    /// Add an entry to the `StackVec`
    pub fn push(&mut self, entry: T) -> Result<()> {
        // Make sure the index is still within bounds of the data array
        ensure!(self.index < self.data.len(), &StackVecError::Full);

        // Add the entry to the stack
        self.data[self.index] = Some(entry);

        // Increment the current index of the array
        self.index += 1;

        Ok(())
    }
}
