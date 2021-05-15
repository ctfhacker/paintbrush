//! Architecture agnostic Page Table implementations for translating virtual addresses to
//! physical addresses

#![no_std]

use global_types::{PhysAddr, VirtAddr};
use phys_mem::PhysMem;

mod x86;

#[cfg(target_arch="x86_64")]
pub use x86::{PageTable, Entry, EntryBuilder, EntryFlags};

use errchain::Result;

/// Has the ability to translate a [`VirtAddr`] into the [`PhysAddr`]
pub trait CanTranslate {
    fn _translate(&self, virt_addr: VirtAddr, 
        print: Option<&dyn Fn(core::fmt::Arguments)>) -> Result<Translated>;

    #[cfg(feature = "verbose")]
    fn translate(&self, virt_addr: VirtAddr, print: &dyn Fn(core::fmt::Arguments))  
            -> Result<Translated> {
        self._translate(virt_addr, Some(print))
    }

    #[cfg(not(feature = "verbose"))]
    fn translate(&self, virt_addr: VirtAddr, _print: &dyn Fn(core::fmt::Arguments)) 
            -> Result<Translated> {
        self._translate(virt_addr, None)
    }
}

/// Has the ability to map a [`VirtAddr`] into the [`PhysAddr`], allocating pages uses
/// the [`PhysMem`]
pub trait CanMap: CanTranslate {
    /// Map the given [`PageSize`] page at [`PhysAddr`] to the given [`VirtAddr`] with an
    /// optional `print` callback
    fn _map_raw<P: PhysMem>(&self, entry: Entry, virt_addr: VirtAddr, 
            entry_size: PageSize, phys_mem: &mut P, 
            print: Option<&dyn Fn(core::fmt::Arguments)>) -> Result<()>;

    /// Map the given 4 KiB page at [`PhysAddr`] to the given [`VirtAddr`] with an
    /// optional `print` callback
    fn _map_raw_4k<P: PhysMem>(&self, entry: Entry, virt_addr: VirtAddr, phys_mem: &mut P, 
            print: Option<&dyn Fn(core::fmt::Arguments)>) -> Result<()> {
        self._map_raw(entry, virt_addr, PageSize::Size4K, phys_mem, print)
    }

    /// Map the given 2 MiB page at [`PhysAddr`] to the given [`VirtAddr`] with an
    /// optional `print` callback
    fn _map_raw_2m<P: PhysMem>(&self, entry: Entry, virt_addr: VirtAddr, phys_mem: &mut P, 
            print: Option<&dyn Fn(core::fmt::Arguments)>) -> Result<()> {
        self._map_raw(entry, virt_addr, PageSize::Size2M, phys_mem, print)
    }

    /// Map the given 4 KiB page at [`PhysAddr`] to the given [`VirtAddr`] 
    #[cfg(not(feature = "verbose"))]
    fn map_raw_4k<P: PhysMem>(&self, entry: Entry, virt_addr: VirtAddr, phys_mem: &mut P,
            _print: &dyn Fn(core::fmt::Arguments)) -> Result<()> {
        self._map_raw_4k(entry, virt_addr, phys_mem, None)
    }

    /// Map the given 4 KiB page at [`PhysAddr`] to the given [`VirtAddr`] while enabling
    /// print features via the `print` callback
    #[cfg(feature = "verbose")]
    fn map_raw_4k<P: PhysMem>(&self, entry: Entry, virt_addr: VirtAddr, phys_mem: &mut P, 
            print: &dyn Fn(core::fmt::Arguments)) -> Result<()> {
        self._map_raw_4k(entry, virt_addr, phys_mem, Some(print))
    }

    /// Map the given 2 MiB page at [`PhysAddr`] to the given [`VirtAddr`] 
    #[cfg(not(feature = "verbose"))]
    fn map_raw_2m<P: PhysMem>(&self, entry: Entry, virt_addr: VirtAddr, 
            phys_mem: &mut P, _print: &dyn Fn(core::fmt::Arguments)) -> Result<()> {
        self._map_raw_2m(entry, virt_addr, phys_mem, None)
    }

    /// Map the given 2 MiB page at [`PhysAddr`] to the given [`VirtAddr`] while enabling
    /// print features via the `print` callback
    #[cfg(feature = "verbose")]
    fn map_raw_2m<P: PhysMem>(&self, entry: Entry, virt_addr: VirtAddr, 
            phys_mem: &mut P, print: &dyn Fn(core::fmt::Arguments)) -> Result<()> {
        self._map_raw_2m(entry, virt_addr, phys_mem, Some(print))
    }
}

pub trait CanUpdatePerms: CanTranslate {
    fn _update_perms(&mut self, virt_addr: VirtAddr, perms: Permissions,
            print: Option<&dyn Fn(core::fmt::Arguments)>) -> Result<()>;

    #[cfg(feature = "verbose")]
    fn update_perms(&mut self, virt_addr: VirtAddr, perms: Permissions, 
            print: &dyn Fn(core::fmt::Arguments)) -> Result<()> {
        self._update_perms(virt_addr, perms, Some(print))
    }

    #[cfg(not(feature = "verbose"))]
    fn update_perms(&mut self, virt_addr: VirtAddr, perms: Permissions, 
            _print: &dyn Fn(core::fmt::Arguments)) -> Result<()> {
        self._update_perms(virt_addr, perms, None)
    }

    #[cfg(not(feature = "verbose"))]
    fn set_writable_executable(&mut self, virt_addr: VirtAddr) -> Result<()> {
        let perms = Permissions {
            readable:   true,
            writable:   true,
            executable: true,
        };

        self.update_perms(virt_addr, perms, &|_|{})
    }
}

/// The result of a [`translate`](CanTranslate::translate) containing the page size and
/// address
#[derive(Debug, Copy, Clone)]
pub struct Translated {
    /// The physical address of this translated page
    phys_addr: Option<PhysAddr>,

    /// The virtual address of this translated page
    virt_addr: VirtAddr,

    /// The size of the translated page
    size: Option<PageSize>,

    /// Address of each intermediate translation levels. This is the address of the entry
    /// and not the entry itself. Holding the address allows us to cache this specific
    /// address without having to translate an address again to check for changes in the
    /// entry itself (like looking for new dirty bits)
    entries: [Option<PhysAddr>; 4],

    /// [`Permissions`] for this entry
    perms: Permissions,
}

impl Translated {
    /// Create a new [`Translated`] with the given [`PhysAddr`] and [`PageSize`]
    pub fn new(virt_addr: VirtAddr, phys_addr: PhysAddr, size: PageSize, 
            entries: [Option<PhysAddr>; 4], perms: Permissions) -> Self {
        Self { 
            virt_addr, 
            phys_addr: Some(phys_addr), 
            size:      Some(size), 
            entries,
            perms
        }
    }

    pub fn new_not_present(virt_addr: VirtAddr, entries: [Option<PhysAddr>; 4]) -> Self {
        Self { 
            virt_addr, 
            phys_addr: None,
            size:      None, 
            entries,
            perms: Permissions { readable: false, writable: false, executable: false }
        }
    }

    /// Get the [`PhysAddr`] of this translated page
    pub fn phys_addr(&self) -> Option<PhysAddr> {
        self.phys_addr
    }

    /// Get the [`VirtAddr`] of this translated page
    pub fn virt_addr(&self) -> VirtAddr {
        self.virt_addr
    }

    /// Get the [`PageSize`] of this translated page
    pub fn size(&self) -> Option<PageSize> {
        self.size
    }

    /// Get the physical addresses of the intermediate entries for this translation
    pub fn entries(&self) -> [Option<PhysAddr>; 4] {
        self.entries
    }
}

/// The size of a given page 
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PageSize {
    /// A page with 512 gigabytes (512Gib)
    Size512G,

    /// A page with 2 megabytes (2MiB)
    Size2M,

    /// A page with 4 kilobytes (4KiB)
    Size4K,
}

/// The permissions for the translated entry
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Permissions {
    /// The page is readable
    readable:  bool,

    /// The page is writable
    writable:  bool,

    /// The page is executable
    executable: bool
}
