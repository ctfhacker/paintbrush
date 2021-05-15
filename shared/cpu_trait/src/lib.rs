//! Various functions a CPU can perform

#![no_std]

pub trait CpuTrait {
    /// Read the page table address for the given architecture
    fn read_page_table_addr() -> u64;

    /// Set the given `addr` to be the active page table
    fn set_page_table_addr(addr: u64);

    /// Read the current time counter
    fn read_time_counter() -> u64;
}
