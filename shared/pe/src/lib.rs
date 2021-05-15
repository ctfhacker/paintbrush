//! Minimalistic, no-copy PE parser used to extract sections
//!
//! Reference: [

#![no_std]

use core::convert::TryInto;
use errchain::*;

#[derive(Copy, Clone, Debug)]
pub enum Error {
    /// MZ header missing from the beginning of file
    InvalidMZHeader,

    /// PE header missing at the `pe_offset` (e_lfanew) found in the MZ header
    InvalidPEHEader,

    /// Parsed PE has too many sections for this implementation to parse. Increase the
    /// `NUM_SECTIONS` value to parse everything properly.
    TooManySections,
}

/// The architecture type of the computer. An image file can only be run on the specified
/// computer or a system that emulates the specified computer.
#[derive(Debug)]
#[repr(u16)]
pub enum Machine {
    I386  = 0x014c,
    Ia4   = 0x0200,
    Amd64 = 0x8664
}

/// The state of the image file
#[derive(Debug)]
#[repr(u16)]
pub enum Magic {
    /// The file is an 32-bit executable image. 
    Hdr32  = 0x10b,

    /// The file is an 64-bit executable image. 
    Hdr64  = 0x20b,

    ///  The file is a ROM image. 
    RomHdr = 0x107
}

/// Section permissions parsed from the [`Characteristics`]
#[derive(Debug, Copy, Clone)]
pub struct SectionPermissions {
    /// This section is executable
    pub executable: bool,

    /// This section is readable
    pub readable:   bool,

    /// This section is writable
    pub writable:   bool
}

/// Minimal
/// [`characteristics`](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_section_header) for the image
pub enum Characteristics {
    /// This section contains executable code
    Code = 0x00000020,

    /// The section can be read. 
    MemRead = 0x40000000,

    /// This section can be written to.
    MemWrite = 0x80000000,
}

/// PE Header from [`IMAGE_NT_HEADERS64`](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_nt_headers64)
#[derive(Debug)]
#[repr(C)]
struct PeHeader {
    /// A 4-byte signature identifying the file as a PE image. The bytes are `PE\0\0`.
    signature: [u8; 4],

    /// The architecture type of the computer. An image file can only be run on the
    /// specified computer or a system that emulates the specified computer.
    machine: Machine,
    
    /// The number of sections. This indicates the size of the section table, which
    /// immediately follows the headers.
    number_of_sections: u16,

    /// The low 32 bits of the time stamp of the image.
    date_stamp: u32,

    /// The offset of the symbol table, in bytes, or zero if no COFF symbol table exists.
    symbol_table_ptr: u32,

    /// The number of symbols in the symbol table.
    number_of_symbols: u32,

    /// Optional header size
    opt_header_size: u16,

    /// The characteristics of the image.
    characteristics: u16,

    /// The state of the image file.
    magic: u16,
    
    /// The linker version (major, minor) number of the linker.
    linker_version: [u8; 2],

    /// The size of the code section, in bytes, or the sum of all such sections if there
    /// are multiple code sections.
    code_size: u32,

    /// The size of the initialized data section, in bytes, or the sum of all such
    /// sections if there are multiple initialized data sections.
    init_data_size: u32,

    /// The size of the uninitialized data section, in bytes, or the sum of all such
    /// sections if there are multiple uninitialized data sections.
    uninit_data_size: u32,
    
    /// A pointer to the entry point function, relative to the image base address. For
    /// executable files, this is the starting address. For device drivers, this is the
    /// address of the initialization function. The entry point function is optional for
    /// DLLs. When no entry point is present, this member is zero
    entry_point_rva: u32,

    /// A pointer to the beginning of the code section, relative to the image base.
    code_base_rva: u32,

    /// The preferred address of the first byte of the image when it is loaded in memory.
    /// This value is a multiple of 64K bytes. The default value for DLLs is
    /// `0x10000000`.  The default value for applications is `0x00400000`, except on
    /// Windows CE where it is `0x00010000`.
    image_base: u64,
}

/// A section header from
/// [`IMAGE_SECTION_HEADER`](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_section_header)
#[derive(Debug)]
#[repr(C)]
struct Section {
    /// An 8-byte, null-padded UTF-8 string. There is no terminating null character if
    /// the string is exactly eight characters long. For longer names, this member
    /// contains a forward slash (/) followed by an ASCII representation of a decimal
    /// number that is an offset into the string table. Executable images do not use a
    /// string table and do not support section names longer than eight characters.
    name: [u8; 8],

    /// The total size of the section when loaded into memory, in bytes. If this value is
    /// greater than the SizeOfRawData member, the section is filled with zeroes. This
    /// field is valid only for executable images and should be set to 0 for object
    /// files.
    virt_size: u32,

    /// The address of the first byte of the section when loaded into memory, relative to
    /// the image base. For object files, this is the address of the first byte before
    /// relocation is applied.
    virt_addr: u32,

    /// The size of the initialized data on disk, in bytes. This value must be a multiple
    /// of the FileAlignment member of the
    /// [`IMAGE_OPTIONAL_HEADER`](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_optional_header32)
    /// structure. If this value
    /// is less than the VirtualSize member, the remainder of the section is filled with
    /// zeroes. If the section contains only uninitialized data, the member is zero.
    raw_data_size: u32,

    /// A file pointer to the first page within the COFF file. This value must be a
    /// multiple of the FileAlignment member of the
    /// [`IMAGE_OPTIONAL_HEADER`](https://docs.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-image_optional_header32)
    /// structure. If a section contains only uninitialized data, set this member is
    /// zero.
    raw_data_ptr: u32,

    /// A file pointer to the beginning of the relocation entries for the section. If
    /// there are no relocations, this value is zero.
    reloations: u32,

    /// A file pointer to the beginning of the line-number entries for the section. If
    /// there are no COFF line numbers, this value is zero.
    line_numbers: u32,

    /// The number of relocation entries for the section. This value is zero for
    /// executable images.
    num_relocations: u16,

    /// The number of line-number entries for the section.
    num_line_numbers: u16,

    /// The characteristics of the image.
    characteristics: u32
}

impl Section {
    /// Returns `true` if the section is executable
    pub fn is_executable(&self) -> bool {
        self.characteristics & Characteristics::Code as u32 > 0
    }

    /// Returns `true` if the section is readable
    pub fn is_readable(&self) -> bool {
        self.characteristics & Characteristics::MemRead as u32 > 0
    }

    /// Returns `true` if the section is writable
    pub fn is_writable(&self) -> bool {
        self.characteristics & Characteristics::MemWrite as u32 > 0
    }

    /// Get the [`SectionPermissions`] for this section
    pub fn permissions(&self) -> SectionPermissions {
        SectionPermissions {
            executable: self.is_executable(),
            readable:   self.is_readable(),
            writable:   self.is_writable()
        }
    }
}

/// Parsed information from a given PE file
pub struct Parsed<'a> {
    /// Parsed sections with their permissions
    pub sections: [Option<(&'a [u8], u32, SectionPermissions)>; 6],

    /// Requested image base for the PE file
    pub image_base: u64,

    /// Entry point of the PE file
    pub entry_point: u64
}

/// Number of sections that can be parsed and returned
const NUM_SECTIONS: u16 = 6;

pub fn parse<'a>(data: &'a [u8]) -> Result<Parsed> {
    // Ensure the data begins with MZ
    ensure!(&data[..2] == b"MZ", &Error::InvalidMZHeader);

    // Get the offset to the PE section from the MZ header
    let pe_offset = u32::from_le_bytes(data[0x3c..0x40].try_into().unwrap()) as usize;

    // Get the PE header
    let pe_header = &data[pe_offset..];

    // Ensure the PE header was found
    ensure!(&pe_header[..2] == b"PE", &Error::InvalidPEHEader);

    let header = unsafe {
        &*(pe_header[..core::mem::size_of::<PeHeader>()].as_ptr() as *const PeHeader)
    };

    ensure!(header.number_of_sections <= NUM_SECTIONS, &Error::TooManySections);

    let section_start_offset = (header.opt_header_size + 0x18) as usize;

    // Init the returned parsed sections
    let mut sections = [None; 6];

    for section_num in 0..header.number_of_sections {
        // Get the beginning of this section header
        let section_ptr = &pe_header[section_start_offset..];

        // Store the length of the section header
        let section_len = core::mem::size_of::<Section>() as usize;

        // Get the start/end of the current section header
        let section_start = section_len * section_num as usize;
        let section_end   = section_start + section_len;

        // Cast the current data location as a `Section`
        let section = unsafe {
            &*(section_ptr[section_start..section_end].as_ptr() as *const Section)
        };

        // Get the start/end of the actual section data
        let section_data_start = section.raw_data_ptr as usize;
        let section_data_end   = (section.raw_data_ptr + section.raw_data_size) as usize;

        // Store the parsed section
        sections[section_num as usize] = Some((
            &data[section_data_start..section_data_end],
            section.virt_addr,
            section.permissions()
        ));
    }

    Ok(Parsed {
        sections,
        image_base: header.image_base,
        entry_point: header.entry_point_rva as u64 + header.image_base
    })
}
