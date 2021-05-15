//! Provides a physical memory management trait

use core::alloc::Layout;
use core::convert::TryInto;

use global_types::PhysAddr;

use errchain::prelude::*;
use crate::rangeset::RangeSet;

/// Trait used for handling physical memory allocation and management
pub trait PhysMem {
    /// Get a mutable slice to the given [`PhysAddr`] of `size` bytes.
    ///
    /// # Returns
    ///
    /// `slice` if the [`PhysAddr`] can be accessed
    ///
    /// # Errors
    ///
    /// Errors specific to the implementation of the physical memory manager
    unsafe fn get_mut_slice(&mut self, phys_addr: PhysAddr, size: usize) 
        -> Result<&mut [u8]>;

    /// Allocate the given [`PhysAddr`] with the given [`Layout`]
    fn alloc_phys(&mut self, layout: Layout) -> Result<PhysAddr>;

    /// Allocate a `0x1000` aligned physical memory region
    fn alloc_page_aligned(&mut self, size: u64) -> Result<PhysAddr> {
        let layout = Layout::from_size_align(size.try_into().unwrap(), 0x1000).unwrap();
        self.alloc_phys(layout)
    }
}

impl PhysMem for RangeSet {
    unsafe fn get_mut_slice(&mut self, phys_addr: PhysAddr, size: usize) 
        -> Result<&mut [u8]> {
        Ok(core::slice::from_raw_parts_mut(phys_addr.0 as *mut u8, size))
    }

    /// Allocate a physical address with the given [`Layout`]
    fn alloc_phys(&mut self, layout: Layout) -> Result<PhysAddr> {
        Ok(PhysAddr(self.allocate(layout.size() as u64, layout.align() as u64)?))
    }
}
