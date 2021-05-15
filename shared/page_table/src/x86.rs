//! Platform agnostic 4-level page table implementation

use core::ops::{Index, IndexMut};
use core::slice::{Iter, IterMut};

#[cfg(target_arch="x86_64")]
use cpu_x86::{X86Cpu as cpu, CpuTrait};
use global_types::{PhysAddr, VirtAddr};
use phys_mem::PhysMem;
use errchain::{Ok, err, Err, ErrorType, Result, ErrorChain};

use crate::{Translated, CanTranslate, PageSize, CanMap, Permissions, CanUpdatePerms};

/// Errors specific to [`PageTable`] functions
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Attempted to map an physical address that is not page aligned
    CannotMapNonPageAligned,

    /// Attempted to map a virtual address that is already mapped
    VirtAddrAlreadyMapped,
}

impl ErrorType for Error {}

/// A page table containing [`Entry`]
/// 
/// This struct impls [`Index`] and [`IndexMut`] so that it can be indexed directly.
/// 
/// # Example
///
/// ```rust
/// let table = PageTable::from_phys_addr(cr3);
/// let entry = table[10];
/// ```
pub struct PageTable {
    /// The entries in the page table
    pub entries: [Entry; 512]
}

/// An `entry` in a [`PageTable`] containing permission and the address of the next
/// [`Entry`] or the address of the final `page`
#[derive(Debug, Copy, Clone)]
#[repr(transparent)]
pub struct Entry(u64);

impl Entry {
    /// Create a new, empty page table entry
    #[inline]
    pub fn new() -> Self {
        Self(0)
    }

    /// Get the [`EntryFlags`] for this [`Entry`]
    #[inline]
    pub fn flags(self) -> EntryFlags {
        EntryFlags::from(self)
    }

    /// Get the address for this [`Entry`]
    #[inline]
    pub fn address(self) -> PhysAddr {
        PhysAddr(self.0 & 0x000f_ffff_ffff_f000)
    }

    /// Set the `address` field for this [`Entry`] with the given `addr`
    #[inline]
    pub fn set_address(&mut self, addr: u64) {
        // Ensure the address is page aligned
        let addr = addr & !0xfff;

        // Clear the former address
        self.0 &= !(0x000f_ffff_ffff_f000);

        // Set the new address
        self.0 |= addr;
    }

    /// Set the entry as writable
    #[inline]
    pub fn set_writable(&mut self) {
        self.0 |= 1 << 1;
    }

    /// Set the entry as executable
    #[inline]
    pub fn set_executable(&mut self) {
        self.0 &= !(1 << 63);
    }
}

/// Various flags corresponding to a page table entry.
///
/// Reference: [`Page Table Entries`](../../../../../references/Intel_manual_Vol3.pdf#page=134)
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub struct EntryFlags {
    /// Set if this entry is present
    present: bool,
    
    /// Set if this entry is writable
    writable: bool,

    /// Set if this entry can be accessed from Ring 3
    user_permitted: bool,

    /// Set if this entry has `write-through` caching policy or unset if this entry has
    /// `write-back` caching policy
    ///
    /// `write-through`: Writes and reads to and from system memory are cached. Reads
    /// come from cache lines on cache hits; read misses cause cache fills. Speculative
    /// reads are allowed. All writes are written to a cache line (when possible) and
    /// through to system memory. When writing through to memory, invalid cache lines are
    /// never filled, and valid cache lines are either filled or invalidated. Write
    /// combining is allowed. This type of cache-control is appropriate for frame buffers
    /// or when there are devices on the system bus that access system memory, but do not
    /// perform snooping of memory accesses. It enforces coherency between caches in the
    /// processors and system memory.
    ///
    /// `write-back`: Writes and reads to and from system memory are cached. Reads come
    /// from cache lines on cache hits; read misses cause cache fills. Speculative reads
    /// are allowed. Write misses cause cache line fills (in processor families starting
    /// with the P6 family processors), and writes are performed entirely in the cache,
    /// when possible. Write combining is allowed. The write-back memory type reduces bus
    /// traffic by eliminating many unnecessary writes to system memory. Writes to a
    /// cache line are not immediately forwarded to system memory; instead, they are
    /// accumulated in the cache. The modified cache lines are written to system memory
    /// later, when a write-back operation is performed. Write-back operations are
    /// triggered when cache lines need to be deallocated, such as when new cache lines
    /// are being allocated in a cache that is already full. They also are triggered by
    /// the mechanisms used to maintain cache consistency. This type of cache-control
    /// provides the best performance, but it requires that all devices that access
    /// system memory on the system bus be able to snoop memory accesses to insure system
    /// memory and cache coherency.
    ///
    /// Reference: [`Memory Cache Control`](../../../../../references/Intel_manual_Vol3.pdf#page=435)
    write_through: bool,

    /// Set if this entry is `uncacheable`
    cache_disable: bool,

    /// Set if this entry has been accessed
    accessed: bool,

    /// Set if this entry has been modified
    dirty: bool,

    /// Set if this entry is for an extended page size (For example, 1GB or 2MB)
    page_size: bool,

    /// Set if this entry is global (only applies when CR4.global is set)
    global: bool,

    /// Unused bit but can be used as a custom flag for implmentations
    bit_9: bool,

    /// Unused bit but can be used as a custom flag for implmentations
    bit_10: bool,

    /// Unused bit but can be used as a custom flag for implmentations
    bit_11: bool,

    /// Protection key for `Page Attribute Table`
    ///
    /// Reference: [`Page Attribute Table`](../../../../../references/Intel_manual_Vol3.pdf#page=462)
    protection_key: u8,

    /// Set if execution is disabled for this entry
    execute_disable: bool,
}

impl EntryFlags {
    /// Returns `true` is the `present` bit is set in the [`EntryFlags`]
    pub fn present(&self) -> bool {
        self.present
    }

    /// Returns `true` is the `page_size` bit is set in the [`EntryFlags`]
    pub fn page_size(&self) -> bool {
        self.page_size
    }
}


/// Various methods of caching available for memory
///
/// Reference: [`Methods of Caching Avaailable`](../../../../../references/Intel_manual_Vol3.pdf#page=434)
#[allow(dead_code)]
enum CacheType {
    /// System memory locations are not cached. All reads and writes appear on the system
    /// bus and are executed in program order without reordering. No speculative memory
    /// accesses, page-table walks, or prefetches of speculated branch targets are made.
    /// This type of cache-control is useful for memory-mapped I/O devices. When used
    /// with normal RAM, it greatly reduces processor performance.
    StrongUncacheable,

    /// Has same characteristics as the strong uncacheable (UC) memory type, except that
    /// this memory type can be overridden by programming the MTRRs for the WC memory
    /// type. This memory type is available in processor families starting from the
    /// Pentium III processors and can only be selected through the PAT.
    Uncacheable,

    ///  System memory locations are not cached (as with uncacheable memory) and
    ///  coherency is not enforced by the processor’s bus coherency protocol. Speculative
    ///  reads are allowed. Writes may be delayed and combined in the write combining
    ///  buffer (WC buffer) to reduce memory accesses. If the WC buffer is partially
    ///  filled, the writes may be delayed until the next occurrence of a serializing
    ///  event; such as an SFENCE or MFENCE instruction, CPUID or other serializing
    ///  instruction, a read or write to uncached memory, an interrupt occurrence, or an
    ///  execution of a LOCK instruction (including one with an XACQUIRE or XRELEASE
    ///  prefix). In addition, an execution of the XEND instruction (to end a
    ///  transactional region) evicts any writes that were buffered before the
    ///  corresponding execution of the XBEGIN instruction (to begin the transactional
    ///  region) before evicting any writes that were performed inside the transactional
    ///  region.
    WriteCombining,

    /// Writes and reads to and from system memory are cached. Reads come from cache
    /// lines on cache hits; read misses cause cache fills. Speculative reads are
    /// allowed. All writes are written to a cache line (when possible) and through to
    /// system memory. When writing through to memory, invalid cache lines are never
    /// filled, and valid cache lines are either filled or invalidated. Write combining
    /// is allowed. This type of cache-control is appropriate for frame buffers or when
    /// there are devices on the system bus that access system memory, but do not perform
    /// snooping of memory accesses. It enforces coherency between caches in the
    /// processors and system memory.
    WriteThrough,

    ///  Reads come from cache lines when possible, and read misses cause cache fills.
    ///  Writes are propagated to the system bus and cause corresponding cache lines on
    ///  all processors on the bus to be invalidated. Speculative reads are allowed. This
    ///  memory type is available in processor families starting from the P6 family
    ///  processors by programming the MTRRs (see Table 11-6)
    WriteProtected,
}

impl From<Entry> for EntryFlags {
    #[inline]
    fn from(entry: Entry) -> Self {
        // Get bits 62:59 for the protection key. We do want to truncate this value so we
        // explictly tell clippy to bypass this check
        #[allow(clippy::cast_possible_truncation)]
        let protection_key = ((entry.0 >> 59) & 0xf) as u8;

        Self {
            present:         entry.0 & (1 <<  0) > 0,
            writable:        entry.0 & (1 <<  1) > 0,
            user_permitted:  entry.0 & (1 <<  2) > 0,
            write_through:   entry.0 & (1 <<  3) > 0,
            cache_disable:   entry.0 & (1 <<  4) > 0,
            accessed:        entry.0 & (1 <<  5) > 0,
            dirty:           entry.0 & (1 <<  6) > 0,
            page_size:       entry.0 & (1 <<  7) > 0,
            global:          entry.0 & (1 <<  8) > 0,
            bit_9:           entry.0 & (1 <<  9) > 0,
            bit_10:          entry.0 & (1 << 10) > 0,
            bit_11:          entry.0 & (1 << 11) > 0,
            execute_disable: entry.0 & (1 << 63) > 0,
            protection_key
        }
    }
}

impl PageTable {
    /// Get a [`PageTable`] from the given `address`
    pub unsafe fn from_phys_addr(address: PhysAddr) -> &'static mut PageTable {
        // Cast the given `PhysAddr` into a pointer to the `PageTable`
        let table = address.0 as *mut PageTable;

        // Return a reference back to this table
        &mut *table
    }

    /// Get a [`PageTable`] from the current value of the `page table` register. On
    /// `x86_64` that will be `cr3`.
    pub unsafe fn current() -> &'static mut PageTable {
        let addr = cpu::read_page_table_addr();
        PageTable::from_phys_addr(PhysAddr(addr))
    }

    /// Get the starting address of this [`PageTable`]
    pub fn start_address(&self) -> PhysAddr {
        PhysAddr(&self[0] as *const _ as u64)
    }

    /// Get the [`PhysAddr`] of the entry at the given `index`.
    pub fn entry_address(&self, index: usize) -> PhysAddr {
        assert!(index < 512, "Attempted to index page table out of bounds");

        // Get the address of the beginning of this table
        let table_start = self.start_address();

        // Add the offset to reach the given index
        table_start.offset((core::mem::size_of::<Entry>() * index) as u64)
    }

    /// Return an [`Iter`] of the internal array of [`Entry`]
    pub fn iter(&self) -> Iter<Entry> {
        self.entries.iter()
    }

    /// Return an [`IterMut`] of the internal array of [`Entry`]
    pub fn iter_mut(&mut self) -> IterMut<Entry> {
        self.entries.iter_mut()
    }
}

impl CanTranslate for PageTable {
    /// Translate the given [`VirtAddr`] into the corresponding [`PhysAddr`] by walking
    /// the 4-level page table
    fn _translate(&self, virt_addr: VirtAddr, 
            _print: Option<&dyn Fn(core::fmt::Arguments)>) -> Result<Translated> {
        #[cfg(feature = "verbose")]
        let print = _print.unwrap();

        // verbose-only print using the passed in print callback
        macro_rules! print {
            ($($arg:tt)*) => { 
                #[cfg(feature = "verbose")]
                print(format_args!($($arg)*)); 
            }
        }

        // Get the table indexes for each level of the page table walk
        let indexes = virt_addr.table_indexes();

        // Start at the current table
        let mut table_address = self.start_address();

        // Init return address
        let mut address = None;

        let mut perms = Permissions {
            readable:   true,
            writable:   false,
            executable: false,
        };

        // Empty intermediate entries
        let mut entries = [None; 4];

        for (level, index) in indexes.iter().enumerate() {
            print!("Translate [{}] table addr: {:x?} ", level, table_address);

            // Use this address as the next page table
            let table = unsafe { PageTable::from_phys_addr(table_address) };

            // Get the entry for the current page table level
            let entry = table[*index];

            print!("entry: {:#x} ", entry.0);

            // Write the entry address into the entries array
            entries[level] = Some(table.entry_address(*index));

            // Get the flags for this entry
            let flags = entry.flags();

            // If the current entry is not present, return the current translation state
            if !flags.present() {
                return Ok(Translated::new_not_present(virt_addr, entries));    
            }

            // Update the perms for the entry based on the entry flags
            perms.writable   = flags.writable;
            perms.executable = !flags.execute_disable;

            // Get the address of the next page table from this entry
            let next_table_address = entry.address();

            print!("next table addr: {:#x}\n", next_table_address.0);

            // If `page_size` is set, then this entry corresponds to a larger page
            if flags.page_size() {
                // Get the page size and offset into the page for a large page
                let (size, offset) = match level {
                    0 => panic!("Page size on level 0?!"),
                    1 => {
                        let offset = virt_addr.0 & (512 * 1024 * 1024 * 1024 - 1);
                        (PageSize::Size512G, offset)
                    }
                    2 => {
                        let offset = virt_addr.0 & (2 * 1024 * 1024 - 1);
                        (PageSize::Size2M, offset)
                    }
                    3 => panic!("Large page with 4k page?!"),
                    _ => unreachable!()
                };

                print!("offset: {:#x}\n", offset);

                let res = Translated::new(virt_addr, next_table_address.offset(offset), 
                        size, entries, perms);    

                print!("FOUND: {:x?}\n", res);

                return Ok(res);
            }

            // Set the table address for the next iteration
            table_address = next_table_address;

            // Set the address for the final page
            address = Some(next_table_address);
        }

        let offset = virt_addr.0 & (4 * 1024 - 1);

        // Return the final address found
        let res = Translated::new(virt_addr, address.unwrap().offset(offset), PageSize::Size4K, 
                entries, perms);

        print!("FOUND: {:x?}\n", res);

        Ok(res)
    }
}

impl CanMap for PageTable {
    fn _map_raw<P: PhysMem>(&self, entry: Entry, virt_addr: VirtAddr, 
            entry_size: PageSize, phys_mem: &mut P, 
            _print: Option<&dyn Fn(core::fmt::Arguments)>) -> Result<()> {
        // verbose-only print using the passed in print callback
        macro_rules! print {
            ($($arg:tt)*) => { 
                #[cfg(feature = "verbose")]
                _print.unwrap()(format_args!($($arg)*)); 
            }
        }

        // Can only map physical addresses that are page aligned
        if !entry.address().is_page_aligned() {
            return err!(&Error::CannotMapNonPageAligned);
        }

        print!("[map_raw] Mapping {:#x} -> {:#x}\n", virt_addr.0, entry.0);

        // Get the current translation for this virtual address
        let mut translation = self._translate(virt_addr, _print)?;   

        print!("map_raw before: {:x?}\n", translation);

        // If the translation is valid, return the translation
        if translation.phys_addr.is_some() {
            print!("Translation already exists?!\n");
            return err!(&Error::VirtAddrAlreadyMapped);
        }

        // Maximum number of levels to traverse for the given entry size
        let max_depth = match entry_size {
            PageSize::Size512G => 2,
            PageSize::Size2M   => 3,
            PageSize::Size4K   => 4,
        };

        // Walk the levels of translation for this virtual address, allocating
        // intermediate pages as necessary to reach the final translation layer. This
        // loop iterates over depth indexes rather than `entries` directly because once
        // an empty page has been found, the previous page must be written to. This would
        // cause a problem for the borrow checker, so we use indexes instead.
        for curr_depth in 1..max_depth {
            print!("[{}] entry addr: {:x?}\n", curr_depth, 
                    translation.entries[curr_depth]);

            // If this translation layer already exists, no need to allocate it. Continue
            // looking for the first empty page
            if translation.entries[curr_depth].is_some() {
                continue;
            }

            // Found an empty page needed for this translation. Allocate a new one.
            let new_page_table_addr = phys_mem.alloc_page_zeroed()?;

            print!("new_page_table: {:x?}\n", new_page_table_addr);
    
            // Create the page entry for the PREVIOUS translation that will point to this
            // newly allocated page
            let new_entry = EntryBuilder::default()
                .address(new_page_table_addr)
                .present(true)
                .user_permitted(true)
                .writable(true)
                .execute_disable(false)
                .page_size(PageSize::Size4K)
                .finish();

            // Calculate the index into the table that his new entry must be written to
            let next_table_index = virt_addr.table_indexes()[curr_depth];

            // Get a `PageTable` at the allocated physical address
            let new_page_table = unsafe {
                PageTable::from_phys_addr(new_page_table_addr)
            };

            // Get the address to the 
            let next_entry_address = new_page_table.entry_address(next_table_index);

            print!("[{}] next entry addr: {:#x}\n", curr_depth, next_entry_address.0);

            // This cannot underflow since curr_depth begins at 1
            if let Some(entry_addr) = translation.entries[curr_depth - 1] {
                // Write the previous entry at the physical address of the entry_addr
                unsafe { entry_addr.write_u64(new_entry.0); }

                print!("[{}] Writing {:#x} = {:#x}\n", curr_depth, entry_addr.0, 
                    new_entry.0);

                // Update the translation with the newly created entry
                translation.entries[curr_depth] = Some(next_entry_address);

                // Get the current translation for this virtual address
                #[cfg(feature = "verbose")]
                {
                    let mut self_translation = self._translate(virt_addr, _print)?;   
                    print!("[{}] self_translation: {:x?}\n", curr_depth, self_translation);
                    print!("[{}] translation: {:x?}\n", curr_depth, translation);
                }
            }
        }

        let curr_depth = max_depth - 1;

        // This cannot underflow since curr_depth begins at 1
        if let Some(entry_addr) = translation.entries[curr_depth] {
            print!("[{}] Writing {:#x} = {:#x}\n", curr_depth, entry_addr.0, entry.0);

            // Write the previous entry at the physical address of the entry_addr
            unsafe { entry_addr.write_u64(entry.0); }
        }

        Ok(())
    }
}

impl CanUpdatePerms for PageTable {
    fn _update_perms(&mut self, virt_addr: VirtAddr, perms: Permissions,
            _print: Option<&dyn Fn(core::fmt::Arguments)>) -> Result<()> {
        let translation = self._translate(virt_addr, _print)?;   

        for entry_addr in &translation.entries {
            if let Some(entry) = entry_addr {
                let mut curr_entry = unsafe { Entry(entry.read_u64()) };
                if perms.writable {
                    curr_entry.set_writable();
                }

                if perms.executable {
                    curr_entry.set_executable();
                }
            }
        }

        Ok(())
    }
}

impl Index<usize> for PageTable {
    type Output = Entry;

    #[inline]
    fn index(&self, val: usize) -> &Self::Output {
        &self.entries[val]
    }
}

impl IndexMut<usize> for PageTable {
    #[inline]
    fn index_mut(&mut self, val: usize) -> &mut Self::Output {
        &mut self.entries[val]
    }
}

/// Builder struct to create an [`Entry`]
#[derive(Default)]
pub struct EntryBuilder {
    present: bool, 
    writable: bool, 
    user_permitted: bool, 
    write_through: bool, 
    cache_disable: bool, 
    accessed: bool, 
    dirty: bool, 
    page_size: Option<PageSize>, 
    global: bool, 
    execute_disable: bool, 
    protection_key: u8,
    address: u64
}

impl EntryBuilder {
    pub fn present(mut self, flag: bool) -> Self {
        self.present = flag;
        self
    }

    pub fn writable(mut self, flag: bool) -> Self {
        self.writable = flag;
        self
    }

    pub fn user_permitted(mut self, flag: bool) -> Self {
        self.user_permitted = flag;
        self
    }

    pub fn write_through(mut self, flag: bool) -> Self {
        self.write_through = flag;
        self
    }

    pub fn cache_disable(mut self, flag: bool) -> Self {
        self.cache_disable = flag;
        self
    }

    pub fn accessed(mut self, flag: bool) -> Self {
        self.accessed = flag;
        self
    }

    pub fn dirty(mut self, flag: bool) -> Self {
        self.dirty = flag;
        self
    }

    pub fn page_size(mut self, page_size: PageSize) -> Self {

        self.page_size = Some(page_size);
        self
    }

    pub fn global(mut self, flag: bool) -> Self {
        self.global = flag;
        self
    }

    pub fn execute_disable(mut self, flag: bool) -> Self {
        self.execute_disable = flag;
        self
    }

    pub fn protection_key(mut self, key: u8) -> Self {
        self.protection_key = key;
        self
    }

    pub fn address(mut self, address: PhysAddr) -> Self {
        assert!(address.is_page_aligned(), "Must have page aligned address for Entry");
        self.address = address.0;
        self
    }

    pub fn finish(self) -> Entry {
        let mut entry: u64 = 0; 

        entry |= self.address;
        entry |= u64::from(self.present) << 0;
        entry |= u64::from(self.writable) << 1;
        entry |= u64::from(self.user_permitted) << 2;
        entry |= u64::from(self.write_through) << 3;
        entry |= u64::from(self.cache_disable) << 4;
        entry |= u64::from(self.accessed) << 5;
        entry |= u64::from(self.dirty) << 6;

        // Only set the page_size bit if the entry is NOT a 4KiB entry
        let page_size = self.page_size.expect("No page size set");
        entry |= u64::from(page_size != PageSize::Size4K) << 7;

        entry |= u64::from(self.global) << 8;
        entry |= u64::from(self.execute_disable) << 63;
        entry |= u64::from(self.protection_key) << 59;

        Entry(entry)
    }
}
