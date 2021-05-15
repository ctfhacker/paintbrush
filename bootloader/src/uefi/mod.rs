//! EFI implementation based on the UEFI Spec 2.8
//!
//! Reference: [`UEFI_Spec_2_8_final.pdf`](../../../../../../references/UEFI_Spec_2_8_final.pdf)

mod system_table;
pub use system_table::{SystemTable, EfiMainSystemTable};

mod boot;
use boot::BootServices;

mod runtime;
use runtime::RuntimeServices;

mod status;
pub use status::Status;

mod multiprocessor;
pub use multiprocessor::{cpu_count, startup_this_ap};

pub mod tftp;

pub mod serial;

mod event;
pub use event::Event;

use errchain::prelude::*;
use rangeset::RangeSet;

/// Various errors that EFI functions can result in
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum Error {
    /// The global system table has not been set yet
    SystemTableNotFound,

    /// Failure response of call to [`BootServices.get_memory_map`]
    GetMemoryMapFailed,

    /// The memory descriptor size from [`BootServices.get_memory_map`] and the size of
    /// the locally implemented descriptor did not match
    MemoryDescriptorSizeMismatch,
    
    /// The call to [`BootServices.exit_boot_services`] failed with status
    ExitBootServicesFailed, 

    /// The call to [`BootServices.locate_protocol`] failed with status
    LocateProtocolFailed,

    /// The call to [`BootServices.locate_protocol`] failed with a null address
    LocateProtocolNullAddress,

    /// The call to [`MpServices.get_number_of_processors`] failed with status
    GetNumberOfProcessorsFailed,

    /// The call to [`MpServices.startup_this_ap`] failed with status
    StartupThisApFailed,

    /// The call to [`MpServices.startup_all_aps`] failed with status
    StartupAllAPsFailed,

    /// The call to [`MpServices.disable_core`] failed with status
    DisableCoreFailed,

    /// The call to [`MpServices.get_processor_info`] failed with status
    GetProcessorInfoFailed,

    /// The call to [`SerialIo.write`] failed with status
    SerialWriteFailed,

    /// The call to [`TftpServices.configure`] failed with status
    TftpConfigureFailed,

    /// The call to [`TftpServices.read_file`] failed with status
    TftpReadFileFailed,
}

/// Stored EFI system table passed in the entry point
static mut EFI_SYSTEM_TABLE: Option<EfiMainSystemTable> = None;

/// Set the given system table to the global singleton
pub fn use_system_table(system_table: EfiMainSystemTable) {
    unsafe { 
        // Already initialialized the system table. No need to initialize it again.
        if EFI_SYSTEM_TABLE.is_some() {
            return;
        }

        EFI_SYSTEM_TABLE = Some(system_table);
    }
}

/// Get the [`SystemTable`] singleton as mutable
fn table_mut() -> Result<&'static mut EfiMainSystemTable> {
    unsafe { 
        // If the table hasn't been set yet, panic since we should always have a table
        ensure!(EFI_SYSTEM_TABLE.is_some(), &Error::SystemTableNotFound);

        // Get the currently loaded table
        Ok(EFI_SYSTEM_TABLE.as_mut().unwrap())
    }
}

/// Get the currently loaded boot services
fn boot_services() -> Result<&'static mut BootServices> {
    Ok(table_mut()?.boot_services())
}

/// Disable the watchdog timer
pub fn disable_watchdog_timer() -> Result<()> {
    boot_services()?.disable_watchdog_timer();
    Ok(())
}

/// Data structure that precedes all standard EFI table types
#[repr(C, packed)]
pub struct TableHeader {
    /// A 64-bit signature that identifies the type of table that follows.
    signature:   u64,

    /// The revision of the EFI Specification to which this table conforms. 
    ///
    /// The upper 16 bits of this field contain the major revision value, and the lower 
    /// 16 bits contain the minor revision value. The minor revision values are binary 
    /// coded decimals and are limited to the range of `00..99`. 
    ///
    /// When printed or displayed UEFI spec revision is referred as 
    /// `(Major revision).(Minor revision upper decimal).(Minor revision lower decimal)`
    /// or `(Major revision).(Minor revision upper decimal)` in case Minor revision lower 
    /// decimal is set to 0. 
    /// For example:
    ///
    /// A specification with the revision value `((2<<16) | (30))` would be referred as 
    /// `2.3`
    ///
    /// A specification with the revision value `((2<<16) | (31))` would be referred as 
    /// `2.3.1`
    revision:    u32,

    /// The size, in bytes, of the entire table including the `EFI_TABLE_HEADER`.
    header_size: u32,

    /// The 32-bit CRC for the entire table. This value is computed by setting this field 
    /// to 0, and computing the 32-bit CRC for `HeaderSize` bytes.
    crc32:       u32,
    
    /// Reserved field
    reserved:    u32
}

/// Contains a set of GUID/pointer pairs comprised of the `ConfigurationTable` field in 
/// the EFI System Table
///
/// Reference: [`EFI_CONFIGURATION_TABLE`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=173)
#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub struct ConfigurationTable {
    /// The 128-bit GUID value that uniquely identifies the system configuration table.
    guid: Guid,

    /// A pointer to the table associated with [`Guid`].
    ///
    /// Whether this pointer is a physical address or a virtual address during runtime is
    /// determined by the [`Guid`]. The [`Guid`] associated with a given VendorTable
    /// pointer defines whether or not a particular address reported in the table gets
    /// fixed up when a call to `SetVirtualAddressMap()` is made. It is the
    /// responsibility of the specification defining the VendorTable to specify whether
    /// to convert the addresses reported in the table.
    address: usize
}


/// The 128-bit GUID value that uniquely identifies the system configuration table
#[derive(Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Guid(u32, u16, u16, [u8; 8]);

/// Efi Memory Allocation Type
///
/// Reference:
/// [`EFI_ALLOCATE_TYPE`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=235)
#[allow(dead_code)]
enum EfiAllocateType {
    /// Allocate any available range of pages that satisfies the request. On input, the
    /// address pointed to by Memory is ignored.
    AllocateAnyPages,
    
    /// Allocate any available range of pages whose uppermost address is less than or
    /// equal to the address pointed to by Memory on input.
    AllocateMaxAddress,

    /// Allocate pages at the address pointed to by Memory on input. 
    AllocateAddress,

    /// Undocumented in the UEFI spec
    MaxAllocateType
}

/// Memory type describing a memory region as seen by EFI
#[derive(Debug, Copy, Clone)]
#[repr(u32)]
#[allow(dead_code)]
enum MemoryType {
    /// Not usable
    ReservedMemory,  

    /// The code portions of a loaded UEFI application
    LoaderCode,  
    
    /// The data portions of a loaded UEFI application and the default data allocation 
    /// type used by a UEFI application to allocate pool memory
    LoaderData,  

    /// The code portions of a loaded UEFI Boot Service Driver
    BootServicesCode,  

    /// The data portions of a loaded UEFI Boot Serve Driver, and the default data 
    /// allocation type used by a UEFI Boot Service Driver to allocate pool memory
    BootServicesData,  

    /// The code portions of a loaded UEFI Runtime Driver.
    RuntimeServicesCode,  

    /// The data portions of a loaded UEFI Runtime Driver and the default data allocation 
    /// type used by a UEFI Runtime Driver to allocate pool memor
    RuntimeServicesData,  

    /// Free (unallocated) memory
    ConventionalMemory,  

    /// Memory in which errors have been detected.
    UnusableMemory,  

    /// Memory that holds the ACPI tables. 
    AcpiReclaimMemory,  

    /// Address space reserved for use by the firmware.
    AcpiMemoryNvs,  

    /// Used by system firmware to request that a memory-mapped IO region be mapped by 
    /// the OS to a virtual address so it can be accessed by EFI runtime services.
    MemoryMappedIo,  

    /// System memory-mapped IO region that is used to translate memory cycles to IO 
    /// cycles by the processor.
    MemoryMappedIoPortSpace,  

    /// Address space reserved by the firmware for code that is part of the processor.
    PalCode,  

    /// A memory region that operates as `ConventionalMemory`. However, it happens to 
    /// also support byte-addressable non-volatility.
    PersistentMemory,  

    /// Default enum type
    Unknown
}

impl MemoryType {
    /// Returns if this memory type is free (unallocated) memory
    ///
    /// # Returns
    /// 
    /// `true` if `self` is [`MemoryType::ConventionalMemory`] or 
    /// [`MemoryType::PersistentMemory`]; else `false`
    pub fn is_available(self) -> bool {
        matches!(self, 
            MemoryType::ConventionalMemory | MemoryType::PersistentMemory)
    }

    /// Returns if this memory type is available after exiting boot services
    ///
    /// Reference: (after `exit_boot_services`): On success, the UEFI OS loader owns all 
    /// available memory in the system. In addition, the UEFI OS loader can treat all 
    /// memory in the map marked as `EfiBootServicesCode` and `EfiBootServicesData` as 
    /// available free memory.
    ///
    /// Reference: [`Explanation`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=292)
    ///
    /// # Returns
    ///
    /// `true` if `self` is [`MemoryType::BootServicesData`] or
    /// [`MemoryType::BootServicesCode`] else `false`
    pub fn _is_available_after_exit_boot_services(self) -> bool {
        matches!(self, 
            MemoryType::BootServicesData | MemoryType::BootServicesCode)
    }
}

impl Default for MemoryType {
    fn default() -> MemoryType {
        MemoryType::Unknown
    }
}

/// Describes a section of memory provided by EFI from the `get_memory_map` callback
#[derive(Debug, Copy, Clone, Default)]
#[repr(C)]
pub struct MemoryDescriptor {
    /// Type of memory region
    type_: MemoryType,

    /// Physical address of the first byte in the memory region
    physical_start: u64,

    /// Virtual address of the first byte in the memory region
    virtual_start: u64,

    /// Number of 4KiB pages in the memory region
    number_of_pages: u64,

    /// Attributes of the memory region that describe the bit mask of capabilities for
    /// that memory region and not necessarily the current settings for that memory
    /// region
    attribute: u64,

    /// Padding bytes 
    reserved: u64
}

/// Size of buffer to populate before flushing output
const OUTPUT_BUF_SIZE: usize = 128;

/// Output a string via the output console in the `SystemTable`
///
/// # Parameters
///
/// * `string` - The string to print 
pub fn output_string(string: &str) -> Result<()> {
    // Get the table pointer
    let conout = table_mut()?.console_out();

    // The +1 is to include a null terminator and a potential `\r`
    let mut res   = [0_u16; OUTPUT_BUF_SIZE + 3];
    let mut index = 0;

    // Convert the string to UCS-2
    for chr in string.encode_utf16() {
        // Add a carriage return if a newline has been seen and increment the index
        if chr == '\n' as u16 {
            res[index] = '\r' as u16;
            index += 1;
        }

        // Add the current character and increment the index pointer
        res[index] = chr;
        index += 1;

        // If we have filled the local stack, print the string and reset it
        if index >= OUTPUT_BUF_SIZE {
            // Null termiante the buffer
            res[index] = 0;

            // Print the buffer
            conout.output_string(&mut res);

            // Reset the buffer index
            index = 0;
        }
    }

    if index > 0 {
        // Null termiante the remaining buffer
        res[index] = 0;

        // Print the buffer
        conout.output_string(&mut res);
    }

    Ok(())
}

/*
/// Definition of the EFI ACPI Table GUID
const EFI_ACPI_TABLE_GUID: Guid = Guid(
    0x8868_e871,
    0xe4f1,
    0x11d3,
    [0xbc,0x22,0x00,0x80,0xc7,0x3c,0x88,0x81]
);

/// Get the ACPI base from EFI
///
/// This searches the `ConfigurationTable` from the `SystemTable` for the ACPI Table 
/// address
///
/// # Returns
///
/// * `addr` - The address to the ACPI Table found
///
/// # Errors
///
/// If [`SystemTable`] has not been set globally or if ACPI table is not found
pub unsafe fn acpi_base() -> Result<usize> {
    // Get the configuration table
    let config_table = table_mut()?.config_table();

    // Search the configuration table for the ACPI Table GUID and, if found, return the
    // address to the ACPI table
    let res = config_table.iter().find_map(|ConfigurationTable { guid, address }| {
        (guid == &EFI_ACPI_TABLE_GUID).then_some(*address)
    });

    // Check that we found the ACPI table guid and error if not
    let address = res.context_str("Failed to find ACPI TABLE GUID")?;

    Ok(address)
}
*/

/// Returns the memory map as given by EFI
///
/// # Parameters
///
/// `image_handle`: The image handle passed in [`crate::efi_main`] used to exit boot processes
///
/// # Returns
///
/// [`RangeSet`] containing the found memory map
///
/// # Errors
///
/// [`SystemTable`] has not been set globally
pub fn memory_map(_image_handle: usize) -> Result<RangeSet> {
    // Get the boot services
    let boot_services = boot_services()?;

    // Sanity check to make sure we can handle the current revision. Structures can
    // change between revisions, so if we are ever handed a different revision, we must
    // check that the structures haven't changed shape
    let rev = boot_services.revision();
    match rev {
        (2, 70) => {}
        _ => panic!("Implementation is only for structs for version 2.70")
    }

    let (available_memory, _map_key) = boot_services.get_memory_map()?;

    /*
    if false {
        // Exit the boot services
        let ret = unsafe { 
            ((*boot_services).exit_boot_services)(image_handle, map_key)
        };

        // Hard panic if exit_boot_services failed
        ensure!(ret == Status::Success, Error::ExitBootServicesFailed(ret));

        unsafe {
            // Empty the System Table
            EFI_SYSTEM_TABLE = None;
        }
    }
    */

    Ok(available_memory)
}

/// Sleep for the given `micro`seconds
///
/// # Errors
///
/// Can error during getting `boot_services`
pub fn sleep(micro: usize) -> Result<()> {
    boot_services()?.sleep(micro);

    Ok(())
}

