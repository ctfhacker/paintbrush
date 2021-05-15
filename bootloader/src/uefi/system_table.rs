//! UEFI System Table functions

use super::{TableHeader, BootServices, ConfigurationTable, RuntimeServices};

/// Wrapper around the [`SystemTable`] argument passed into `efi_main`
#[repr(transparent)]
#[allow(clippy::module_name_repetitions)]
pub struct EfiMainSystemTable {
    /// Static [`SystemTable`] passed along `efi_main`
    table: &'static SystemTable
}

#[allow(clippy::mut_from_ref)]
impl EfiMainSystemTable {
    /// Get a reference to the boot services
    pub fn boot_services(&self) -> &mut BootServices {
        unsafe { &mut *(self.table.boot_services) }
    }

    /// Get a reference to the output console
    pub fn console_out(&self) -> &mut SimpleTextOutputProtocol {
        unsafe { &mut *(self.table.console_out) }
    }

    /// Get a slice to the current configuration table
    pub fn _config_table(&self) -> &[ConfigurationTable] {
        let table_ptr   = self.table.configuration_table;
        let num_entries = self.table.number_table_entries;

        // Convert configuration table pointer to a Rust &[ConfigurationTable]
        unsafe {
            core::slice::from_raw_parts(table_ptr, num_entries)
        }
    }
}

/// Table that contains the standard input and output handles for a UEFI application, as 
/// well as pointers to the boot services and runtime services tables.
///
/// Reference: [`4.3 EFI System Table`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=165)
#[repr(C)]
pub struct SystemTable {
    /// The table header for the EFI System Table. This header contains the
    /// EFI_SYSTEM_TABLE_SIGNATURE and EFI_SYSTEM_TABLE_REVISION values along with the
    /// size of the EFI_SYSTEM_TABLE structure and a 32-bit CRC to verify that the
    /// contents of the EFI System Table are valid.
    header: TableHeader,
    
    /// A pointer to a null terminated string that identifies the vendor that produces
    /// the system firmware for the platform.
    firmware_vendor:   *const u16,

    /// A firmware vendor specific value that identifies the revision of the system
    /// firmware for the platform.
    firmware_revision: u32,

    /// The handle for the active console input device. This handle must support
    /// [EFI_SIMPLE_TEXT_INPUT_PROTOCOL](SimpleTextInputProtocol) 
    /// and EFI_SIMPLE_TEXT_INPUT_EX_PROTOCOL.
    console_in_handle: *const u8,

    /// A pointer to the [EFI_SIMPLE_TEXT_INPUT_PROTOCOL](SimpleTextInputProtocol) 
    /// interface that is associated with
    /// [ConsoleInHandle](SystemTable::console_in_handle)
    console_in: *mut SimpleTextInputProtocol,

    /// The handle for the active console output device. This handle must support the
    /// [EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL](SimpleTextOutputProtocol).
    console_out_handle: *const u8,

    /// A pointer to the [EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL](SimpleTextOutputProtocol) 
    /// interface that is associated with
    /// [ConsoleOutHandle](SystemTable::console_out_handle).
    console_out: *mut SimpleTextOutputProtocol,

    /// The handle for the active standard error console device. This handle must support
    /// the EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL.
    console_err_handle: *const u8,

    /// A pointer to the [EFI_SIMPLE_TEXT_OUTPUT_PROTOCOL](SimpleTextOutputProtocol) 
    /// interface that is associated with 
    /// [StandardErrorHandle](SystemTable::console_err_handle)
    console_err: *mut SimpleTextOutputProtocol,

    /// A pointer to the [EFI Runtime Services Table](RuntimeServices).
    runtime_services: *mut RuntimeServices,

    /// A pointer to the [EFI Boot Services Table](BootServices).
    boot_services: *mut BootServices,

    /// The number of system configuration tables in the buffer
    /// [ConfigurationTable](ConfigurationTable)
    number_table_entries: usize,

    /// A pointer to the system [configuration tables](ConfigurationTable). The 
    /// number of entries in the table is
    /// [NumberOfTableEntries](SystemTable::number_table_entries).
    configuration_table: *mut ConfigurationTable
}

/// A protocol that is used to control text-based output devices
pub struct SimpleTextOutputProtocol {
    /// Resets the text output device hardware
    ///
    /// Reference: [`Reset()`](../../../../../references/UEFI_Spec_2_8_final.pdf#page=516)
    _reset:         unsafe extern fn(),

    /// Writes a string to the output device 
    ///
    /// Reference: [`OutputString()`](../../../../../references/UEFI_Spec_2_8_final.pdf#page=517)
    output_string: unsafe extern fn(*mut SimpleTextOutputProtocol, *mut u16),
}

impl SimpleTextOutputProtocol {
    /// Safe wrapper around [`SimpleTextOutputProtocol.output_string`]
    pub fn output_string(&mut self, out_string: &mut [u16]) {
        unsafe { (self.output_string)(self, out_string.as_mut_ptr()) }
    }
}

/// A protocol that is used to obtain input from the `ConsoleIn` device
struct SimpleTextInputProtocol;
