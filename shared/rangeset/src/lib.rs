//! Organization of `u64` ranges used for physical memory management
//!
//! This is a re-implemention of the [`RangeSet`] from gamozolab's Chocolate Milk

#![no_std]

use global_types::PhysAddr;

use errchain::prelude::*;

/// Number of allocated memory slots available to represent the [`RangeSet`]
const MAX_MEMORY_RANGES: usize = 130;

/// Various errors that [`RangeSet`] can cause
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub enum RangeSetError {
    /// No more free slots in the [`RangeSet`] for another [`InclusiveRange`]
    Full,

    /// An [`InclusiveRange`]'s end is some how less than its start
    InvalidRange,

    /// Attempted to create a zero sized allocation
    ZeroSizedAllocation,

    /// Attempted to create an allocation that was not aligned to a power of two
    UnalignedAllocation,

    /// Attempted to delete an element out of bounds of the current [`RangeSet`]
    DeleteOutOfBounds,
}

/// A range that is inclusive of the final element.
///
/// base + offset structure cannot work due to not being able to represent a full memory
/// range via offset. If we wanted to represent the full 32-bit memory space, 2^32 cannot
/// be represented in a number. 
///
/// Example:
///
/// ```
/// InclusiveRange::new(0, 5) -> [0, 1, 2, 3, 4, 5]
/// ```
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct InclusiveRange {
    /// Start address of the range
    pub start: u64,

    /// End address of the range (inclusive)
    pub end: u64
}

impl InclusiveRange {
    /// Construct a new [`InclusiveRange`] from the given `start` and `end`
    ///
    /// ## Parameters
    ///
    /// * `start` - Beginning address of the [`InclusiveRange`]
    /// * `end`   - End address (inclusive) of the [`InclusiveRange`]
    pub const fn new(start: u64, end: u64) -> InclusiveRange {
        InclusiveRange { start, end }
    }

    /// Returns if the given range's start is strictly less than the range's end
    ///
    /// ## Returns
    ///
    /// `true` if the [`self.start`] is less than or equal to [`self.end`]; else `false`
    pub fn is_valid(&self) -> bool {
        self.start <= self.end
    }

    /// Returns if `self` is fully encapsulated by the given `InclusiveRange`
    ///
    /// ## Parameters
    ///
    /// * `rhs` - An [`InclusiveRange`] to check if it is fully encapsulated by `self`
    ///
    /// ## Returns
    ///
    /// `true` if the [`self.start`] is less than or equal to [`rhs.start`] and 
    ///               [`self.end`] is less than or equal to [`rhs.end`]; else `false`
    ///
    /// ## Errors
    ///
    /// If either `self` or `rhs` are invalid [`InclusiveRange`]s
    #[allow(dead_code)]
    pub fn contains(&self, rhs: &InclusiveRange) -> Result<bool> {
        ensure!(self.is_valid(), &RangeSetError::InvalidRange);
        ensure!(rhs.is_valid(),  &RangeSetError::InvalidRange);

        Ok(self.start <= rhs.start && self.end >= rhs.end)
    }

    /// Returns the overlapping region of the given `InclusiveRange` if it overlaps 
    /// with `self`
    pub fn overlaps(&self, rhs: &InclusiveRange) -> Result<Option<InclusiveRange>> {
        ensure!(self.is_valid(), &RangeSetError::InvalidRange);
        ensure!(rhs.is_valid(),  &RangeSetError::InvalidRange);

        // Since the above checks pass, this simple check works
        //
        // [   ]             [   ]
        //   {   }       {      }
        // ----------------------------
        //         returns
        //   ( )             (  )
        if self.start <= rhs.end + 1 && rhs.start <= self.end + 1 { 
            Ok(Some(InclusiveRange {
                start: core::cmp::max(self.start, rhs.start),
                end:   core::cmp::min(self.end + 1, rhs.end + 1)
            }))
        } else {
            Ok(None)
        }
    }

    /// The length of this [`InclusiveRange`]
    pub fn len(&self) -> u64 {
        if self.end == 0 && self.start == 0 {
            return 0;
        }

        self.end - self.start + 1
    }
}

/// Total memory available in a system as an array of ranges
#[derive(Clone, Copy)]
pub struct RangeSet {
    /// All [`InclusiveRange`] currently allocated for the system
    pub all_ranges: [InclusiveRange; MAX_MEMORY_RANGES],

    /// Length of the ranges currently being used
    pub length: usize,
}

impl core::fmt::Debug for RangeSet {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "RangeSet {{ all_ranges: ")?;
        for range in self.all_ranges.iter() {
            if range.len() == 0 {
                continue;
            }

            write!(f, "{:x?} ", range)?;
        }

        write!(f, "length: {} }}", self.length)?;

        core::result::Result::Ok(())
    }
}

impl RangeSet {
    /// Get an empty [`RangeSet`]
    pub const fn new() -> RangeSet {
        RangeSet {
            all_ranges: [InclusiveRange::new(0, 0); MAX_MEMORY_RANGES],
            length:     0
        }
    }

    /// Clear the [`RangeSet`]
    pub fn clear(&mut self) {
        self.all_ranges = [InclusiveRange::new(0, 0); MAX_MEMORY_RANGES];
        self.length = 0;
    }

    /// Return the number of ranges in [`RangeSet`]
    pub fn len(&self) -> usize {
        self.length
    }

    /// Return the currently used ranges in [`RangeSet`]
    pub fn ranges(&self) -> &[InclusiveRange] {
        &self.all_ranges[..self.len()]
    }
    
    /// Return the total size of memory covered by the ranges
    #[allow(dead_code)]
    pub fn size(&self) -> Result<u64> {
        let mut acc = 0;
        for range in self.ranges().iter() {
            ensure!(range.is_valid(), &RangeSetError::InvalidRange);

            // This sub can't fail if it passes the `is_valid` check
            acc += add!((range.end - range.start), 1);
        }

        Ok(acc)
    }


    /// Deletes an element by swapping the given `index` with the last index and then
    /// reducing the length of available ranges by one. 
    ///
    /// Example:
    ///
    /// ```test
    /// State: len()=0              State: len()=1              State: len()=2
    /// free ------|                   free ----|                     free |
    ///  |              Allocate()      |          Allocate()          |
    ///  v                              v                              v
    /// [a, b, c, d]                [a, b, c, d]                [0, 1, 2, 3]
    ///                             |--|                        |-----|
    ///                           used                        used
    /// Delete(0)
    ///
    /// State: len()=2
    ///    free
    ///     |
    ///     v
    /// [1, 0, 2, 3]
    /// |--|
    /// used
    /// ```
    fn delete(&mut self, index: usize) -> Result<()> {
        ensure!(index < self.len(), &RangeSetError::DeleteOutOfBounds);

        // Swap the index with the last currently in use element
        let last_in_use_index = self.len() - 1;
        self.all_ranges.swap(index, last_in_use_index);

        // Reduce the length by one
        self.length -= 1;

        Ok(())
    }

    /// Inserts the given range into the available [`RangeSet`]. If the range overlaps any
    /// existing memory regions, those regions are merged together.
    pub fn insert(&mut self, mut range: InclusiveRange) -> Result<()> {
        ensure!(range.is_valid(),               &RangeSetError::InvalidRange);
        ensure!(self.len() < MAX_MEMORY_RANGES, &RangeSetError::Full);

        'merging: loop {
            for index in 0..self.len() {
                // Get the current range
                let curr_range = self.all_ranges[index];

                // Check if the given range overlaps with the current range.
                // If an overlap is found, the given range will be extended to fit the
                // overlapping range, and then it will be deleted
                if curr_range.overlaps(&range)?.is_none() {
                    continue;
                }

                // Expand the given range to fit the overlapping range
                range.start = core::cmp::min(range.start, curr_range.start);
                range.end   = core::cmp::max(range.end,   curr_range.end);

                // Now delete the engulfed range
                self.delete(index)?;

                // Restart the loop to see if anything else must be merged
                continue 'merging;
            }

            // No merge found, can now insert the range into the memory
            break;
        }

        // No more merging needs to occur, so we can insert the range that has engulfed
        // all inner ranges

        // Base case of insertion
        self.all_ranges[self.len()] = range;
        self.length += 1;

        Ok(())
    }

    /// Remove the given [`InclusiveRange`] from the current [`RangeSet`]
    #[allow(dead_code)]
    pub fn remove(&mut self, range: InclusiveRange) -> Result<()> {
        ensure!(range.is_valid(), &RangeSetError::InvalidRange);

        'removing: loop {
            for index in 0..self.len() {
                // Get the current range
                let curr_range = self.all_ranges[index];

                // Check if the given range overlaps with the current range.
                // If an overlap is found, the given range will be shrunk to remove
                // the requested range
                if range.overlaps(&curr_range)?.is_none() {
                    continue;
                }

                // If the current range is completely engulfed by the given range,
                // we can delete it since the given range will also be deleted
                if range.contains(&curr_range)? {
                    // Delete the current range by index
                    self.delete(index)?;

                    // Restart the loop to look for which regions to remove
                    continue 'removing;
                }

                if range.start <= curr_range.start {
                    self.all_ranges[index].start = range.end.saturating_add(1);
                } else if range.end >= curr_range.end {
                    self.all_ranges[index].end  = range.start.saturating_sub(1);
                } else {
                    // Current [----------------------]
                    // Remove        [---------]
                    //
                    // Result  [----]           [-----]
                    ensure!(self.len() < MAX_MEMORY_RANGES, &RangeSetError::Full);
                        

                    // Cache the old end of the current range
                    let old_end = curr_range.end;

                    // Shrink the current range to be the left result
                    self.all_ranges[index].end = range.start.saturating_sub(1);

                    // Create the new shorted right result
                    let new_range = InclusiveRange::new(
                        range.end.saturating_add(1),
                        old_end
                    );

                    // Insert the new range into the ranges
                    self.all_ranges[self.len()] = new_range;
                    self.length += 1;
                    continue 'removing;
                }
            }

            // No more slicing
            break;
        }

        Ok(())
    }

    /// Attempts to allocate a `size` length region aligned to `align`. Will iterate 
    /// through all available ranges looking for a range that requires the least amount
    /// of padding to return the requested aligned range.
    #[allow(dead_code)]
    pub fn allocate(&mut self, size: u64, align: u64) -> Result<u64> {
        ensure!(size > 0,                &RangeSetError::ZeroSizedAllocation);
        ensure!(align.count_ones() == 1, &RangeSetError::UnalignedAllocation);

        // Since align is a power of 2, align - 1 is guarenteed to be a mask.
        //
        // Example:
        //
        // align = 0x1000
        // mask  = align - 1 = 0xfff
        let mask = align - 1;

        let mut best_padding = u64::MAX;
        let mut allocation = None;

        for index in 0..self.len() {
            let range = self.all_ranges[index];

            // Calculate the amount of bytes needed to pad from the start of this entry
            // in order to be the required alignment
            // 
            // start: 0xdead, align: 0x1000
            // padding = (0x1000 - (0xdead & 0xfff) & 0xfff
            // padding = 0x153
            // 0xdead + 0x153 = 0xe000
            let padding = (align - (range.start & mask)) & mask;

            // Calculate the aligned address
            let aligned_start = add!(range.start, padding);

            // Calculate the inclusive end of the region
            let end = add!(aligned_start, size - 1);

            // Check if the calculated region will fit in a pointer
            if range.start > usize::MAX as u64 || end > usize::MAX as u64 {
                continue;
            }

            // If the calculated end exceeds the end of the current range, 
            // continue looking
            if end > range.end {
                continue;
            }

            // Found a better segment for aligning
            if allocation.is_none() || best_padding > padding {
                // Best case found, return early
                if padding == 0 {
                    self.remove(InclusiveRange::new(range.start, end))?;

                    return Ok(aligned_start);
                }

                // Update best padding to current padding
                best_padding = padding;

                // Update allocation with current best allocation
                // allocation = Some((range.start, end, aligned_start));
                allocation = Some((aligned_start, end));
            }
        }

        match allocation {
            Some((return_addr, end)) => {
                self.remove(InclusiveRange::new(return_addr, end))?;

                // Successful allocation
                Ok(return_addr)
            }

            // This code is not unreachable, not sure why the compiler thinks it is..
            #[allow(unreachable_code)]
            _ => unreachable!()
        }
    }
}

impl phys_mem::PhysMem for RangeSet {
    unsafe fn get_mut_slice(&mut self, phys_addr: PhysAddr, size: usize) 
        -> &mut [u8] {
        core::slice::from_raw_parts_mut(phys_addr.0 as *mut u8, size)
    }

    /// Allocate a physical address with the given [`Layout`](core::alloc::Layout)
    fn alloc_phys(&mut self, layout: core::alloc::Layout) -> Result<PhysAddr> {
        let res = self.allocate(layout.size() as u64, layout.align() as u64)
            .expect("Failed to alloc_phys");
        Ok(PhysAddr(res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::*;

    #[cfg(target_arch="x86_64")]
    extern crate std;

    #[cfg(target_arch="x86_64")]
    use std::print;

    const HEADER_LEN: usize = 48;

    pub fn ascii_headers() {
        for i in 0..HEADER_LEN {
            print!("{}", i / 10);
        }
        print!("\n");
        for i in 0..HEADER_LEN {
            print!("{}", i % 10);
        }
        print!("\n");
    }

    // Print out the InclusiveRange in an ascii diagram.
    //                              0123
    // InclusiveRange::new(0, 3) -> [  ]
    pub fn ascii_print(range: &InclusiveRange) {
        let is_len_one = range.len() == 1;

        for _ in 0..range.start {
            print!(" ");
        }
        if is_len_one {
            print!("|");
        } else {
            print!("[");
        }
        for _ in range.start+1..range.end {
            print!(" ");
        }

        if !is_len_one {
            print!("]");
        }
        if range.end < HEADER_LEN  as u64 {
            for _ in range.end..HEADER_LEN as u64 {
                print!(" ");
            }
        }

        print!("{:?}\n", range);
    }

    #[test]
    #[should_panic]
    fn test_max_sum() {
        let mut mem = RangeSet::new();
        mem.all_ranges[0] = InclusiveRange { start: 0, end: u64::MAX };
        mem.length = 1;
        let _ = mem.size();
    }

    #[test]
    fn test_insert() {
        fn test() -> Result<()> {
            ascii_headers();
            let mut mem = RangeSet::new();
            mem.insert(InclusiveRange { start: 0, end: 1 });
            mem.insert(InclusiveRange { start: 4, end: 5 });
            for range in mem.ranges() {
                ascii_print(&range);
            }

            ensure!(mem.ranges() == &[
                InclusiveRange { start: 0,  end: 1 },
                InclusiveRange { start: 4,  end: 5 },
            ], "Wrong insert 1 in test_insert");

            ascii_headers();
            mem.insert(InclusiveRange { start: 2, end: 3 });
            for range in mem.ranges() {
                ascii_print(&range);
            }

            ensure!(mem.ranges() == &[
                InclusiveRange { start: 0,  end: 5 },
            ], "Wrong insert 2 in test_insert");

            ascii_headers();
            mem.insert(InclusiveRange { start: 4, end: 10 });
            for range in mem.ranges() {
                ascii_print(&range);
            }

            ensure!(mem.ranges() == &[
                InclusiveRange { start: 0,  end: 10 },
            ], "Wrong insert 3 in test_insert");

            Ok(())
        }
        let res = test();
        assert_eq!(res.is_ok());
    }

    #[test]
    fn test_remove() {
        print!("Insert 1\n");
        ascii_headers();
        let mut mem = RangeSet::new();
        mem.insert(InclusiveRange { start: 0, end: 9 });
        for range in mem.ranges() {
            ascii_print(&range);
        }
        assert_eq!(mem.ranges() == [
            InclusiveRange { start: 0, end: 9 },
        ], "Wrong insert 1 in test_remove");

        print!("Remove 2\n");
        ascii_headers();
        mem.remove(InclusiveRange { start: 2, end: 6 });
        for range in mem.ranges() {
            ascii_print(&range);
        }
        assert_eq!(mem.ranges() == [
            InclusiveRange { start: 0, end: 1 },
            InclusiveRange { start: 7, end: 9 },
        ], "Wrong remove 2 in test_remove");

        print!("Insert 3\n");
        ascii_headers();
        mem.insert(InclusiveRange { start: 2, end: 6 });
        for range in mem.ranges() {
            ascii_print(&range);
        }

        assert_eq!(mem.ranges() == [
            InclusiveRange { start: 0, end: 9 },
        ], "Wrong insert 3 in test_remove");

        print!("Remove 4\n");
        ascii_headers();
        mem.remove(InclusiveRange { start: 2, end: 4 });
        for range in mem.ranges() {
            ascii_print(&range);
        }

        assert_eq!(mem.ranges() == [
            InclusiveRange { start: 0, end: 1 },
            InclusiveRange { start: 5, end: 9 },
        ], "Wrong remove 4 in test_remove");
        
        print!("Insert 5\n");
        ascii_headers();
        mem.insert(InclusiveRange { start: 3, end: 6 });
        for range in mem.ranges() {
            ascii_print(&range);
        }

        assert_eq!(mem.ranges() == [
            InclusiveRange { start: 0, end: 1 },
            InclusiveRange { start: 3, end: 9 },
        ], "Wrong insert 5 in test_remove");

        print!("Remove 6\n");
        ascii_headers();
        mem.remove(InclusiveRange { start: 7, end: 8 });
        for range in mem.ranges() {
            ascii_print(&range);
        }

        assert_eq!(mem.ranges() == [
            InclusiveRange { start: 0, end: 1 },
            InclusiveRange { start: 3, end: 6 },
            InclusiveRange { start: 9, end: 9 },
        ], "Wrong insert 5 in test_remove");
    }

    #[test]
    fn test_allocate() {
        fn test() -> Result<()> {
            ascii_headers();
            let mut mem = RangeSet::new();
            mem.insert(InclusiveRange { start: 0, end: 32 });
            print!("Insert\n");
            for range in mem.ranges() {
                ascii_print(range);
            }

            ensure!(mem.ranges() == [
                InclusiveRange { start: 0, end: 32 },
            ], "Wrong insert in test_allocate");

            ascii_headers();
            let addr = mem.allocate(5, 16).unwrap();
            print!("Allocate 1: {:#x}\n", addr);
            for range in mem.ranges() {
                ascii_print(range);
            }

            ensure!(mem.ranges() == &[
                InclusiveRange { start: 5,  end: 32  },
            ], "Wrong allocation 1 in test_allocate");
            ensure!(addr == 0, std::format!("Wrong result addr 1"));

            ascii_headers();
            let addr2 = mem.allocate(5, 16).unwrap();
            print!("Allocate 2: {:#x}\n", addr2);
            for range in mem.ranges() { ascii_print(range); }

            ensure!(addr2 == 0x10, "Wrong result addr 2");
            ensure!(mem.ranges() == &[
                InclusiveRange { start: 5,  end: 15  },
                InclusiveRange { start: 21,  end: 32  },
            ], "Wrong allocation 2 in test_allocate");


            Ok(())
        }

        let res = test();
        print!("{:?}\n", res);
        assert_eq!(res.is_ok());
    }

    #[test]
    fn test_fail_allocate() {
        let mut mem = RangeSet::new();
        mem.insert(InclusiveRange { start: 0, end: 32 });
        assert_eq!(mem.allocate(64, 0x100).is_none());
    }

    #[test]
    fn test_delete() {
        let mut mem = RangeSet::new();
        ascii_headers();
        mem.insert(InclusiveRange { start: 1, end: 4 });
        for range in mem.ranges() { ascii_print(range); }

        mem.insert(InclusiveRange { start: 0, end: 5 });
        ascii_headers();
        for range in mem.ranges() { ascii_print(range); }
        assert_eq!(mem.ranges() == &[
            InclusiveRange { start: 0,  end: 5  },
        ], "Wrong delete");
    }
}
