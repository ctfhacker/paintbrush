//! Specific `aarch64` architecture functionality

#![no_std]
#![feature(asm)]
#![cfg(target_arch="x86_64")]

use core::convert::TryInto;

/// Read the page table address from `ttbr0`
pub unsafe fn read_page_table_addr() -> u64 {
    let res: usize;
    asm!("mov {}, cr3", out(reg) res);
    res.try_into().expect("Only valid on 64 bit x86 architectures")
}
