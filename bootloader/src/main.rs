//! Entry point for Paintbrush

#![feature(asm)]
#![feature(panic_info_message)]
#![feature(bool_to_option)]
#![feature(try_trait)]
#![feature(stmt_expr_attributes)]
#![feature(abi_efiapi)]
#![feature(exclusive_range_pattern)]
#![no_std]
#![no_main]
// Specific clippy deny requests
#![deny(missing_docs, clippy::missing_docs_in_private_items)]

// Specific clippy allow requests
#![allow(clippy::print_with_newline)]

mod uefi;
#[macro_use] mod print;

// #[macro_use] mod errchain;
// mod acpi;
mod stackvec;

#[cfg(target_arch = "x86_64")]
pub mod intel;

use core_arg::CoreArg;

use core::panic::PanicInfo;
use core::convert::TryInto;

use phys_mem::PhysMem;
use page_table::{CanMap, CanTranslate, EntryBuilder, PageSize};
use global_types::{VirtAddr, PhysAddr};

use errchain::prelude::*;

/// Total number of CPUs we can currently handle
const NUM_CPUS: usize = 36;

/// Callback function to used with `cfg("verbose")` to help debug library calls such as
/// `PageTable`
pub fn print_callback(input: core::fmt::Arguments) {
    print!("{}", input);
}

/// Panic handler
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    print!("!!!!!!!!!! PANIC PANIC PANIC PANIC !!!!!!!!!!\n");

    if let Some(location) = info.location() {
        print!("[{}:{}] ", location.file(), location.line());
    } else {
        print!("PANIC: ");
    }

    if let Some(message) = info.message() {
        print!("{} ", message);
    }

    print!("\n");

    print!("!!!!!!!!!! PANIC PANIC PANIC PANIC !!!!!!!!!!\n");

    loop {}
}

/// Real main that is called from `efi_main` and can return a `errchain::Result`
#[allow(clippy::too_many_lines)]
fn try_main(image_handle: usize, system_table: uefi::EfiMainSystemTable) -> Result<()> {
    // Set the global EFI system table from the parameter
    uefi::use_system_table(system_table);

    // Disable the watchdog timer to never auto-reboot us
    uefi::disable_watchdog_timer();

    // Get the memory map from UEFI
    let mut available_memory = uefi::memory_map(image_handle)?;

    // Sanity check that we are allocating enough CPUs
    assert!(NUM_CPUS >= uefi::cpu_count()?.total, 
        "Too few CPUs allocated for this processor");

    // Initialize the CoreArg and alive status array 
    let mut core_args   = [CoreArg::new(); NUM_CPUS];
    let mut alive_cores = [false;          NUM_CPUS];

    print!("Downloading kernel\n");

    // Allocate 2MB space to download the kernel. Will have to resize if the kernel is
    // larger than this
    let kernel_buffer_size = 1024 * 1024 * 2;
    let kernel_buffer_addr = available_memory.allocate(
        kernel_buffer_size as u64, 0x1000)?;

    print!("Kernel buffer: {:#x}\n", kernel_buffer_addr);

    // Get the slice to the kernel buffer
    let mut kernel_buffer = unsafe {
        core::slice::from_raw_parts_mut(kernel_buffer_addr as *mut u8, 
                                        kernel_buffer_size)
    };

    // Save the original available memory to use during soft reboot
    let original_available_memory = available_memory;

    // Download the kernel from the TFTP server
    uefi::tftp::read_file("paintbrush_x86.kernel", &mut kernel_buffer)?;

    // Parse the kernel from the TFTP server for the segments and entry point
    let parsed = pe::parse(&kernel_buffer)?;

    // Create a page table for the next core
    let new_page_table = unsafe { 
        page_table::PageTable::from_phys_addr(available_memory.alloc_page_zeroed()?)
    };

    // Get the current page table to map in the kernel
    let curr_page_table = unsafe { page_table::PageTable::current() };

    // This was benchmarked against using sections.iter().flatten(). Averaging over 5
    // executions of each case showed that .flatten() was slower than doing the
    // manual unpacking.
    #[allow(clippy::manual_flatten)]
    for section in &parsed.sections {
        if let Some((section_data, section_addr, perms)) = section {
            print!("..Data: {:#x} Addr: {:#x} Perms: {:?}\n", section_data.len(), 
                section_addr, perms);

            // Get the section addr as as u64
            let section_addr: u64 = (*section_addr).try_into().unwrap();

            if perms.readable && perms.executable && !perms.writable {
                // Create the entry for the executable/readable section
                let new_entry = EntryBuilder::default()
                    .address(PhysAddr(kernel_buffer_addr + section_addr))
                    .page_size(PageSize::Size4K)
                    .present(true)
                    .user_permitted(true)
                    .writable(true)
                    .execute_disable(false)
                    .finish();

                // Calculate the virtual address for this section
                let virt_addr = VirtAddr(parsed.image_base + section_addr);

                // Map the kernel into the page table for the core
                new_page_table.map_raw_4k(new_entry, virt_addr, &mut available_memory, 
                    &print_callback)?;

                // Map the kernel virtual address into the bootloader's page table
                curr_page_table.map_raw_4k(new_entry, virt_addr, &mut available_memory, 
                    &print_callback)?;
            }
        }
    }

    // Reset the available memory
    available_memory = original_available_memory;

    for core_id in 0..NUM_CPUS {
        // Get the CoreArg for this core
        let mut core_arg = core_args[core_id];

        // Ignore the first core and all cores not available on the system
        if core_id == 0 || core_id >= NUM_CPUS {
            continue;
        }

        // Modify the kernel arg for this core
        core_arg.reset();
        core_arg.set_core(core_id);

        // Get the physical address of the location to write the status of the core
        let alive_addr = &mut alive_cores[core_id] as *mut bool;
        core_arg.set_alive_address(alive_addr);

        // Amount of memory to allocate for each core
        let memory_size = 1024 * 1024 * 1024;

        // Allocate the physcial memory for this core
        let memory_start = available_memory.allocate(memory_size, 0x1000)?;
        core_arg.insert_memory(memory_start, memory_size);

        // Get the physical address of the kernel entry point
        let entry_point_phys = curr_page_table.translate(VirtAddr(parsed.entry_point), 
            &print_callback)?;

        // Cast the physical address as a function for the multiprocessor callback
        let entry_point_func = 
            entry_point_phys.phys_addr().unwrap().0 as *const fn(usize);

        // Get the address of the arguments for this core
        let core_arg_addr = &mut core_args[core_id] as *mut _ as usize;

        // Start the core
        // uefi::startup_this_ap(core_id, parsed.entry_point as usize, core_arg_addr);
        uefi::startup_this_ap(core_id, entry_point_func, core_arg_addr);
    }

    let mut all_cores_finished = false; 

    while !all_cores_finished {
        // Reset the cores check to true which will be set false if any core is still
        // working
        all_cores_finished = true; 

        print!("Cores alive: ");
        for (core_id, is_alive) in alive_cores.iter().enumerate() {
            if *is_alive {
                all_cores_finished = false;
                print!("{} ", core_id);
            }
        }
        print!("\n");

        for _ in 0..100 {
            for (id, core_arg) in core_args.iter().enumerate() {
                print!("[{}]: {:x?}\n", id, core_arg.stats.start_time);
            }
            uefi::sleep(1_000_000);
        }

        uefi::sleep(500_000);
    }


    // Get PEI Services via 8 bytes prior to IDT
    panic!("w00t! Finished!");
}

/// Main entrypoint handed off by UEFI. This only calls the real `try_main` and handles
/// the `Result` from `try_main` in the case of an `Err`
#[no_mangle]
extern fn efi_main(image_handle: usize, system_table: uefi::EfiMainSystemTable) {
    // Now jump to the main 
    if let Err(e) = try_main(image_handle, system_table) {
        panic!("EFI main top level\n{:?}\n", e);
    }

    loop {
        core::hint::spin_loop();
    }
}

/// Empty impl of `__chkstk`
#[no_mangle]
extern fn __chkstk() {}
