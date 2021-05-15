//! Global types used between server and kernel. All addresses are assumed to be `u64`
#![no_std]

use core::convert::TryInto;

// extern crate noodle;
// use noodle::*;

// noodle!(serialize, deserialize,
/// Physical address represented by a `u64`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(C)]
pub struct PhysAddr(pub u64);
// );
//
impl PhysAddr  {
    pub const fn offset(&self, offset: u64) -> PhysAddr {
        PhysAddr(self.0 + offset)
    }

    /// Return if the [`PhysAddr`] is 4 KiB page aligned
    #[inline]
    pub fn is_page_aligned(&self) -> bool {
        self.0 & 0xfff == 0
    }

    /// Read the given `T`
    #[inline]
    pub unsafe fn read_phys<T>(&self) -> T {
        core::ptr::read_unaligned(self.0 as *const T)
    }

    /// Read a `u8` 
    #[inline]
    pub unsafe fn read_u8(&self) -> u8 {
        self.read_phys::<u8>()
    }

    /// Read a `u16`
    #[inline]
    pub unsafe fn _read_u16(&self) -> u16 {
        self.read_phys::<u16>()
    }

    /// Read a `u32` 
    #[inline]
    pub unsafe fn _read_u32(&self) -> u32 {
        self.read_phys::<u32>()
    }

    /// Read a `u64` 
    #[inline]
    pub unsafe fn read_u64(&self) -> u64 {
        self.read_phys::<u64>()
    }

    /// Write the given `T` at the current [`PhysAddr`]
    #[inline]
    pub unsafe fn write<T>(&self, val: T) {
        core::ptr::write_unaligned(self.0 as *mut T, val)
    }

    /// Write the given `u64` at the current [`PhysAddr`]
    #[inline]
    pub unsafe fn write_u64(&self, val: u64) {
        self.write::<u64>(val);
    }

    /// Write the given `u32` at the current [`PhysAddr`]
    #[inline]
    pub unsafe fn write_u32(&self, val: u32) {
        self.write::<u32>(val);
    }

    /// Write the given `u16` at the current [`PhysAddr`]
    #[inline]
    pub unsafe fn write_u16(&self, val: u16) {
        self.write::<u16>(val);
    }

    /// Write the given `u8` at the current [`PhysAddr`]
    #[inline]
    pub unsafe fn write_u8(&self, val: u8) {
        self.write::<u8>(val);
    }
}

impl core::fmt::LowerHex for PhysAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let val = self.0;
        core::fmt::LowerHex::fmt(&val, f)
    }
}

impl core::ops::Deref for PhysAddr {
    type Target = u64;

    #[track_caller]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// noodle!(serialize, deserialize,
/// Virtual address represented by a `u64`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[repr(C)]
pub struct VirtAddr(pub u64);
// );

impl VirtAddr  {
    pub const fn offset(&self, offset: u64) -> VirtAddr {
        VirtAddr(self.0 + offset)
    }

    /// Get the 4 page table indexes that this [`VirtAddr`] corresponds maps with when
    /// translating via a 4-level page table
    pub fn table_indexes(&self) -> [usize; 4] {
        [
            ((self.0 >> 39) & 0x1ff).try_into().unwrap(),
            ((self.0 >> 30) & 0x1ff).try_into().unwrap(),
            ((self.0 >> 21) & 0x1ff).try_into().unwrap(),
            ((self.0 >> 12) & 0x1ff).try_into().unwrap(),
        ]
    }
}

impl core::ops::Deref for VirtAddr {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl core::fmt::LowerHex for VirtAddr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let val = self.0;
        core::fmt::LowerHex::fmt(&val, f)
    }
}

// noodle!(serialize, deserialize,
/// Cr3 represented by a `u64`
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Default, Hash)]
#[repr(C)]
pub struct Cr3(pub u64);
// );

impl core::ops::Deref for Cr3 {
    type Target = u64;

    #[track_caller]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
