//! Basic ACPI parsing functionality, focused on launching cores for the system
//!
//! Reference: [`ACPI_6_2.pdf`](../../../../../references/ACPI_6_2.pdf)

use core::mem::size_of;
use core::convert::TryInto;

use global_types::PhysAddr;

use errchain::prelude::*;
use crate::uefi;
use crate::stackvec::StackVec;

/// Various errors that Acpi can throw
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Checksum mismatch 
    InvalidChecksum,

    /// Signature mismatch for RSDP
    InvalidRsdpSignature,

    /// The revision is too old for this implementation
    InvalidRsdpRevision,

    /// Length mismatch in RSDP parsing
    InvalidRsdpLength,

    /// Signature mismatch for XSDT
    InvalidXsdtSignature,

    /// Data has been shown to be misaligned
    MisalignedData, 
}

impl ErrorType for Error {}

/// Maximum number of cores able to be used
const MAX_NUM_CPUS: usize = 48;

/// ACPI Table Signatures
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TableSignature {
    /// Root System Description Pointer
    ///
    /// Reference: [`Root System Description Pointer (RSDP)`](../../../../../references/ACPI_6_2.pdf#page=170)
    Rsdp,

    /// Extended System Description Table
    ///
    /// Reference: [`Extended Description Table (XSDT)`](../../../../../references/ACPI_6_2.pdf#page=176)
    Xsdt,

    /// Fixed ACPI Description Table
    ///
    /// Reference: [`Fixed ACPI Description Table (FADT)`](../../../../../references/ACPI_6_2.pdf#page=177)
    Facp,

    /// Multiple APIC Description Table
    /// 
    /// Reference: [`Multiple ACPI Description Table (MADT)`](../../../../../references/ACPI_6_2.pdf#page=200)
    Madt,

    /// IA-PC High Precision Event Timer Table
    ///
    /// [IA-PC HPET (High Precision Event Timers)](http://www.intel.com/content/dam/www/public/us/en/documents/technical-specifications/software-developers-hpet-spec-1-0a.pdf)
    Hpet,

    /// Boot Graphics Resource Table
    ///
    /// Reference: [`Boot Graphics Resource Table (BGRT)`](../../../../../references/ACPI_6_2.pdf#page=250)
    Bgrt,

    /// Debug Port Table 2
    ///
    /// [`MSDN Reference`](http://msdn.microsoft.com/en-us/library/windows/hardware/dn639131(v=vs.85).aspx_)
    Dbg2,

    /// Generic Timer Description Table
    /// 
    /// Reference: [`Generic Timer Description Table (GTDT)`](../../../../../references/ACPI_6_2.pdf#page=258)
    Gtdt,

    /// PCI Express memory mapped configuration space base address Description Table
    ///
    /// [`Reference`](http://www.pcisig.com/home)
    Mcfg,

    /// Serial Port Console Redirection Table
    /// 
    /// [`MSDN Reference`](http://msdn.microsoft.com/en-us/library/windows/hardware/dn639132(v=vs.85).aspx)
    Spcr,

    /// Secondary System Description Table
    ///
    /// Reference: [`Differentiated System Description Table (DSDT)`](../../../../../references/ACPI_6_2.pdf#page=198)
    Ssdt,

    /// Processor Properties Topology Table
    ///
    /// Reference: [`Processor Properties Topology Table (PPTT)`](../../../../../references/ACPI_6_2.pdf#page=295)
    Pptt,

    /*
     * MIGT
     * MSCT - Maximum System Characteristics Table - 5.2.19
     * PCAT 
     * RASF - ACPI RAS Feature Table - 5.2.20
     * SLIT - System Locality Information Table - 6.2.15
     * SRAT - System Resource Affinity TableA - 5.2.16
     * SVOS
     * WDDT
     * OEM4
     * NIT$ - 
     * MSDM - Microsoft Software Licensing Tables - Microsoft
     * LPIT - Low Power Idle Table - Microsoft
     * DBGP - Debug Port Table
     * SLIC - Microsoft Software Licensing Tables - Microsoft
     * UEFI - Unified Extensible Firmware Interface Spec - 
     * DMAR - DMA Remapping Table - External
     * HEST - Hardware Error Source Table - Table 18-371
     * BERT - Boot Error Record Table - 18.3.1
     * ERST - Error Record Serialization Table - 18.5
     * EINJ - Error Injection Table - 18.6.1
     * ASF!
     */

    /// Unknown signature found
    Unknown([char; 4])
}

impl From<[u8; 4]> for TableSignature {
    fn from(sig: [u8; 4]) -> TableSignature {
        match &sig {
            b"XSDT" => TableSignature::Xsdt,
            b"FACP" => TableSignature::Facp,
            b"APIC" => TableSignature::Madt,
            b"HPET" => TableSignature::Hpet,
            b"BGRT" => TableSignature::Bgrt,
            b"DBG2" => TableSignature::Dbg2,
            b"GTDT" => TableSignature::Gtdt,
            b"MCFG" => TableSignature::Mcfg,
            b"SPCR" => TableSignature::Spcr,
            b"SSDT" => TableSignature::Ssdt,
            b"PPTT" => TableSignature::Pptt,
            _       => TableSignature::Unknown(
                [sig[0] as char, sig[1] as char, sig[2] as char, sig[3] as char]
            )
        }
    }
}
/// ACPI checksum function
unsafe fn checksum(addr: PhysAddr, length: u64) -> Result<()> {
    // Calculate the checksum for the RSDP
    let checksum = (0..length).fold(0_u8, |acc, index| {
        let byte = addr.offset(index).read_u8();
        acc.wrapping_add(byte)
    });

    // Validate the checksum is zero
    ensure!(checksum == 0, Error::InvalidChecksum);

    Ok(())
}

/// Structure for the Root Sytem Description Pointer
///
/// Reference: [`Root System Description Pointer (RSDP)`](../../../../../references/ACPI_6_2.pdf#page=170)
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct Rsdp {
    /// "RSD PTR "
    signature:    [u8; 8],

    /// This is the checksum of the fields defined in the ACPI 1.0 specification. This 
    /// includes only the first 20 bytes of this table, bytes 0 to 19, including the 
    /// checksum field. These bytes must sum to zero
    checksum:     u8,
    
    /// An OEM-supplied string that identifies the OEM
    oem_id:       [u8; 6],

    /// The revision of this structure. Larger revision numbers are backward compatible 
    /// to lower revision numbers. The ACPI version 1.0 revision number of this table is 
    /// zero. The ACPI version 1.0 RSDP Structure only includes the first 20 bytes of 
    /// this table, bytes 0 to 19. It does not include the Length field and beyond. The 
    /// current value for this field is 2
    revision:     u8,

    /// 32 bit physical address of the RSDT.
    rsdt_address: u32,

    /// The length of the table, in bytes, including the header, starting from offset 0. 
    /// This field is used to record the size of the entire table. This field is not 
    /// available in the ACPI version 1.0 RSDP Structure.
    length:       u32,

    /// 64 bit physical address of the RSDT.
    xsdt_address: u64,
    
    /// This is a checksum of the entire table, including both checksum fields
    ext_checksum: u32,

    // /// Reserved field
    // reserved:     [u8; 3]
}

impl Rsdp {
    /// Get an [`Rsdp`] structure from the given [`PhysAddr`]
    pub unsafe fn from_phys_addr(phys_addr: PhysAddr) -> Result<Self> {
        // Read an RSDP struct at the current address
        let rsdp = phys_addr.read_phys::<Rsdp>();

        // Ensure the RSDP signature is correct
        ensure!(&rsdp.signature == b"RSD PTR ", Error::InvalidRsdpSignature);

        // Ensure the revision is high enough for this implementation
        ensure!(rsdp.revision >= 2, Error::InvalidRsdpRevision);

        // Ensure the length in the struct matches our implementation
        ensure!(rsdp.length == size_of::<Rsdp>().try_into().unwrap(), 
            Error::InvalidRsdpLength);

        // Validate the checksum of the RSDP structure
        checksum(phys_addr, size_of::<Rsdp>() as u64)?;

        // Return the checked RSDP
        Ok(rsdp)
    }
}

/// MADT structure before the data. This is parsed to retrieve all of the APIC IDs on the 
/// system.
///
/// Reference: [`Root System Description Pointer (RSDP)`](../../../../../references/ACPI_6_2.pdf#page=200)
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct Madt {
    /// The 32-bit physical address at which each processor can access its local interrupt
    /// controller.
    interrupt_controller_address: u32,

    /// Multiple APIC Flags
    flags: u32,
}

/// Header used for all system description tables. The signature field determines the
/// content of hte system description table
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct DescriptionTable {
    /// The ASCII string representation of the table identifier.
    signature:        [u8; 4],

    /// The length of the table, in bytes, including the header, starting from offset 0. 
    /// This field is used to record the size of the entire table.
    length:           u32,

    /// The revision of the structure corresponding to the signature field for this table. 
    revision:         u8,

    /// The entire table, including the checksum field, must add to zero to be considered 
    /// valid
    checksum:         u8,

    /// An OEM-supplied string that identifies the OEM.
    oem_id:           [u8; 6],

    /// An OEM-supplied string that the OEM uses to identify the particular data table.
    oem_table_id:     [u8; 8],
    
    /// An OEM-supplied revision number. Larger numbers are assumed to be newer revisions.
    oem_revision:     [u8; 4],

    /// Vendor ID of utility that created the table. For tables containing Definition 
    /// Blocks, this is the ID for the ASL Compiler.
    creator_id:       [u8; 4],

    /// Revision of utility that created the table.
    creator_revision: [u8; 4],
}

/// The `const` size of a [`DescriptionTable`]
const DESCRIPTION_TABLE_SIZE: usize = size_of::<DescriptionTable>();

impl DescriptionTable {
    /// Parses and validates a `DescriptionTable` at the given `phys_addr`  and returns 
    /// (`DesscriptionTable`, data start address, data len)
    pub unsafe fn from_phys_addr(phys_addr: PhysAddr) -> Result<(Self, PhysAddr, usize)> {
        // Read the table at the current address
        let table: Self = phys_addr.read_phys::<Self>();

        // Validate the checksum for this description table
        checksum(phys_addr, u64::from(table.length))?;

        // Calculate the start of the data for this table
        let data_start  = phys_addr.offset(DESCRIPTION_TABLE_SIZE as u64);

        // Calculate the length of the data for this table
        let data_len = sub!(table.length, DESCRIPTION_TABLE_SIZE.try_into().unwrap());

        Ok((table, data_start, data_len as usize))
    }

    /// Get the ACPI table signature
    pub fn signature(&self) -> TableSignature {
        TableSignature::from(self.signature)
    }
}

/// Reference: [`Local APIC Flags`](../../../../../references/ACPI_6_2.pdf#page=203)
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
enum LocalApicFlags {
    /// The local APIC is disabled
    Disabled = 0,

    /// The local APIC is enabled
    Enabled  = 1
}

/// Reference: [`Processor Local APIC Structure`](../../../../../references/ACPI_6_2.pdf#page=202)
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct LocalApic {
    /// The OS associates this Local APIC Structure with a processor 
    /// object in the namespace when the _UID child object of the 
    /// processor's device object (or the  ProcessorId listed in the 
    /// Processor declaration operator) evaluates to a numeric value 
    /// that matches the numericvalue in this field.
    acpi_processor_uid: u8,

    /// The processor’s local APIC ID
    apic_id: u8,

    /// Local APIC flags
    flags: LocalApicFlags,
}

impl LocalApic {
    fn enabled(&self) -> bool {
        matches!(self.flags, LocalApicFlags::Enabled)
    }
}

/// Local `x2APIC` Flags
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
enum Localx2ApicFlags {
    /// The local x2apic is disabled
    Disabled = 0,

    /// The local x2apic is enabled
    Enabled  = 1
}

/// Reference: [`Processor Local APIC Structure`](../../../../../references/ACPI_6_2.pdf#page=210)
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct Localx2Apic {
    /// The processor’s local x2APIC ID
    x2apic_id: u32,

    /// Local APIC flags
    flags: Localx2ApicFlags,

    /// OSPM associates the X2APIC Structure with a processor object 
    /// declared in the namespace using the Device statement, when the 
    /// _UID child object of the processor device evaluates to a 
    /// numeric value, by matching the numeric value with this field
    acpi_processor_uid: u32,
}

/// Flags for the GIC CPU Interface
#[derive(Debug, Copy, Clone)]
struct GicCpuInterfaceFlags(u32);
impl GicCpuInterfaceFlags {
    /// If zero, this processor is unusable, and the operating system 
    /// support will not attempt to use it.
    pub fn _enabled(self) -> bool {
        self.0 & 1 == 1
    }
}

/// GIC CPU Interface (GICC) Structure
///
/// Reference: [`GIC CPU Interface (GICC) Structure`](../../../../../references/ACPI_6_2.pdf#page=212)
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
struct GicCpuInterface {
    /// GIC's CPU Interface Number.
    cpu_interface_number: u32,

    /// The OS associates this GICC Structure with a processor device 
    /// object in the namespace when the _UID child object of the 
    /// processor device evaluates to a numeric value that matches 
    /// the numeric value in this field.
    acpi_processor_uid: u32,

    /// GIC Cpu Interface Flags
    flags: GicCpuInterfaceFlags,

    /// Version of the ARM-Processor Parking Protocol implemented. 
    /// See <http://uefi.org/acpi> 
    /// The document  link is listed under: 
    /// "Multiprocessor Startup for ARM Platforms"
    parking_protocol_version: u32,

    /// The GSIV used for Performance Monitoring Interrupts
    performance_interrupt_gsiv: u32,

    /// The 64-bit physical address of the processor’s Parking 
    /// Protocol mailbox
    parked_address: u64,

    /// On GICv1/v2 systems and GICv3/4 systems in GICv2 
    /// compatibility mode, this field holds the 64-bit physical 
    /// address at which theprocessor can access this GIC CPU 
    /// Interface. If provided here, the "Local Interrupt Controller 
    /// Address" field in the MADT must be ignored by the OSPM.
    physical_base_address: u64,

    /// Address of the GIC virtual CPU interface registers. If the 
    /// platform is not presenting a GICv2 with virtualization 
    /// extensions this field can be 0.
    gicv: u64,

    /// Address of the GIC virtual interface control block registers. 
    /// If the platform is not presenting a GICv2 with virtualization 
    /// extensions this field can be 0.
    gich: u64,

    /// GSIV for Virtual GIC maintenance interrupt
    vgic_maintenance_interrupt: u32,

    /// On systems supporting GICv3 and above, this field holds the 
    /// 64-bit physical address of the associated Redistributor.
    gicr_base_address: u64,

    /// This fields follows the MPIDR formatting of ARM architecture. 
    /// If the implements ARMv7 architecure then  the format must be:
    /// ```text
    /// Bits [63:24] Must be zero
    /// Bits [23:16] Aff2 : Match Aff2 of target processor MPIDR
    /// Bits [15:08] Aff1 : Match Aff1 of target processor MPIDR
    /// Bits [07:00] Aff0 : Match Aff0 of target processor MPIDR
    /// ```
    ///
    /// For platforms implementing ARMv8 the format must be:
    /// ```text
    /// Bits [63:40] Must be zero
    /// Bits [39:32] Aff3 : Match Aff3 of target processor MPIDR
    /// Bits [31:24] Must be zero
    /// Bits [23:16] Aff2 : Match Aff2 of target processor MPIDR
    /// Bits [15:08] Aff1 : Match Aff1 of target processor MPIDR
    /// Bits [07:00] Aff0 : Match Aff0 of target processor MPIDR
    /// ```
    mpidr: u64,

    /// Describes the relative power efficiency of the associated 
    /// processor. 
    ///
    /// Lower efficiency class numbers are more efficient than higher 
    /// ones (e.g. efficiency class 0 should be treated as more 
    /// efficient than efficiency class 1). 
    ///
    /// However, absolute values of this number have no meaning: 
    /// 2 isn't necessarily half as efficient as 1
    processor_power_efficiency_class: u8,
}

impl Madt {
    /// Parse a MADT structure at the given `PhysAddr` and return the MADT struct and the 
    /// found APIC ids
    ///
    /// Returns an Error if the `MAX_NUM_CPUS` is exceeded when parsing the APIC IDs
    pub unsafe fn from_phys_addr(phys_addr: PhysAddr, payload_length: usize) 
            -> Result<StackVec::<u32, MAX_NUM_CPUS>> {
        // Parse the MADT data from the given physical address
        let _madt = phys_addr.read_phys::<Madt>();

        // Possible APIC IDs. We use a static array here instead of a dynamic Vec just
        // to avoid needing alloc. 
        let mut apics = StackVec::<u32, MAX_NUM_CPUS>::new();

        // let mut x2apics      = [None; MAX_NUM_CPUS];
        // let mut x2apic_index = 0;

        // Get the address which starts the dynamic data in the MADT
        let mut data_addr = phys_addr.offset(core::mem::size_of::<Madt>() as u64);

        // Calculate the end of the data
        let end_of_data = phys_addr.offset(payload_length as u64);

        // Iterate through all of the interrupt controllers in the MADT
        while data_addr.0 < end_of_data.0 {
            // First extract the type, length of the next controller so we know what to parse
            let type_  = data_addr.offset(0).read_u8();
            let length = data_addr.offset(1).read_u8();

            // print!("[MADT][Type: {}] : ", type_);

            match (type_, length) {
                (0, 8) => {
                    let read_addr = data_addr.offset(2);

                    let apic = read_addr.read_phys::<LocalApic>();

                    // If APIC is enabled, push their ID
                    // if apic.flags == LocalApicFlags::Enabled {
                    if apic.enabled() {
                        apics.push(u32::from(apic.apic_id))?;
                    }
                }
                /*
                (1, 12) => {
                    // print!("I/O APIC\n");
                }
                (2, 10) => {
                    // print!("Interrupt Source Override Structure\n");
                }
                (3, 8) => {
                    // print!("Non-Maskable Interrupt Source\n");
                }
                (4, 6) => {
                    // print!("Local APIC NMI\n");
                }
                (5, 12) => {
                    // print!("Local APIC Address Override\n");
                }
                (6, 16) => {
                    // print!("I/O SAPIC\n");
                }
                (7, _) => {
                    // print!("Local SAPIC\n");
                }
                (8, 16) => {
                    // print!("Platform Interrupt Source\n");
                }
                */
                (9, 16) => {
                    let read_addr = data_addr.offset(2);
                    let _x2apic = read_addr.read_phys::<Localx2Apic>();
                }
                (0xa, 12) => {
                    // print!("Local x2APIC NMI\n");
                }
                (0xb, 80) => {

                    print!("GIC CPU Interface\n");

                    // Skip over the reserved field
                    let read_addr = data_addr.offset(2);

                    let gicc = read_addr.read_phys::<GicCpuInterface>();
                    print!("{:x?}\n", gicc);
                }
                (0xc, 24) => {
                    /// GIC Version from the GIC Distributor
                    #[derive(Debug, Copy, Clone)]
                    #[allow(dead_code)]
                    enum GicVersion {
                        /// No GIC version specified. Fall back to hardware discovery for
                        /// GIC version
                        Unknown = 0,

                        /// GIC v1
                        Gicv1   = 1,

                        /// GIC v2
                        Gicv2   = 2,

                        /// GIC v3
                        Gicv3   = 3,

                        /// GIC v4
                        Gicv4   = 4,

                        /// Reserved for future use
                        Reserved
                    }

                    /// GIC Distributor (GICD) Structure structure data
                    #[derive(Debug, Copy, Clone)]
                    #[repr(C, packed)]
                    struct GicDistributor {
                        /// This GIC Distributor’s hardware ID
                        gic_id: u32,

                        /// The 64-bit physical address for this Distributor
                        physical_base_address: u64,

                        /// The global system interrupt number where this GIC 
                        /// Distributor’s interrupt inputs start.
                        ///
                        /// For a given GSIV, GIC INT ID = GSIV - System Vector Base
                        system_vector_base: u32,

                        /// GIC Version
                        gic_version: GicVersion,

                        /// Reserved field
                        reserved: [u8; 3]
                    }

                    print!("GIC Distributor Structure\n");

                    // Skip over the reserved field
                    let read_addr = data_addr.offset(2);

                    let gicd = read_addr.read_phys::<GicDistributor>();
                    print!("{:x?}\n", gicd);

                }
                (0xd, 24) => {
                    /// Each `GICv2m` MSI frame consists of a 4k page which includes 
                    /// registers to generate message signaled interrupts to an 
                    /// associated GIC distributor.
                    ///
                    /// Reference: [`GIC MSI Frame Structure`](../../../../../references/ACPI_6_2.pdf#page=215)
                    #[derive(Debug, Clone, Copy)]
                    #[repr(C, packed)]
                    struct GicMsiFrame {
                        /// GIC MSI Frame ID. In asystem with multiple GIC MSI frames, 
                        /// this value must be unique to each one.
                        gic_msi_frame_id: u32,

                        /// The 64-bit physical address for this MSI Frame
                        physical_base_address: u64,

                        /// GIC MSI Frame Flags
                        flags: u32,

                        /// SPI Count used by this frame. Unless the SPI Count Select flag 
                        /// is set to 1 this value should match the lower 16 bits of the 
                        /// `MSI_TYPER` register in the frame
                        spi_count: u16,

                        /// SPI Base used by this frame. Unless the SPI Base Select flag 
                        /// is set to 1 this value should match the upper 16 bits of the 
                        /// `MSI_TYPER` register in the frame
                        spi_base: u16,
                    }

                    print!("GIC MSI Frame Structure\n");

                    // Skip over the reserved field
                    let read_addr = data_addr.offset(2);

                    let gic_msi_frame = read_addr.read_phys::<GicMsiFrame>();
                    print!("{:x?}\n", gic_msi_frame);
                }
                (0xe, 16) => {
                    /// This structure enables the discovery of GIC Redistributor base 
                    /// addresses by providing the Physical Base Address of a page range 
                    /// containing the GIC Redistributors.
                    #[derive(Debug, Copy, Clone)]
                    #[repr(C, packed)]
                    struct GicRedistributor {
                        /// The 64-bit physical address of a page range containing all 
                        /// GIC Redistributors
                        discovery_range_base_address: u64,

                        /// Length of the GIC Redistributor Discovery page range
                        discovery_range_length: u32,
                    }

                    print!("GIC Redistributor Structure\n");

                    // Skip over the reserved field
                    let read_addr = data_addr.offset(2);

                    let dic_redist = read_addr.read_phys::<GicRedistributor>();
                    print!("{:x?}\n", dic_redist);
                }
                _ => { 
                    print!("Unknown: {} {}\n", type_, length); 
                }
            }

            // Advance to the next one
            data_addr = data_addr.offset(u64::from(length));
        }

        Ok(apics)
    }
}

/*
/// SRAT structure used to get the memory map on the system.
///
/// Reference: Table 5-70  Static Resource Affinity Table Format
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct Srat {
    signature:        [u8; 4],
    length:           u32,
    revision:         u8,
    checksum:         u8,
    oem_id:           [u8; 6],
    oem_table_id:     [u8; 8],
    oem_revision:     [u8; 4],
    creator_id:       [u8; 4],
    creator_revision: [u8; 4],
    reserved0:        u32,
    reserved1:        u64,
}


impl Srat {
    /// Parse a MADT structure at the given `PhysAddr` and return the MADT struct and the 
    /// found APIC ids
    pub fn parse(phys_addr: PhysAddr) 
            -> Result<(BTreeMap<u32, u32>, BTreeMap<u32, Vec<(u64, u64)>>)> {
        let srat = memory_manager::read::<Srat>(phys_addr)?;

        let mut data_addr = phys_addr.0 + core::mem::size_of::<Srat>() as u64;

        let mut apic_to_domain = BTreeMap::new();
        let mut domains = BTreeMap::new();

        // Iterate through all of the interrupt controllers in the SRAT
        while data_addr < phys_addr.0 + srat.length as u64 {
            // First extract the type, length of the next controller so we know what to parse
            let type_  = memory_manager::read_u8(PhysAddr(data_addr + 0))?;
            let length = memory_manager::read_u8(PhysAddr(data_addr + 1))?;

            // print!("[Type: {}] ", type_);

            const AFFINITY_ENABLED: u32 = 1 << 0;
            const HOT_PLUGGABLE   : u32 = 1 << 1;
            const NON_VOLATILE    : u32 = 1 << 2;

            match (type_, length) {
                (0, 16) => { 
                    let local_apic: ProcessorLocalApicAffinity = 
                        memory_manager::read(PhysAddr(data_addr + 2))?;


                    if local_apic.flags & AFFINITY_ENABLED > 0 {
                        let proximity_domain = 
                            (local_apic.proximity_domain_high as u32) << 24 |
                            (local_apic.proximity_domain_mid  as u32) << 8  |
                            (local_apic.proximity_domain_high as u32);

                        ensure!(proximity_domain == 0, 
                            "Found a system with more than 1 NUMA domain!");

                        ensure!(apic_to_domain.insert(local_apic.apic_id as u32, 
                            proximity_domain).is_none());
                    }
                }
                (1, 40) => { 
                    let memory_affinity: MemoryAffinity = 
                        memory_manager::read(PhysAddr(data_addr + 2))?;

                    if memory_affinity.flags & AFFINITY_ENABLED > 0 {
                        print!("{:x} ({:x}, {:x})\n", memory_affinity.proximity_domain,
                                memory_affinity.base_address, memory_affinity.length);

                        domains.entry(memory_affinity.proximity_domain)
                            .or_insert(Vec::new())
                            .push((memory_affinity.base_address, memory_affinity.length));
                    }
                }
                (2, 24) => { 
                    let local_x2apic: ProcessorLocalx2ApicAffinity = 
                        memory_manager::read(PhysAddr(data_addr + 2))?;

                    ensure!(local_x2apic.proximity_domain == 0, 
                        "Found a system with more than 1 NUMA domain!");

                    if local_x2apic.flags & AFFINITY_ENABLED > 0 {
                        ensure!(apic_to_domain.insert(local_x2apic.x2apic_id, 
                            local_x2apic.proximity_domain).is_none());
                    }

                }
                _ => { print!("Unknown SRAT: {} {}\n", type_, length); }
            }

            // Advance to the next one
            data_addr += length as u64;
        }

        Ok((apic_to_domain, domains))
    }
}
*/

/*
/// Processor Local APIC/SAPIC Affinity Structure
///
/// Reference: 5.2.16.1
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct ProcessorLocalApicAffinity {
    proximity_domain_low: u8,
    apic_id: u8,
    flags: u32,
    local_sapic_eid: u8,
    proximity_domain_mid: u16,
    proximity_domain_high: u8,
    clock_domain: u32,
}
*/

/*
/// Memory affinity structure
///
/// Reference: 5.2.16.2
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct MemoryAffinity {
    proximity_domain: u32,
    reserved0: u16,
    base_address: u64,
    length: u64,
    reserved1: u32,
    flags: u32,
    reserved2: u64
}
*/

/*
/// Processor Local x2APIC Affinity Structure
///
/// Reference: 5.2.16.3
#[derive(Copy, Clone)]
#[repr(C, packed)]
struct ProcessorLocalx2ApicAffinity {
    reserved0: u16,
    proximity_domain: u32,
    x2apic_id: u32,
    flags: u32,
    clock_domain: u32,
    reserved2: u32
}
*/

/// Get all of the valid APIC IDs
pub unsafe fn init_apics() -> Result<()> {
    let acpi_base = uefi::acpi_base()?;

    // Read an RSDP struct at the current address
    let rsdp = Rsdp::from_phys_addr(PhysAddr(acpi_base as u64))?;

    // Read the XSDT from the address in the RSDP
    // let xsdt = Xsdt::from_phys_addr(PhysAddr(rsdp.xsdt_address))?;
    let xsdt_addr = PhysAddr(rsdp.xsdt_address);
    let (xsdt, data_addr, data_len) = DescriptionTable::from_phys_addr(xsdt_addr)?;

    // Sanity check we received an XSDT table
    ensure!(xsdt.signature() == TableSignature::Xsdt, Error::InvalidXsdtSignature);

    // Sanity check the data is aligned as expected
    ensure!(data_len % size_of::<u64>() == 0, Error::MisalignedData);

    // Get the number of entries in the table
    let num_entries = data_len / size_of::<u64>();

    // Grab each entry
    for index in 0..num_entries {
        // Calculate the offset into the data for the current index
        let curr_offset = mul!(index, size_of::<u64>()) as u64;

        // Get the address of the entry
        let curr_addr = data_addr.offset(curr_offset);
        
        // Read the entry
        let entry = PhysAddr(curr_addr.read_u64());

        let (table, data_addr, data_len) = DescriptionTable::from_phys_addr(entry)?;

        print!("Table!! {:?}\n", table.signature());
        if matches!(table.signature(), TableSignature::Madt)  {
            let apics = Madt::from_phys_addr(data_addr, data_len)?;
            print!("APICS\n{:x?}\n", apics.data());
        }

        if matches!(table.signature(), TableSignature::Spcr)  {
            print!("!!!SPCR!!!\n");
            print!("!!!SPCR!!!\n");
            print!("!!!SPCR!!!\n");
            print!("!!!SPCR!!!\n");
            print!("!!!SPCR!!!\n");
            print!("!!!SPCR!!!\n");
        }

    }

    Ok(())

    /*

    for &entry in xsdt_entries {
        let signature = memory_manager::read_phys::<[u8; 4]>(PhysAddr(entry.into()));
        print!("Sig: {:x?}\n", signature);
    }

    return Ok(());
    */

    /*
    let mut all_apics = Vec::new();

    // Search for the APIC table and ignore all others since we don't care about the
    // other tables at the moment
    for &entry in rsdt_entries {
        let signature = memory_manager::read::<[u8; 4]>(PhysAddr(entry as u64))?;

        match &signature {
            b"APIC" => {
                // We only care about the MADT structure at the moment, which has a
                // signature of APIC
                let (_madt, apics) = Madt::new(PhysAddr(entry as u64))?;
                all_apics = apics;
            }
            b"SRAT" => {
                let (apic_to_domain, domains) = Srat::parse(PhysAddr(entry))?;
            }
            _ => {
                print!("Ignoring APIC signature: {}\n", 
                        core::str::from_utf8(&signature).asdfsadf());
                continue;
            }
        }
    }

    // Set the number of cores found on the system
    NUM_CORES.store(all_apics.len() as u32, Ordering::SeqCst);

    // Initialize all found APICs
    for &apic_id in all_apics.iter() {
        if apic_id == corelocals!().apic_id {
            // No need to re-init this core
            continue;
        }

        // APIC reference: 10.6.1
        // Send INIT-SIPI-SIPI to this found APIC ID
        corelocals!().apic.lock().init_sipi_sipi_id(apic_id)?;

        // Core is ready, mark that it is ready in the global status
        while APIC_STATES[apic_id as usize].load(Ordering::SeqCst) != ApicState::Online as u8 {
            spin_loop();
        }
    }

    // Save the APIC ids in the global 
    let mut apic_ids = APIC_IDS.lock();
    *apic_ids = Some(all_apics);

    return Ok(());

    return Err(err!("RSDP not found"));
    */
}
