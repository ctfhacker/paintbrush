
#![no_std]

use rangeset::{RangeSet, InclusiveRange};
use errchain::prelude::*;
use global_types::PhysAddr;

mod stats;
pub use stats::Stats;

/// Argument passed to the kernel from UEFI
#[derive(Debug, Copy, Clone)]
#[repr(C, align(4096))]
pub struct CoreArg {
    /// ID for this core
    pub core: Option<usize>,

    /// [`RangeSet`] containing the physical memory available to this core
    pub memory: RangeSet,

    /// Physical address of the alive bit to set in the bootloader
    pub alive_address: Option<*mut bool>,

    /// The [`PhysAddr`] of the page table specific for this core
    pub page_table: PhysAddr,

    /// The [`Stats`] for this core
    pub stats: Stats
}

impl CoreArg {
    /// Create an empty [`CoreArg`]. Created as `new()` instead of `Default` for
    /// `const`
    pub const fn new() -> Self {
        CoreArg {
            core:          None,
            memory:        RangeSet::new(),
            alive_address: None,
            page_table:    PhysAddr(0),
            stats:         Stats::new()
        }
    }

    /// Reset the core arg back to the original state
    pub fn reset(&mut self) {
        self.core = None;
        self.memory.clear();
    }

    /// Set the core id for this core
    pub fn set_core(&mut self, core: usize) {
        self.core = Some(core);
    }

    /// Set the alive address to write the status of if this core is alive
    pub fn set_alive_address(&mut self, addr: *mut bool) {
        self.alive_address = Some(addr);
    }

    /// Set the beginning of memory for this core
    ///
    /// # Errors
    ///
    /// If `memory_size` is zero or if `memory_start + memory_size - 1` overflows
    pub fn insert_memory(&mut self, memory_start: u64, memory_size: u64) -> Result<()> {
        // Calculate the end of memory
        let memory_end = add!(memory_start, sub!(memory_size, 1));

        // Insert the entire physical memory chunk into the `RangeSet`
        self.memory.insert(InclusiveRange::new(memory_start, memory_end));

        Ok(())
    }
}
