//! UEFI Boot Services
use core::ffi::c_void;

use errchain::prelude::*;
use rangeset::{RangeSet, InclusiveRange};
use crate::uefi::{Guid, MemoryDescriptor, TableHeader, MemoryType, Status, Error};
use crate::print;

/// Boot service table containing function pointers to the various services available
/// on boot
///
/// Reference: [`4.4 EFI Boot Services Table`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=166)
#[repr(C)]
#[allow(clippy::module_name_repetitions)]
pub struct BootServices {
    /// The table header for the EFI Boot Services Table. This header contains the
    /// EFI_BOOT_SERVICES_SIGNATURE and EFI_BOOT_SERVICES_REVISION values along with the
    /// size of the EFI_BOOT_SERVICES structure and a 32-bit CRC to verify that the
    /// contents of the EFI Boot Services Table are valid.
    header: TableHeader,

    /// Raises the task priority level. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.RaiseTPL()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=229)
    _raise_tpl:      unsafe extern fn(),

    /// Restores/lowers the task priority level.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.RestoreTPL()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=231)
    _restore_tpl:    unsafe extern fn(),

    /// Allocates pages of a particular type.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.AllocatePages()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=234)
    _allocate_pages:    unsafe extern fn(),

    /// Frees allocated pages.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.FreePages()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=237)
    _free_pages:     unsafe extern fn(),
    

    /// Returns the current boot services memory map and memory map key.
    ///
    /// Reference: [`EFI_BOOT_SERVICES.GetMemoryMap()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=237)
    get_memory_map: unsafe extern fn(
        map_size:           &mut usize, 
        memory_map:         *mut u8,
        map_key:            &mut usize,
        descriptor_size:    &mut usize,
        descriptor_version: &mut u32
    ) -> Status,

    /// Allocates a pool of a particular type. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.AllocatePool()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=241)
    _allocate_pool:  unsafe extern fn(),

    /// Frees allocated pool.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.FreePool()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=242)
    _free_pool:      unsafe extern fn(),

    /// Creates a general-purpose event structure.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.CreateEvent()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=218)
    _create_event:   unsafe extern fn(),

    /// Sets an event to be signaled at a particular time.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.SetTimer()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=227)
    _set_timer:      unsafe extern fn(),

    /// Stops execution until an event is signaled.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.WaitForEvent()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=226)
    _wait_for_event: unsafe extern fn(),

    /// Signals an event.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.SignalEvent()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=225)
    _signal_event:   unsafe extern fn(),

    /// Closes and frees an event structure.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.CloseEvent()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=224)
    _close_event:    unsafe extern fn(),

    /// Checks whether an event is in the signaled state.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.CheckEvent()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=227)
    _check_event:    unsafe extern fn(),

    /// Installs a protocol interface on a device handle.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.InstallProtocolInterface()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=247)
    _install_protocol_interface:   unsafe extern fn(),

    /// Reinstalls a protocol interface on a device handle.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.ReinstallProtocolInterface()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=252)
    _reinstall_protocol_interface: unsafe extern fn(),

    /// Removes a protocol interface from a device handle.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.UninstallProtocolInterface()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=250)
    _uninstall_protocol_interface: unsafe extern fn(),

    /// Queries a handle to determine if it supports a specified protocol. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.HandleProtocol()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=256)
    _handle_protocol:              unsafe extern fn(),

    /// Reserved. Must be NULL.
    _reserved:                     unsafe extern fn(),

    /// Registers an event that is to be signaled whenever an interface is installed for
    /// a specified protocol. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.RegisterProtocolNotify()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=253)
    _register_protocol_notify:     unsafe extern fn(),

    /// Returns an array of handles that support a specified protocol. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.LocateHandle()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=254)
    _locate_handle:                unsafe extern fn(),

    /// Locates all devices on a device path that support a specified protocol and
    /// returns the handle to the device that is closest to the path.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.LocateDevicePath()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=258)
    _locate_device_path:           unsafe extern fn(),

    /// Adds, updates, or removes a configuration table from the EFI System Table.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.InstallConfigurationTable()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=296)
    _install_configuration_table:  unsafe extern fn(),

    /// Loads an EFI image into memory. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.LoadImage()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=284)
    _load_image:         unsafe extern fn(),

    /// Transfers control to a loaded image’s entry point. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.StartImage()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=286)
    _start_image:        unsafe extern fn(),

    /// Exits the image’s entry point.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.Exit()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=289)
    _exit:               unsafe extern fn(),

    /// Unloads an image.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.UnloadImage()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=287)
    _unload_image:       unsafe extern fn(),

    /// Terminates all boot services.
    ///
    /// Reference: [`EFI_BOOT_SERVICES.ExitBootServices()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=291)
    exit_boot_services: unsafe extern fn(
        image_handle: usize,
        map_key:      usize
    ) -> Status,

    /// Returns a monotonically increasing count for the platform. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.GetNextMonotonicCount()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=296)
    _get_next_monotonic_count: unsafe extern fn(),

    /// Stalls the processor for the given `microseconds`
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.Stall()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=294)
    stall: unsafe extern fn(
        microseconds: usize
    ),

    /// Resets and sets a watchdog timer used during boot services time.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.SetWatchdogTimer()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=293)
    set_watchdog_timer: unsafe extern fn(
        timeout:       usize,
        watchdog_code: u64,
        data_size:     usize,
        watchdog_data: *const u16
    ),

    /// Uses a set of precedence rules to find the best set of drivers to manage a
    /// controller.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.ConnectController()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=269)
    _connect_controller:    unsafe extern fn(),

    /// Informs a set of drivers to stop managing a controller. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.DisconnectController()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=273)
    _disconnect_controller: unsafe extern fn(),

    /// Adds elements to the list of agents consuming a protocol interface.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.OpenProtocol()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=259)
    _open_protocol:             unsafe extern fn(),

    /// Removes elements from the list of agents consuming a protocol interface. 
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.CloseProtocol()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=265)
    _close_protocol:            unsafe extern fn(),

    /// Retrieve the list of agents that are currently consuming a protocol interface.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.OpenProtocolInformation()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=267)
    _open_protocol_information: unsafe extern fn(),

    /// Retrieves the list of protocols installed on a handle. The return buffer is
    /// automatically allocated.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.ProtocolsPerHandle()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=275)
    _protocols_per_handle: unsafe extern fn(),

    /// Retrieves the list of handles from the handle database that meet the search
    /// criteria. The return buffer is automatically allocated.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.LocateHandleBuffer()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=276)
    _locate_handle_buffer: unsafe extern fn(),

    /// Finds the first handle in the handle database the supports the requested
    /// protocol.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.LocateProtocol()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=279)
    locate_protocol: unsafe extern "efiapi" fn(
        protocol:     &Guid,
        registration: *mut u8,
        interface:    &mut *mut c_void
    ) -> Status,

    /// Installs one or more protocol interfaces onto a handle.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.InstallMultipleProtocolInterfaces()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=280)
    _install_multiple_protocol_interfaces: unsafe extern fn(),

    /// Uninstalls one or more protocol interfaces from a handle.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.UninstallMultipleProtocolInterfaces()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=281)
    _uninstall_multiple_protocol_interfaces: unsafe extern fn(),

    /// Computes and returns a 32-bit CRC for a data buffer.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.CalculateCrc32()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=297)
    _calculate_crc32: unsafe extern fn(),

    /// Copies the contents of one buffer to another buffer.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.CopyMem()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=294)
    _copy_mem:        unsafe extern fn(),

    /// Fills a buffer with a specified value.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.SetMem()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=295)
    _set_mem:         unsafe extern fn(),

    /// Creates an event structure as part of an event group.
    /// 
    /// Reference: [`EFI_BOOT_SERVICES.CreateEventEx()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=221)
    _create_event_ex: unsafe extern fn()
}

impl BootServices {
    /// Get the (major, minor) revision number from the table header
    ///
    /// # Returns
    ///
    /// (Major, Minor) revision numbers found in the header
    pub fn revision(&self) -> (u8, u16) {
        #[allow(clippy::cast_possible_truncation)]
        let version = (self.header.revision >> 16) as u8;
        
        #[allow(clippy::cast_possible_truncation)]
        let subver  = self.header.revision as u16;
        (version, subver)
    }

    /// Disable the watchdog timer
    pub fn disable_watchdog_timer(&self) {
        unsafe {
            (self.set_watchdog_timer)(
                /* timeout:       */ 0,
                /* watchdog_code: */ 0,
                /* data_size:     */ 0,
                /* watchdog_data: */ core::ptr::null()
            )
        }
    }

    /// Stall the processor for the given `micro`seconds
    #[allow(dead_code)]
    pub fn stall(&self, micro: usize) {
        unsafe { 
            (self.stall)(micro);
        }
    }

    /// Stall the processor for the given `micro`seconds
    #[allow(dead_code)]
    pub fn sleep(&self, micro: usize) {
        self.stall(micro);
    }

    /// Get the memory map as a [`RangeSet`] 
    pub fn get_memory_map(&mut self) -> Result<(RangeSet, usize)> {
        // Allocate 4KiB to receive the memory map
        let mut output_map         = [MemoryDescriptor::default(); 512];

        let mut memory_map_size    = core::mem::size_of_val(&output_map);
        let mut map_key            = 0;
        let mut descriptor_size    = 0;
        let mut descriptor_version = 0;

        // Call the get_memory_map callback
        unsafe {
            let ret = (self.get_memory_map)(
                &mut memory_map_size,
                output_map.as_mut_ptr().cast::<u8>(),
                &mut map_key,
                &mut descriptor_size,
                &mut descriptor_version
            );

            // Ensure successful return from get_memory_map
            // This will hard panic, because we must get a memory map in order to 
            // progress in the kernel
            if ret != Status::Success {
                print!("[boot::get_memory_map] Error: {:?}\n", ret);
                return err!(&Error::GetMemoryMapFailed);
            }
        }

        // Ensure our descriptor struct has the same size as the descriptor length
        // returned from `get_memory_size`
        ensure!(descriptor_size == core::mem::size_of::<MemoryDescriptor>(),
            &Error::MemoryDescriptorSizeMismatch);

        let mut available_memory = RangeSet::new();

        /*
        // Iterate through the memory map by the given descriptor size from the call to
        // `get_memory_map`
        for (curr_entry, _) in (0..memory_map_size).step_by(descriptor_size).enumerate() {
            // Read the bytes for the memory descriptor of the current entry
            let mem = &output_map[
                curr_entry * descriptor_size..(curr_entry + 1) * descriptor_size
            ];
        */

        for mem in output_map.iter() {
            // Read those bytes as a Rust structure
            // let mem = unsafe { *(bytes.as_ptr().cast::<MemoryDescriptor>()) };

            // The first instance of an `Unknown` memory type is the end of the found memory
            if matches!(mem.type_, MemoryType::Unknown) {
                break;
            }

            // Check if the current memory is marked as free now or free after we reclaim
            // memory after exiting boot services
            // if mem.type_.is_available() || mem.type_.is_available_after_exit_boot_services() {
            if mem.type_.is_available() {
                // Calculate the inclusive memory end address
                // let end   = mem.physical_start + mul!(mem.number_of_pages, 4096) - 1;
                let end   = mem.physical_start + (mem.number_of_pages * 4096) - 1;

                // Create an InclusiveRange to insert into the RangeSet
                let entry = InclusiveRange::new(mem.physical_start, end);

                // Add the memory to the resulting array
                available_memory.insert(entry)?;
            }
        }

        Ok((available_memory, map_key))
    }

    /// Return first protocol instance that matches the protocol with the given [`Guid`]
    /// without a registration.
    pub fn locate_protocol(&self, guid: &Guid) -> Result<*mut c_void> {
        // Initialize the return function pointer
        let mut addr = core::ptr::null_mut();

        // Call the locate protocol function
        unsafe {
            let ret = (self.locate_protocol)(&guid, core::ptr::null_mut(), &mut addr);

            // Ensure `locate_protocol` returned success
            if ret != Status::Success {
                print!("[boot::locate_protocol] Error: {:?}\n", ret);
                return err!(&Error::LocateProtocolFailed);
            }

            if addr.is_null() {
                return err!(&Error::LocateProtocolNullAddress);
            }
        }

        Ok(addr)
    }
}

