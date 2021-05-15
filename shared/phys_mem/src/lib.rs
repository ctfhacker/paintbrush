//! Provides a physical memory management trait

#![no_std]

use core::alloc::Layout;
use core::convert::TryInto;

use global_types::PhysAddr;
use errchain::*;

/// Trait used for handling physical memory allocation and management
pub trait PhysMem {
    /// Get a mutable slice to the given [`PhysAddr`] of `size` bytes.
    unsafe fn get_mut_slice(&mut self, phys_addr: PhysAddr, size: usize) 
        -> &mut [u8];

    /// Allocate the given [`PhysAddr`] with the given [`Layout`]
    fn alloc_phys(&mut self, layout: Layout) -> Result<PhysAddr>;

    /// Allocate a `0x1000` aligned physical memory region
    fn alloc_page_aligned(&mut self, size: u64) -> Result<PhysAddr> {
        let layout = Layout::from_size_align(size.try_into().unwrap(), 0x1000)
            .expect("Failed to create the layout for alloc_page_aligned");
        self.alloc_phys(layout)
    }

    /// Allocate a 4 KiB page
    fn alloc_page(&mut self) -> Result<PhysAddr> {
        self.alloc_page_aligned(0x1000)
    }

    /// Allocate a `0x1000` aligned physical memory region
    fn alloc_page_zeroed(&mut self) -> Result<PhysAddr> {
        // Allocate the page
        let page = self.alloc_page()?;

        // Fill the page with zeros
        unsafe { 
            let slice = self.get_mut_slice(page, 0x1000);
            slice.copy_from_slice(&[0; 0x1000]);
        }

        // Return the cleared page
        Ok(page)
    }
}
