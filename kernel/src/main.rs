//! Functionality isolated to each individual core

#![feature(asm)]
#![feature(global_asm)]
#![no_std]
#![no_main]

extern crate compiler_builtins;

use errchain::prelude::*;
use core_arg::CoreArg;

/// Entry point called from the UEFI bootloader
#[no_mangle]
pub fn kernel_main(arg: usize) {
    // Get access to the `CoreArg` passed from the bootlaoder
    let mut arg = unsafe { &mut *(arg as *mut CoreArg) };

    // Ensure the correct core and alive address from the CoreArg
    assert!(arg.core.is_some(), "Core ID not set in CoreArg");
    assert!(arg.alive_address.is_some(), "Alive address not set in CoreArg");

    // Get access to the core id and address to notify the bootloader that this core is
    // alive
    let core_id = arg.core.unwrap();
    let alive_address = arg.alive_address.unwrap();

    // Set the start time for this core
    arg.stats.start_time = unsafe { core::arch::x86_64::_rdtsc() as usize };

    // Set that this core is alive
    unsafe { alive_address.write(true); }

    let res = try_main(core_id, &mut arg);

    // Set that this core is dead
    unsafe { alive_address.write(false); }

    if let Err(e) = res {
        panic!("Top kernel main\n{:?}\n", e);
    }
}

/// Actual main entry point for this individual core in order to wrap the `Result`
pub fn try_main(core_id: usize, arg: &mut CoreArg) -> Result<usize> {
    let mut sum = 0;

    for _ in 0..10 {
        for _ in 0..0x7ff_ffff {
            unsafe { asm!("pause") }
        }

        sum += core_id;
        arg.stats.start_time = unsafe { core::arch::x86_64::_rdtsc() as usize };
    }

    Ok(sum)
}

/// Panic handler
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
