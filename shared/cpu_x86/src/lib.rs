//! Specific x86_64 architecture functionality

#![no_std]
#![feature(asm)]
#![cfg(target_arch="x86_64")]

use core::convert::TryInto;
pub use cpu_trait::CpuTrait;

/// Struct to impl [`CpuTrait`] on
pub struct X86Cpu;

impl CpuTrait for X86Cpu {
    /// Read the page table address from `cr3`
    fn read_page_table_addr() -> u64 {
        let res: usize;
        unsafe {
            asm!("mov {}, cr3", out(reg) res);
        }
        res.try_into().expect("Only valid on 64 bit x86 architectures")
    }

    /// Set the given `addr` to be the active page table
    fn set_page_table_addr(addr: u64) {
        unsafe {
            asm!("mov cr3, {}", in(reg) addr);
        }
    }

    /// Read the current time counter via `rdtsc`
    fn read_time_counter() -> u64 {
        unsafe { core::arch::x86_64::_rdtsc() }
    }
}

/// x86 CPU feature identifiers
///
/// Reference: [`Intel CPUID`](../../../references/Intel_cpuid.pdf)
#[derive(Clone, Copy)]
#[repr(u64)]
pub enum Feature {
    /// FPU: The processor contains an FPU that supports the Intel387 floating-point
    /// instruction set.
    FloatingPointUnit = 1 << 0,

    /// VME: The processor supports extensions to virtual-8086 mode.
    VirtualModeExtension = 1 << 1,

    /// DE: The processor supports I/O breakpoints, including the CR4.DE bit for enabling
    /// debug extensions and optional trapping of access to the DR4 and DR5 registers.
    DebugingExtension = 1 << 2,

    /// PSE: The processor supports 4-Mbyte pages.
    PageSizeExtension = 1 << 3,

    /// TSC: The RDTSC instruction is supported including the CR4.TSD bit for
    /// access/privilege control.
    TimestampCounter = 1 << 4,

    /// MSR: Model Specific Registers are implemented with the RDMSR, WRMSR instructions 
    ModelSpecificRegisters = 1 << 5,
    
    /// PAE: Physical addresses greater than 32 bits are supported.
    PhysicalAddressExtension = 1 << 6,

    /// MCE: Machine Check Exception, Exception 18, and the CR4.MCE enable bit are
    /// supported
    MachineCheckException = 1 << 7,

    /// CX8: The compare and exchange 8 bytes instruction is supported.
    CMPXCHG8B = 1 << 8,

    /// APIC: The processor contains a software-accessible Local APIC.
    APIC = 1 << 9,

    /// SEP: Indicates whether the processor supports the Fast System Call instructions,
    /// `SYSENTER` and `SYSEXIT`. NOTE: Refer to Section 3.4 for further information
    /// regarding `SYSENTER`/`SYSEXIT` feature and `SEP` feature bit.
    SysenterSysexit = 1 << 11,
    
    /// MTRR: The Processor supports the Memory Type Range Registers specifically the
    /// `MTRR_CAP` register
    MemoryTypeRangeRegisters = 1 << 12,

    /// PGE: The global bit in the page directory entries (PDEs) and page table entries
    /// (PTEs) is supported, indicating TLB entries that are common to different
    /// processes and need not be flushed. The CR4.PGE bit controls this feature.
    PageGlobalEnable = 1 << 13,

    /// MCA: The Machine Check Architecture is supported, specifically the `MCG_CAP`
    /// register.
    MachineCheckArchitecture = 1 << 14,

    /// CMOV: The processor supports `CMOVcc`, and if the FPU feature flag (bit 0) is also
    /// set, supports the `FCMOVCC` and `FCOMI` instructions.
    ConditionalMove = 1 << 15,

    /// PAT: Indicates whether the processor supports the Page Attribute Table. This
    /// feature augments the Memory Type Range Registers (MTRRs), allowing an operating
    /// system to specify attributes of memory on 4K granularity through a linear
    /// address.
    PageAttributeTable         = 1 << 16,

    /// PSE-36: Indicates whether the processor supports 4-Mbyte pages that are capable
    /// of addressing physical memory beyond 4GB. This feature indicates that the upper
    /// four bits of the physical address of the 4-Mbyte page is encoded by bits 13-16 of
    /// the page directory entry.
    PageAddressExtension36bit  = 1 << 17,

    /// PSN: The processor supports the 96-bit processor serial number feature, and the
    /// feature is enabled.
    ProcessorSerialNumber = 1 << 18,

    /// CLFSH: Indicates that the processor supports the CLFLUSH instruction.
    CLFLUSH = 1 << 19,

    /// DS: Indicates that the processor has the ability to write a history of the branch
    /// to and from addresses into a memory buffer.
    DebugStore = 1 << 21,

    /// ACPI: The processor implements internal MSRs that allow processor temperature to
    /// be monitored and processor performance to be modulated in predefined duty cycles
    /// under software control.
    ACPI = 1 << 22,

    /// MMX: The processor supports the MMX technology instruction set extensions to Intel
    /// Architecture
    MMX = 1 << 23,

    /// FXSR: Indicates whether the processor supports the FXSAVE and FXRSTOR instructions for
    /// fast save and restore of the floating point context. Presence of this bit also
    /// indicates that CR4.OSFXSR is available for an operating system to indicate that
    /// it uses the fast save/restore instructions.
    FXSAVE = 1 << 24,

    /// SSE: The processor supports the Streaming SIMD Extensions to the Intel Architecture.
    SSE = 1 << 25,

    /// SSE2: Indicates the processor supports the Streaming SIMD Extensions - 2 Instructions.
    SSE2 = 1 << 26,

    /// SS: The processor supports the management of conflicting memory types by
    /// performing a snoop of its own cache structure for transactions issued to the bus.
    SelfSnoop = 1 << 27,

    /// This processor’s microarchitecture has the capability to operate as multiple
    /// logical processors within the same physical package.
    ///
    /// This field does not indicate that Hyper-Threading Technology has been enabled for
    /// this specific processor. To determine if Hyper-Threading Technology is supported,
    /// check the value returned in EBX\[23:16\] after executing CPUID with EAX=1.  If
    /// EBX\[23:16\] contains a value >1, then the processor supports Hyper-Threading
    /// Technology.  
    HyperThreading = 1 << 28,

    /// TM: The processor implements the Thermal Monitor automatic thermal control
    /// circuit (TCC).
    ThermalMonitor = 1 << 29,

    /// From OSDEV Wiki
    IA64 = 1 << 30,

    /// The processor supports the use of the FERR#/PBE# pin when the processor is in the
    /// stop-clock state (STPCLK# is asserted) to signal the processor that an interrupt
    /// is pending and that the processor should return to normal operation to handle the
    /// interrupt. Bit 10 (PBE enable) in the IA32_MISC_ENABLE MSR enables this
    /// capability.
    PendingBreakEnable = 1 << 31,

    /// SSE3: The processor supports the Streaming SIMD Extensions 3 instructions.
    SSE3 = 1 << (32 + 0),

    /// PCLMULDQ: The processor supports PCLMULDQ instruction.
    PCLMULDQ = 1 << (32 + 1),

    /// DTES64: Indicates that the processor has the ability to write a history of the
    /// 64-bit branch to and from addresses into a memory buffer.
    DebugStore64 = 1 << (32 + 2),

    /// MONITOR: The processor supports the MONITOR and MWAIT instructions.
    MONITOR = 1 << (32 + 3),

    /// DS-CPL: The processor supports the extensions to the Debug Store feature to allow
    /// for branch message storage qualified by CPL (privilege level).
    CplQualifiedDebugStore = 1 << (32 + 4),

    /// VMX: The processor supports Intel® Virtualization Technology
    VirtualizationTechnology = 1 << (32 + 5),

    /// SMX: The processor supports Intel® Trusted Execution Technology
    SaferModeExceptions = 1 << (32 + 6),

    /// EST: The processor supports Enhanced Intel SpeedStep Technology and implements
    /// the `IA32_PERF_STS` and `IA32_PERF_CTL` registers.
    EnhancedSpeedStep = 1 << (32 + 7),

    /// TM2: The processor implements the Thermal Monitor 2 thermal control circuit (TCC).
    ThermalMonitor2 = 1 << (32 + 8),

    /// SSSE3: The processor supports the Supplemental Streaming SIMD Extensions 3
    /// instructions.
    SSSE3 = 1 << (32 + 9),

    /// CNXT-ID: The L1 data cache mode can be set to either adaptive mode or shared mode
    /// by the BIOS.
    ContextId = 1 << (32 + 10),

    /// CX16: This processor supports the CMPXCHG16B instruction.
    CMPXCHG16B = 1 << (32 + 13),

    /// xTPR: The processor supports the ability to disable sending Task Priority
    /// messages.  When this feature flag is set, Task Priority messages may be disabled.
    /// Bit 23 (Echo TPR disable) in the `IA32_MISC_ENABLE` MSR controls the sending of
    /// Task Priority messages.
    XTPRUpdateControl = 1 << (32 + 14),

    /// PDCM: The processor supports the Performance Capabilities MSR.
    /// `IA32_PERF_CAPABILITIES` register is MSR `0x345`
    PerfMonDebug = 1 << (32 + 15),

    /// DCA: The processor supports the ability to prefetch data from a memory mapped
    /// device.
    DirectCacheAccess = 1 << (32 + 18),

    /// SSE4.1: The processor supports the Streaming SIMD Extensions 4.1 instructions.
    SSE41 = 1 << (32 + 19),

    /// SSE4.2: The processor supports the Streaming SIMD Extensions 4.2 instructions.
    SSE42 = 1 << (32 + 20),

    /// x2APIC: The processor supports x2APIC feature.
    X2Apic = 1 << (32 + 21),

    /// MOVBE: The processor supports MOVBE instruction (endian swap).
    ///
    /// MOVBE — Move Data After Swapping Bytes
    MOVBE = 1 << (32 + 12),

    /// POPCNT: The processor supports the POPCNTinstruction.
    ///
    /// POPCNT — Return the Count of Number of Bits Set to 1
    POPCNT = 1 << (32 + 23),

    /// AES: The processor supports AES instruction.
    AES = 1 << (32 + 25),

    /// XSAVE: The processor supports the `XSAVE`/`XRSTOR` processor extended states
    /// feature, the XSETBV/XGETBV instructions, and the `XFEATURE_ENABLED_MASK` register
    /// (XCR0 XSAVE = 1 << (32 + 26),
    OSXSAVE = 1 << (32 + 27),

    /// A value of 1 indicates that the OS has enabled `XSETBV`/`XGETBV` instructions to
    /// access the `XFEATURE_ENABLED_MASK` register (XCR0), and support for processor
    /// extended state management using `XSAVE`/`XRSTOR`.
    AdvancedVectorExtensions = 1 << (32 + 28),
}

/// Software IO port mappings
#[repr(u16)]
pub enum IoPort {
    /// Software port mapped from BIOS POST for the Primary PIC Interrupt Mask Register
    PrimaryPicInterruptMask   = 0x21,

    /// Software port mapped from BIOS POST for the Secondary PIC Interrupt Mask Register
    SecondaryPicInterruptMask = 0xa1,
}

impl X86Cpu {
    /// Reads from the given `cpuid` and returns the output of (ecx, edx)
    #[inline]
    pub fn cpuid(leaf: u32) -> u64 {
        let out_ecx: u32;
        let out_edx: u32;
        unsafe {
            asm!("cpuid", 
                in("eax") leaf, 
                out("ecx") out_ecx, 
                out("edx") out_edx);
        }

        (out_ecx as u64) << 32 | out_edx as u64
    }

    /// Returns the feature information (cpuid(1))
    pub fn feature_information() -> u64 {
        Self::cpuid(1)
    }

    /// Returns `true` if the processor has the given [`Feature`]
    #[inline]
    pub fn has_feature(feature: Feature) -> bool {
        Self::feature_information() & (feature as u64) > 0
    }

    /// Reads from the given [`Msr`]
    ///
    /// Example:
    ///
    /// ```rust
    /// let mut apic_base_msr = rdmsr(Msr::ApicBase);
    /// ```
    #[inline]
    pub fn rdmsr(msr: Msr) -> u64 {
        if matches!(msr.permissions(), Permission::WriteOnly) {
            panic!("Attempted to read to write-only Msr: {:?}", msr);
        }

        // Read the value via `rdmsr`
        let mut hi:  u32;
        let mut low: u32;
        unsafe { 
            asm!("rdmsr",
                in("ecx") msr as u32,
                out("edx") hi, 
                out("eax") low,
                options(nomem, nostack));
        }

        (hi as u64) << 32 | low as u64
    }

    /// Reads from the given [`Msr`] and return the result as a `u32`
    ///
    /// Example:
    ///
    /// ```rust
    /// let mut apic_base_msr = rdmsr_u32(Msr::ApicBase);
    /// ```
    #[inline]
    pub fn rdmsr_u32(msr: Msr) -> u32 {
        if matches!(msr.permissions(), Permission::WriteOnly) {
            panic!("Attempted to read to write-only Msr: {:?}", msr);
        }

        // Read the value via `rdmsr`
        let mut low: u32;
        unsafe { 
            asm!("rdmsr",
                in("ecx") msr as u32,
                out("eax") low,
                options(nomem, nostack));
        }

        low
    }

    /// Writes a given `val` to the given [`Msr`]
    #[inline]
    pub fn wrmsr(msr: Msr, val: u64) {
        // Ensure we can write to the given Msr
        if matches!(msr.permissions(), Permission::ReadOnly) {
            panic!("Attempted to write to read-only Msr: {:?}", msr);
        }
       
        // Write the value via `wrmsr`
        unsafe { 
            asm!("wrmsr",
                in("ecx") msr as u32,
                in("edx") (val >> 32) as u32,
                in("eax") val as u32,
                options(nomem, nostack));
        }
    }

    /// Read the APIC base
    #[inline]
    pub fn read_apic_base() -> u64 {
        Self::rdmsr(Msr::ApicBase) 
    }

    /// Write a new the APIC base with the given `val`
    #[inline]
    pub fn write_apic_base(val: u64) {
        Self::wrmsr(Msr::ApicBase, val) 
    }

    /// `out` command on the given IO port `addr` with the given u8 `val`
    #[inline]
    pub unsafe fn out8(addr: IoPort, val: u8) {
        asm!("out dx, al", 
            in("dx") addr as u16, 
            in("al") val,
            options(nomem, nostack),
            );
    }

    /// Read a `u8` from the IO port `addr`
    #[inline]
    pub unsafe fn in8(addr: IoPort) -> u8 {
        let res: u8;
        asm!("in al, dx", 
            in("dx") addr as u16, 
            out("al") res,
            options(nomem, nostack),
        );
        res
    }

    /// Disable the PIC interrupts by masking off all interrupts for both Primary PIC and
    /// Secondary PIC
    ///
    /// Refernece: [`59A Software Port Mappings`](../../../references/pic_tutorial.html)
    #[inline]
    pub fn disable_pic_interrupts() {
        unsafe { 
            Self::out8(IoPort::PrimaryPicInterruptMask,   0xff);
            Self::out8(IoPort::SecondaryPicInterruptMask, 0xff);
        }
    }

    /// Get the primary PIC's interrupt mask
    #[inline]
    pub fn get_primary_pic_interrupt_mask() -> u8 {
        unsafe { 
            Self::in8(IoPort::PrimaryPicInterruptMask)
        }
    }

    /// Set the primary PIC's interrupt mask
    #[inline]
    pub unsafe fn set_primary_pic_interrupt_mask(val: u8) {
        Self::out8(IoPort::PrimaryPicInterruptMask, val)
    }
    
    /// Get the secondary PIC's interrupt mask
    #[inline]
    pub fn get_secondary_pic_interrupt_mask() -> u8 {
        unsafe { 
            Self::in8(IoPort::SecondaryPicInterruptMask)
        }
    }

    /// Set the secondary PIC's interrupt mask
    #[inline]
    pub unsafe fn set_secondary_pic_interrupt_mask(val: u8) {
        Self::out8(IoPort::SecondaryPicInterruptMask, val)
    }

    /// Enable interrupts via `sti`
    #[inline]
    pub unsafe fn enable_interrupts() {
        asm!("sti", options(nomem, nostack));
    }

    /// Disable interrupts via `cli`
    #[inline]
    pub unsafe fn disable_interrupts() {
        asm!("cli", options(nomem, nostack));
    }
}

/// Various MSRs available in an x86 system
///
/// Reference: [`Intel MSRs Manual`](../../../../../references/Intel_manual_Vol4_MSRs.pdf)
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u32)]
pub enum Msr {
    /// This register holds the APIC base address, permitting the relocation of the APIC
    /// memory map. (R/W)
    ///
    /// Bit fields:
    /// 7:0                 - Reserved
    /// 8                   - BSP flag (R/W)
    /// 9                   - Reserved
    /// 10                  - Enable x2APIC mode
    /// 11                  - APIC Global Enable (R/W)
    /// (MAXPHYADDR - 1):12 - APIC Base (R/W)
    /// 63:MAXPHYADDR       - Reserved
    ///
    /// Reference: [`IA32_APIC_BASE`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=20)
    ApicBase = 0x1b,

    /// Control Features in Intel 64 Processor (R/W)
    ///
    /// Bit fields:
    /// 0     - Lock (it (R/WO): (1 = locked). When set,(locks this MSR from being 
    ///         written; writes to this bit will result in GP(0)    
    /// 1     - Enable VMX inside SMX operation (R/WL): This bit enables a system 
    ///         executive to use VMX in conjunction with SMX to support Intel® Trusted 
    ///         Execution Technology.
    /// 2     - Enable VMX outside SMX operation (R/WL): This bit enables VMX for a 
    ///         system
    /// 7:3   - Reserved
    /// 14:8  - SENTER Local Function Enables (R/WL): When set, each bit in the field
    ///         represents an enable control for a corresponding SENTER function. This 
    ///         field is supported only if CPUID.1:ECX.[bit 6] is set.
    /// 15    - SENTER Global Enable (R/WL): This bit must be set to enable SENTER leaf
    ///         functions. This bit is supported only if CPUID.1:ECX.[bit 6] is set
    /// 16    - Reserved 
    /// 17    - SGX Launch Control Enable (R/WL): This bit must be set to enable runtime
    ///         re-configuration of SGX Launch Control via the `IA32_SGXLEPUBKEYHASHn` 
    ///         MSR
    /// 18    - SGX Global Enable (R/WL): This bit must be set to enable SGX leaf 
    ///         functions. 
    /// 19    - Reserved 
    /// 20    - LMCE On (R/WL): When set, system software can program the MSRs associated
    ///         with LMCE to configure delivery of some machine check exceptions to a 
    ///         single logical processor.
    /// 63:31 - Reserved
    ///
    /// Reference: [`IA_32_FEATURE_CONTROL`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=20)
    FeatureControl = 0x3a,

    /// Fixed-Function Performance Counter 0 (R/W)
    ///
    /// This event counts the number of instructions that retire execution. For
    /// instructions that consist of multiple uops, this event counts the retirement of
    /// the last uop of the instruction. The counter continues counting during hardware
    /// interrupts, traps, and in-side interrupt handlers. (R/W)
    ///
    /// Reference: [`INST_RETIRED.ANY`](../../../../../references/Intel_manual_Vol3.pdf#page=685)
    AnyInstructionRetired = 0x309,

    /// Fixed-Function-Counter Control Register (R/W) 
    ///
    /// Counter increments while the results of ANDing respective enable bit in
    /// `IA32_PERF_GLOBAL_CTRL` with the corresponding OS or USR bits in this MSR is true.
    ///
    /// Reference: [`IA32_FIXED_CTR_CTRL`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=40)
    FixedCounterControl = 0x38d,
    

    /// Global Performance Counter Control (R/W)
    ///
    /// Counter increments while the result of ANDing the respective enable bit in this
    /// MSR with the corresponding OS or USR bits in the general-purpose or fixed counter
    /// control MSR is true.
    ///
    /// Reference: [`IA32_PERF_GLOBAL_CTRL`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=42)
    PerfGlobalControl = 0x38f,

    /// Reporting Register of Basic VMX Capabilities (R/O) 
    ///
    /// Reference: [`IA32_VMX_BASIC`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=48)
    VmxBasic = 0x480,
    
    /// Reporting Register of Pin-based VM-execution  Controls (R/O) 
    ///
    /// Reference: [`IA32_VMX_PINBASED_CTLS`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=48)
    VmxPinBasedControls = 0x481,

    /// Reporting Register of Primary Processor-based VM-execution Controls (R/O) 
    ///
    /// Reference: [`IA32_VMX_PROCBASED_CTLS`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxProcBasedControls = 0x482,

    /// Reporting Register of VM-exit Controls (R/O) 
    ///
    /// Reference: [`IA32_VMX_EXIT_CTLS`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxExitControls = 0x483,

    /// Capability Reporting Register of VM-entry Controls (R/O) 
    ///
    /// Reference: [`IA32_VMX_ENTRY_CTLS`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxEntryControls = 0x484,

    /// Reporting Register of Miscellaneous VMX Capabilities (R/O) 
    ///
    /// Reference: [`IA32_VMX_MISC`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxMisc = 0x485,

    /// Capability Reporting Register of CR0 Bits Fixed to 0 (R/O) 
    ///
    /// Reference: [`IA32_VMX_CR0_FIXED0`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxCr0Fixed0 = 0x486,

    /// Capability Reporting Register of CR0 Bits Fixed to 1 (R/O) 
    ///
    /// Reference: [`IA32_VMX_CR0_FIXED1`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxCr0Fixed1 = 0x487,

    /// Capability Reporting Register of CR4 Bits Fixed to 0 (R/O) 
    ///
    /// Reference: [`IA32_VMX_CR4_FIXED0`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxCr4Fixed0 = 0x488,

    /// Capability Reporting Register of CR4 Bits Fixed to 1 (R/O) 
    ///
    /// Reference: [`IA32_VMX_CR4_FIXED1`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxCr4Fixed1 = 0x489,

    /// Capability Reporting Register of Secondary Processor-Based VM-Execution Controls
    /// (R/O)
    ///
    /// Reference: [`IA32_VMX_PROCBASED_CTLS2`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxProcBasedControls2 = 0x48b,

    /// Capability Reporting Register of Pin-Based VM-Execution Flex Controls (R/O)
    ///
    /// Reference: [`IA32_VMX_TRUE_PINBASED_CTLS`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=49)
    VmxTruePinBasedControls = 0x48d,

    /// Capability Reporting Register of Pin-based VM-execution Flex  Controls (R/O) 
    ///
    /// Reference: [`IA32_VMX_TRUE_PROCBASED_CTLS`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=50)
    VmxTrueProcBasedControls = 0x48e,

    /// Capability Reporting Register of Pin-based VM-execution Flex  Controls (R/O) 
    ///
    /// Reference: [`IA32_VMX_TRUE_EXIT_CTLS`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=50)
    VmxTrueExitControls = 0x48f,

    /// Capability Reporting Register of VM-entry Flex Controls (R/O) 
    ///
    /// Reference: [`IA32_VMX_TRUE_ENTRY_CTLS`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=50)
    VmxTrueEntryControls = 0x490,

    /// x2APIC ID register (R/O) 
    ///
    /// Reference: [`IA32_X2APIC_APICID`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicApicid = 0x802,

    /// x2APIC ID register (R/O) 
    ///
    /// Reference: [`IA32_X2APIC_VERSION`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicVersion = 0x803,

    /// x2APIC Task Priority register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_TPR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicTpr = 0x808,

    /// x2APIC Processor Priority register (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TPR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicPpr = 0x80a,

    /// x2APIC EOI Register (W/O)
    ///
    /// Reference: [`IA32_X2APIC_EOI`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicEoi = 0x80b,

    /// x2APIC Logical Destination register (R/O)
    ///
    /// Reference: [`IA32_X2APIC_LDR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicLdr = 0x80d,

    /// x2APIC Spurious Interrupt Vector register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_SIVR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicSivr = 0x80f,

    /// x2APIC In-Service register bits \[31:0\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_ISR0`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicIsr0 = 0x810,

    /// x2APIC In-Service register bits \[63:32\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_ISR1`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicIsr1 = 0x811,

    /// x2APIC In-Service register bits \[95:64\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_ISR2`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicIsr2 = 0x812,

    /// x2APIC In-Service register bits \[127:96\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_ISR3`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicIsr3 = 0x813,

    /// x2APIC In-Service register bits \[159:128\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_ISR4`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicIsr4 = 0x814,

    /// x2APIC In-Service register bits \[191:160\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_ISR5`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicIsr5 = 0x815,

    /// x2APIC In-Service register bits \[223:192\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_ISR6`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicIsr6 = 0x816,

    /// x2APIC In-Service register bits \[255:224\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_ISR7`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicIsr7 = 0x817,

    /// x2APIC Trigger Mode register bits \[31:0\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TMR0`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicTmr0 = 0x818,

    /// x2APIC Trigger Mode register bits \[63:32\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TMR1`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicTmr1 = 0x819,

    /// x2APIC Trigger Mode register bits \[95:64\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TMR2`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicTmr2 = 0x81a,

    /// x2APIC Trigger Mode register bits \[127:96\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TMR3`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicTmr3 = 0x81b,

    /// x2APIC Trigger Mode register bits \[159:128\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TMR4`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=60)
    X2apicTmr4 = 0x81c,

    /// x2APIC Trigger Mode register bits \[191:160\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TMR5`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicTmr5 = 0x81d,

    /// x2APIC Trigger Mode register bits \[223:192\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TMR6`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicTmr6 = 0x81e,

    /// x2APIC Trigger Mode register bits \[255:224\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_TMR7`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicTmr7 = 0x81f,

    /// x2APIC Interrupt Request register bits \[31:0\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_IRR0`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIrr0 = 0x820,

    /// x2APIC Interrupt Request register bits \[63:32\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_IRR1`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIrr1 = 0x821,

    /// x2APIC Interrupt Request register bits \[95:64\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_IRR2`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIrr2 = 0x822,

    /// x2APIC Interrupt Request register bits \[127:96\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_IRR3`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIrr3 = 0x823,

    /// x2APIC Interrupt Request register bits \[159:128\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_IRR4`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIrr4 = 0x824,

    /// x2APIC Interrupt Request register bits \[191:160\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_IRR5`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIrr5 = 0x825,

    /// x2APIC Interrupt Request register bits \[223:192\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_IRR6`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIrr6 = 0x826,

    /// x2APIC Interrupt Request register bits \[255:224\] (R/O)
    ///
    /// Reference: [`IA32_X2APIC_IRR7`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIrr7 = 0x827,

    /// X2APIC Error Status Register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_ESR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicEsr = 0x828,

    /// x2APIC LVT Corrected Machine Check Interrupt register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_LVT_CMCI`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicLvtCmci = 0x82f,

    /// x2APIC Interrupt Command register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_ICR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicIcr = 0x830,

    /// x2APIC LVT Timer Interrupt register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_LVT_TIMER`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=61)
    X2apicLvtTimer = 0x832,

    /// x2APIC LVT Thermal Sensor Interrupt register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_LVT_THERMAL`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicLvtThermal = 0x833,

    /// x2APIC LVT Performance Monitor register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_LVT_PMI`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicLvtPmi = 0x834,

    /// X2APIC LVT LINT0 Register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_LVT_LINT0`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicLvtLint0 = 0x835,

    /// X2APIC LVT LINT1 Register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_LVT_LINT1`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicLvtLint1 = 0x836,

    /// X2APIC LVT Error Register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_LVT_ERROR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicLvtError = 0x837,

    /// x2APIC Initial Count register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_INIT_COUNT`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicInitCount = 0x838,

    /// x2APIC Current Count register (R/O)
    ///
    /// Reference: [`IA32_X2APIC_CUR_COUNT`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicCurCount = 0x839,

    /// x2APIC Divide Configuration register (R/W)
    ///
    /// Reference: [`IA32_X2APIC_DIV_CONF`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicDivConf = 0x83e,

    /// X2APIC Self IPI Register (W/O)
    ///
    /// Reference: [`IA32_X2APIC_SELF_IPI`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=62)
    X2apicSelfIpi = 0x83f,

    /// Extended Feature Enables
    ///
    /// Bit Fields:
    /// 0     - SYSCALL Enable:     IA32_EFER.SCE (R/W) 
    ///           Enables SYSCALL/SYSRET instructions in 64-bit mode
    /// 7:1   - Reserved
    /// 8     - IA-32e Mode Enable: IA32_EFER.LME (R/W) 
    ///           Enables IA-32e mode operation.
    /// 9     - Reserved
    /// 10    - IA-32e Mode Active: IA32_EFER.LMA (R/O) 
    ///           Indicates IA-32e mode is active when set
    /// 11    - Execute Disable Bit Enable: IA32_EFER.NXE (R/W)
    /// 63:12 - Reserved
    ///
    /// Reference: [`IA32_EFER`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    Efer = 0xc000_0080,

    /// System Call Target Address (R/W) 
    ///
    /// Reference: [`IA32_STAR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    Star = 0xc000_0081,

    /// IA-32e Mode System Call Target Address (R/W)  
    ///
    /// Target RIP for the called procedure when SYSCALL is executed in 64-bit mode.
    ///
    /// Reference: [`IA32_LSTAR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    Lstar = 0xc000_0082,

    /// IA-32e Mode System Call Target Address (R/W)
    ///
    /// Not used, as the SYSCALL instruction is not recognized in compatibility mode.
    ///
    /// Reference: [`IA32_CSTAR`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    Cstar = 0xc000_0083,

    /// System Call Flag Mask (R/W)  
    ///
    /// Reference: [`IA32_FMASK`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    SfMask = 0xc000_0084,

    /// Map of BASE Address of FS (R/W)
    ///
    /// Reference: [`IA32_FS_BASE`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    FsBase = 0xc000_0100,

    /// Map of BASE Address of GS (R/W)  
    ///
    /// Reference: [`IA32_GS_BASE`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    GsBase = 0xc000_0101,

    /// Swap Target of BASE Address of GS (R/W)
    ///
    /// Reference: [`IA32_KERNEL_GS_BASE`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    KernelGsBase = 0xc000_0102,

    /// AUXILIARY TSC Signature. (R/W)
    ///
    /// Reference: [`IA32_TSC_AUX`](../../../../../references/Intel_manual_Vol4_MSRs.pdf#page=66)
    TscAux = 0xc000_0103,
}

impl From<u32> for Msr {
    fn from(msr: u32) -> Self {
        match msr {
            0x1b  => Msr::ApicBase,
            0x3a  => Msr::FeatureControl,
            0x309 => Msr::AnyInstructionRetired,
            0x38d => Msr::FixedCounterControl,
            0x38f => Msr::PerfGlobalControl,
            0x480 => Msr::VmxBasic,
            0x481 => Msr::VmxPinBasedControls,
            0x482 => Msr::VmxProcBasedControls,
            0x483 => Msr::VmxExitControls,
            0x484 => Msr::VmxEntryControls,
            0x485 => Msr::VmxMisc,
            0x486 => Msr::VmxCr0Fixed0,
            0x487 => Msr::VmxCr0Fixed1,
            0x488 => Msr::VmxCr4Fixed0,
            0x489 => Msr::VmxCr4Fixed1,
            0x48b => Msr::VmxProcBasedControls2,
            0x48d => Msr::VmxTruePinBasedControls,
            0x48e => Msr::VmxTrueProcBasedControls,
            0x48f => Msr::VmxTrueExitControls,
            0x490 => Msr::VmxTrueEntryControls,
            0x802 => Msr::X2apicApicid,
            0x803 => Msr::X2apicVersion,
            0x808 => Msr::X2apicTpr,
            0x80a => Msr::X2apicPpr,
            0x80b => Msr::X2apicEoi,
            0x80d => Msr::X2apicLdr,
            0x80f => Msr::X2apicSivr,
            0x810 => Msr::X2apicIsr0,
            0x811 => Msr::X2apicIsr1,
            0x812 => Msr::X2apicIsr2,
            0x813 => Msr::X2apicIsr3,
            0x814 => Msr::X2apicIsr4,
            0x815 => Msr::X2apicIsr5,
            0x816 => Msr::X2apicIsr6,
            0x817 => Msr::X2apicIsr7,
            0x818 => Msr::X2apicTmr0,
            0x819 => Msr::X2apicTmr1,
            0x81a => Msr::X2apicTmr2,
            0x81b => Msr::X2apicTmr3,
            0x81c => Msr::X2apicTmr4,
            0x81d => Msr::X2apicTmr5,
            0x81e => Msr::X2apicTmr6,
            0x81f => Msr::X2apicTmr7,
            0x820 => Msr::X2apicIrr0,
            0x821 => Msr::X2apicIrr1,
            0x822 => Msr::X2apicIrr2,
            0x823 => Msr::X2apicIrr3,
            0x824 => Msr::X2apicIrr4,
            0x825 => Msr::X2apicIrr5,
            0x826 => Msr::X2apicIrr6,
            0x827 => Msr::X2apicIrr7,
            0x828 => Msr::X2apicEsr,
            0x82f => Msr::X2apicLvtCmci,
            0x830 => Msr::X2apicIcr,
            0x832 => Msr::X2apicLvtTimer,
            0x833 => Msr::X2apicLvtThermal,
            0x834 => Msr::X2apicLvtPmi,
            0x835 => Msr::X2apicLvtLint0,
            0x836 => Msr::X2apicLvtLint1,
            0x837 => Msr::X2apicLvtError,
            0x838 => Msr::X2apicInitCount,
            0x839 => Msr::X2apicCurCount,
            0x83e => Msr::X2apicDivConf,
            0x83f => Msr::X2apicSelfIpi,
            0xc000_0080 => Msr::Efer,
            0xc000_0081 => Msr::Star,
            0xc000_0082 => Msr::Lstar,
            0xc000_0083 => Msr::Cstar,
            0xc000_0084 => Msr::SfMask,
            0xc000_0100 => Msr::FsBase,
            0xc000_0101 => Msr::GsBase,
            0xc000_0102 => Msr::KernelGsBase,
            0xc000_0103 => Msr::TscAux,
            _ => unimplemented!()
        }
    }
}

/// Read and/or Write permissions for a given [`Msr`]
pub enum Permission {
    ReadOnly,
    ReadWrite,
    WriteOnly
}

impl Msr {
    /// Get the read/write permissions for the current [`Msr`]
    #[inline]
    pub fn permissions(&self) -> Permission {
        match self {
            Msr::VmxBasic | Msr::VmxPinBasedControls | Msr::VmxProcBasedControls |
            Msr::VmxExitControls | Msr::VmxEntryControls | Msr::VmxMisc |
            Msr::VmxCr0Fixed0 | Msr::VmxCr0Fixed1 | Msr::VmxCr4Fixed0 |
            Msr::VmxCr4Fixed1 | Msr::VmxProcBasedControls2 | 
            Msr::VmxTruePinBasedControls | Msr::VmxTrueProcBasedControls | 
            Msr::VmxTrueExitControls | Msr::VmxTrueEntryControls |
            Msr::X2apicApicid | Msr::X2apicVersion | Msr::X2apicPpr | Msr::X2apicLdr |
            Msr::X2apicIsr0 | Msr::X2apicIsr1 | Msr::X2apicIsr2 | Msr::X2apicIsr3 |
            Msr::X2apicIsr4 | Msr::X2apicIsr5 | Msr::X2apicIsr6 | Msr::X2apicIsr7 |
            Msr::X2apicTmr0 | Msr::X2apicTmr1 | Msr::X2apicTmr2 | Msr::X2apicTmr3 |
            Msr::X2apicTmr4 | Msr::X2apicTmr5 | Msr::X2apicTmr6 | Msr::X2apicTmr7 |
            Msr::X2apicIrr0 | Msr::X2apicIrr1 | Msr::X2apicIrr2 | Msr::X2apicIrr3 |
            Msr::X2apicIrr4 | Msr::X2apicIrr5 | Msr::X2apicIrr6 | Msr::X2apicIrr7 |
            Msr::X2apicCurCount 
                => Permission::ReadOnly,

            Msr::ApicBase | Msr::FeatureControl | Msr::AnyInstructionRetired |
            Msr::FixedCounterControl | Msr::PerfGlobalControl | Msr::X2apicTpr |
            Msr::X2apicSivr | Msr::X2apicEsr | Msr::X2apicLvtCmci | Msr::X2apicIcr |
            Msr::X2apicLvtTimer | Msr::X2apicLvtThermal | Msr::X2apicLvtPmi |
            Msr::X2apicLvtLint0 | Msr::X2apicLvtLint1 | Msr::X2apicLvtError |
            Msr::X2apicInitCount | Msr::X2apicDivConf | Msr::Star |
            Msr::Lstar | Msr::Cstar | Msr::SfMask | Msr::FsBase |
            Msr::GsBase | Msr::KernelGsBase | Msr::TscAux | Msr::Efer
                => Permission::ReadWrite,

            Msr::X2apicSelfIpi | Msr::X2apicEoi 
                => Permission::WriteOnly,
        }
    }
}
