//! APIC implementation primarily used to program the APIC timer to watch for serial data
//! to know when to soft reboot. Provides the ability to reset the APIC to its original
//! state to allow for soft reboots
//!
//! Reference: [`Advanced Programmable Interrupt Controller (APIC)`](../../../../../../references/Intel_manual_Vol3.pdf#page=377)

use errchain::prelude::*;

#[cfg(target_arch = "x86_64")]
use cpu_x86::{X86Cpu as cpu, Feature, Msr};

/// Number of writable APIC registers to restore on reset
const NUM_WRITABLE_REGS: usize = 11;

/// Various errors that EFI functions can result in
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// Attempted to perform an IPI without APIC_ID set
    IpiWithoutApicId,

    /// Interrupt vector not set when issuing a non-INIT/NMI
    UnsetVector,

    /// APIC ID was set when destination was IncludingSelf
    ApicIdSetWithIncludingSelf,

    /// APIC ID was set when destination was ExcludingSelf
    ApicIdSetWithExcludingSelf,
}

/// Current and original state of the APIC
pub struct Apic {
    /// Current mode the APIC is running
    mode: Mode,

    /// Original state of the APIC/PIC to reset to during soft reboot
    original_state: ResettableState,

    /// Is the local core the BSP
    pub is_bsp: bool,
}

/// Original APIC/PIC state to reset during soft reboot
pub struct ResettableState {
    /// Original value from the `IA32_APIC_BASE` MSR
    apic_base: u64,

    /// Original state of registers that can be written such that we can restore them on 
    /// soft reboot
    registers: [Option<(Register, u32)>; NUM_WRITABLE_REGS],

    /// Original value of the primary PIC interrupt masks used during soft reboot
    primary_pic_interrupt_mask: u8,

    /// Original value of the secondary PIC interrupt masks used during soft reboot
    secondary_pic_interrupt_mask: u8,
}

/// Various registers available in the APIC
///
/// Reference: [`Register Address Space`](../../../../../../references/Intel_manual_Vol3.pdf#page=414)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Register {
    /// Local APIC ID register (R/O)
    Id = 0x20,

    /// Local APIC Version register (R/O)
    ///
    /// Same version used in xAPIC mode and x2APIC mode
    Version = 0x30,

    /// Task Priority Register (TPR) (R/W)
    TaskPriority = 0x80,

    /// 
    ArbitrationPriority = 0x90,

    /// Processor Priority Register (PPR) (R/O)
    ProcessorPriority = 0xa0,

    /// End of Interrupt (W/O)
    ///
    /// WRMSR of a non-zero value causes #GP(0)
    EndOfInterrupt = 0xb0, 

    /// Logical Destination Register (LDR) (R/O in x2apic | R/W in xAPIC)
    LogicalDestination = 0xd0,

    /// Spurious Interruot Vector Register (SVR) (R/W)
    SpuriousInterruptVector = 0xf0,

    /// ISR Bits 31:0 (R/O)
    InterruptInService0= 0x100,

    /// ISR Bits 63:32 (R/O)
    InterruptInService1= 0x110,

    /// ISR Bits 95:64 (R/O)
    InterruptInService2= 0x120,

    /// ISR Bits 127:96 (R/O)
    InterruptInService3= 0x130,

    /// ISR Bits 159:128 (R/O)
    InterruptInService4= 0x140,

    /// ISR Bits 191:160 (R/O)
    InterruptInService5= 0x150,

    /// ISR Bits 223:192 (R/O)
    InterruptInService6= 0x160,

    /// ISR Bits 255:224 (R/O)
    InterruptInService7= 0x170,

    /// Trigger Mode Register bits 31:0 (R/O)
    TriggerMode0 = 0x180,

    /// Trigger Mode Register bits 63:32 (R/O)
    TriggerMode1 = 0x190,

    /// Trigger Mode Register bits 95:64 (R/O)
    TriggerMode2 = 0x1a0,

    /// Trigger Mode Register bits 127:96 (R/O)
    TriggerMode3 = 0x1b0,

    /// Trigger Mode Register bits 159:128 (R/O)
    TriggerMode4 = 0x1c0,

    /// Trigger Mode Register bits 191:160 (R/O)
    TriggerMode5 = 0x1d0,

    /// Trigger Mode Register bits 223:192 (R/O)
    TriggerMode6 = 0x1e0,

    /// Trigger Mode Register bits 255:224 (R/O)
    TriggerMode7 = 0x1f0,

    /// Interrupt Request Register bits 31:0    (R/O)
    InterruptRequest0= 0x200,

    /// Interrupt Request Register bits 63:32   (R/O)
    InterruptRequest1= 0x210,

    /// Interrupt Request Register bits 95:64   (R/O)
    InterruptRequest2= 0x220,

    /// Interrupt Request Register bits 127:96  (R/O)
    InterruptRequest3= 0x230,

    /// Interrupt Request Register bits 159:128 (R/O)
    InterruptRequest4= 0x240,

    /// Interrupt Request Register bits 191:160 (R/O)
    InterruptRequest5= 0x250,

    /// Interrupt Request Register bits 223:192 (R/O)
    InterruptRequest6= 0x260,

    /// Interrupt Request Register bits 255:224 (R/O)
    InterruptRequest7= 0x270,

    /// Error Status Register (ESR) (R/W)
    ///
    /// WRMSR of a non-zero value causes #GP(0)
    ErrorStatus = 0x280,

    /// LVT CMCI Register (R/W)
    LvtCorrectedMachineCheckInterrupt = 0x2f0,

    /// Interrupt Command Register (ICR) (R/W)
    InterruptCommand  = 0x300,

    /// Interrupt Command Register 2 (ICR) (R/W)
    InterruptCommand2 = 0x310,

    /// LVT Timer register (R/W)
    LvtTimer = 0x320,

    /// LVT Thermal Sensor register
    LvtThermalSensor = 0x330,

    /// LVT Performance Monitoring register (R/W)
    LvtPerformanceMonitoring = 0x340,

    /// LVT LINT0 register (R/W)
    LvtLint0 = 0x350,

    /// LVT LINT1 register (R/W)
    LvtLint1 = 0x360,

    /// LVT Error register (R/W)
    LvtError = 0x370,

    /// Initial Count register (for Timer) (R/W)
    TimerInitialCount = 0x380,

    /// Current Count register (for Timer) (R/O)
    TimerCurrentCount = 0x390,

    /// Divide Configuration Register (DCR; for Timer) (R/W)
    TimerDivideConfiguration = 0x3e0,
}

impl Register {

}

/// The interrupt command register (ICR) is a 64-bit local APIC register (see Figure
/// 10-12) that allows software running on the processor to specify and send
/// interprocessor interrupts (IPIs) to other processors in the system.
///
/// Reference: [`Interrupt Command Register (ICR)`](../../../../../../references/Intel_manual_Vol3.pdf#page=395)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct InterruptCommand {
    /// Specifies the type of IPI to be sent. This field is also know as the IPI message
    /// type field.
    delivery_mode: DeliveryMode,
    
    /// Selects either physical (0) or logical (1) destination mode.
    destination_mode: DestinationMode,

    /// Indicates the IPI delivery status
    delivery_status: DeliveryStatus,

    /// For the INIT level de-assert delivery mode this flag must be set to 0; for all
    /// other delivery modes it must be set to 1.
    level: Level,

    /// Selects the trigger mode when using the INIT level de-assert delivery mode.
    /// Ignored for all other delivery modes.
    trigger_mode: TriggerMode,

    /// Indicates whether a shorthand notation is used to specify the destination of the
    /// interrupt and, if so, which shorthand is used.
    destination_shorthand: DestinationShorthand,

    /// The vector number of the interrupt being sent.
    vector: Option<u32>,

    /// The APIC id to send this request to.
    apic_id: Option<u32>
}

impl Default for InterruptCommand {
    fn default() -> Self {
        InterruptCommand {
            delivery_mode:         DeliveryMode::Fixed,
            destination_mode:      DestinationMode::Physical,
            delivery_status:       DeliveryStatus::Idle,
            level:                 Level::DeAssert,
            trigger_mode:          TriggerMode::Edge,
            destination_shorthand: DestinationShorthand::None,
            vector:                None,
            apic_id:               None
        }
    }
}

impl InterruptCommand {
    /// Setter for `delivery_mode`
    #[allow(unused)]
    pub fn delivery_mode(mut self, val: DeliveryMode) -> Self {
        self.delivery_mode = val;
        self
    }

    /// Setter for `destination_mode`
    #[allow(unused)]
    pub fn destination_mode(mut self, val: DestinationMode) -> Self {
        self.destination_mode = val;
        self
    }

    /// Setter for `delivery_status`
    #[allow(unused)]
    pub fn delivery_status(mut self, val: DeliveryStatus) -> Self {
        self.delivery_status = val;
        self
    }

    /// Setter for `level`
    #[allow(unused)]
    pub fn level(mut self, val: Level) -> Self {
        self.level = val;
        self
    }

    /// Setter for `trigger_mode`
    #[allow(unused)]
    pub fn trigger_mode(mut self, val: TriggerMode) -> Self {
        self.trigger_mode = val;
        self
    }

    /// Setter for `destination_shorthand`
    #[allow(unused)]
    pub fn destination_shorthand(mut self, val: DestinationShorthand) -> Self {
        self.destination_shorthand = val;
        self
    }

    /// Setter for `vector`
    #[allow(unused)]
    pub fn vector(mut self, val: u32) -> Self {
        self.vector = Some(val);
        self
    }

    /// Setter for `apic_id`
    #[allow(unused)]
    pub fn apic_id(mut self, val: u32) -> Self {
        self.apic_id = Some(val);
        self
    }

    /// Convert the structure into its u64 representation
    ///
    /// # Panics
    ///
    /// Improperly formatted [`InterruptCommand`]: 
    ///     * Vector set with NMI or INIT
    ///     * APIC ID set with `AllIncludingSelf`
    ///     * APIC ID set with `AllExcludingSelf`
    ///
    /// Reference: [`APIC Timer`](../../../../../../references/Intel_manual_Vol3.pdf#page=392)
    pub fn raw(&self, mode: &Mode) -> u64 {
        assert!(self.vector.is_some() 
                || self.delivery_mode == DeliveryMode::NonMaskableInterrupt
                || self.delivery_mode == DeliveryMode::Init,  
                "ApicError: {:?}", Error::UnsetVector);

        assert!(self.destination_shorthand == DestinationShorthand::AllIncludingSelf
                    && self.apic_id.is_none(), 
                    "ApicError: {:?}", Error::ApicIdSetWithIncludingSelf);
        assert!(self.destination_shorthand == DestinationShorthand::AllExcludingSelf
                    && self.apic_id.is_none(), 
                    "ApicError: {:?}", Error::ApicIdSetWithExcludingSelf);

        // Get the current apic id if there is one
        let apic_id = self.apic_id.unwrap_or(0);

        // Get the APIC ID based on the mode of the APIC
        let dest_apic_id = match mode {
            Mode::Apic(_) => {
                // Original APIC has ID in bits 24:27
                assert!(apic_id <= 0xf, "Invalid destination APIC ID");
                apic_id << 24
            }
            Mode::X2Apic => {
                // x2APIC has id in bits 0:31
                apic_id
            }
        };

        // Create the raw interrupt command
        u64::from(dest_apic_id)             << 32 |
        (self.destination_shorthand as u64) << 18 |
        (self.trigger_mode as u64)          << 15 |
        (self.level as u64)                 << 14 |
        (self.delivery_status as u64)       << 12 |
        (self.destination_mode as u64)      << 11 |
        (self.delivery_mode as u64)         << 8  |
        u64::from(self.vector.unwrap_or(0))
    }
}

/// The clock frequency adjustment for the APIC timer
///
/// Reference: [`APIC Timer`](../../../../../../references/Intel_manual_Vol3.pdf#page=392)
#[repr(u32)]
pub enum TimerDivideConfiguration {
    /// Timer frequency = Clock frequency / 2
    DivideBy2   = 0b0000,

    /// Timer frequency = Clock frequency / 4
    DivideBy4   = 0b0001,

    /// Timer frequency = Clock frequency / 8
    DivideBy8   = 0b0010,

    /// Timer frequency = Clock frequency / 16
    DivideBy16  = 0b0011,

    /// Timer frequency = Clock frequency / 32
    DivideBy32  = 0b1000,

    /// Timer frequency = Clock frequency / 64
    DivideBy64  = 0b1001,

    /// Timer frequency = Clock frequency / 128
    DivideBy128 = 0b1010,

    /// Timer frequency = Clock frequency / 1
    DivideBy1   = 0b1011,
}

/// Timer mode for the APIC Timer
///
/// Reference: [`Figure 10-8: Local Vector Table`](../../../../../../references/Intel_manual_Vol3.pdf#page=389)
pub enum TimerMode {
    /// In one-shot mode, the timer is started by programming its initial-count
    /// register. The initial count value is then copied into the current-count register
    /// and count-down begins. After the timer reaches zero, an timer interrupt is
    /// generated and the timer remains at its 0 value until reprogrammed. 
    ///
    /// Reference: [`APIC Timer`](../../../../../../references/Intel_manual_Vol3.pdf#page=393)
    OneShot = 0,

    /// In periodic mode, the current-count register is automatically reloaded from the
    /// initial-count register when the count reaches 0 and a timer interrupt is
    /// generated, and the count-down is repeated. If during the count-down process the
    /// initial-count register is set, counting will restart, using the new initial-count
    /// value. The initial-count register is a read-write register; the current-count
    /// register is read only. 
    ///
    /// Reference: [`APIC Timer`](../../../../../../references/Intel_manual_Vol3.pdf#page=393)
    Periodic = 1,

    /// TSC-deadline mode allows software to use the local APIC timer to signal an
    /// interrupt at an absolute time. In TSC-deadline mode, writes to the initial-count
    /// register are ignored; and current-count register always reads 0. Instead, timer
    /// behavior is controlled using the `IA32_TSC_DEADLINE` MSR
    ///
    /// Reference: [`10.5.4.1: TSC-Deadline Mode`](../../../../../../references/Intel_manual_Vol3.pdf#page=393)
    TscDeadline = 2
}

/// Specifies the type of interrupt to be sent to the processor.
///
/// Reference: [`Delivery Mode`](../../../../../../references/Intel_manual_Vol3.pdf#page=395)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum DeliveryMode {
    /// Delivers the interrupt specified in the vector field to the target processor or
    /// processors
    Fixed = 0b000,

    /// Same as fixed mode, except that the interrupt is delivered to the processor
    /// executing at the lowest priority among the set of processors specified in the
    /// destination field. The ability for a processor to send a lowest priority IPI is
    /// model specific and should be avoided by BIOS and operating system software.
    LowestPriority = 0b001,

    /// Delivers an SMI interrupt to the target processor or processors. The vector field
    /// must be programmed to 00H for future compatibility.
    SystemManagementInterrupt = 0b010,

    /// Reserved field
    Reserved0,

    /// Delivers an NMI interrupt to the target processor or processors. The vector
    /// information is ignored.
    NonMaskableInterrupt = 0b100,

    /// Delivers an `INIT` request to the target processor or processors, which causes them
    /// to perform an `INIT`. As a result of this `IPI` (Inter-processor interrupt) 
    /// message, all the target processors perform an `INIT`. The vector field must be 
    /// programmed to `0x00` for future compatibility.
    Init = 0b101,

    /// Sends a special “start-up” `IPI` (called a SIPI) to the target processor or
    /// processors. The vector typically points to a start-up routine that is part of the
    /// BIOS boot-strap code (see Section 8.4, “Multiple-Processor (MP)
    /// Initialization”). IPIs sent with this delivery mode are not automatically
    /// retried if the source APIC is unable to deliver it. It is up to the software to
    /// determine if the SIPI was not successfully delivered and to reissue the SIPI if
    /// necessary
    StartUp = 0b110,

    /// Reserved field
    Reserved1
}

/// Destination mode options for an interrupt command
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum DestinationMode {
    /// In physical destination mode, the destination processor is specified by its local
    /// APIC ID (see Section 10.4.6, “Local APIC ID”). For Pentium 4 and Intel Xeon
    /// processors, either a single destination (local APIC IDs 00H through FEH) or a
    /// broadcast to all APICs (the APIC ID is FFH) may be specified in physical
    /// destination mode.
    ///
    /// Reference: [`Physical Destiantion Mode`](../../../../../../references/Intel_manual_Vol3.pdf#page=399)
    Physical = 0,

    /// In logical destination mode, IPI destination is specified using an 8-bit message
    /// destination address (MDA), which is entered in the destination field of the ICR.
    ///
    /// Reference: [`Logical Destiantion Mode`](../../../../../../references/Intel_manual_Vol3.pdf#page=399)
    Logical
}

/// Delivery status options for an interrupt command
///
/// Reference: [`Delivery Status`](../../../../../../references/Intel_manual_Vol3.pdf#page=396)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum DeliveryStatus {
    /// Indicates that this local APIC has completed sending any previous IPIs
    Idle = 0,

    /// Indicates that this local APIC has not completed sending the last IPI.
    SendPending
}

/// Assertion level options for an interrupt command
///
/// Reference: [`Level`](../../../../../../references/Intel_manual_Vol3.pdf#page=396)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum Level {
    /// For the INIT level de-assert delivery mode this flag must be set to 0
    DeAssert = 0,

    /// For all other delivery modes (non-INIT), level must be Assert
    Assert
}

/// Selects the trigger mode when using the INIT level de-assert delivery mode
///
/// Reference: [`Trigger Mode`](../../../../../../references/Intel_manual_Vol3.pdf#page=397)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum TriggerMode {
    /// Edge delivery mode for INIT
    Edge = 0,

    /// Level delivery mode for INIT
    Level
}

/// Destination shorthand options for an interrupt command
///
/// Reference: [`Destination Shorthand`](../../../../../../references/Intel_manual_Vol3.pdf#page=397)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u8)]
#[allow(unused)]
pub enum DestinationShorthand {
    /// The destination is specified in the destination field.
    None,

    /// The issuing APIC is the one and only destination of the IPI. This destination
    /// shorthand allows software to interrupt the processor on which it is execut-ing.
    /// An APIC implementation is free to deliver the self-interrupt message internally
    /// or to issue the message to the bus and “snoop” it as with any other IPI message.
    Self_,

    /// The IPI is sent to all processors in the system including the processor send-ing
    /// the IPI. The APIC will broadcast an IPI message with the destination field set to
    /// FH for Pentium and P6 family processors and to FFH for Pentium 4 and Intel Xeon
    /// processors.
    AllIncludingSelf,

    /// The IPI is sent to all processors in a system with the exception of the
    /// pro-cessor sending the IPI. The APIC broadcasts a message with the physical
    /// destination mode and destination field set to FH for Pentium and P6 family
    /// processors and to FFH for Pentium 4 and Intel Xeon processors. Support for this
    /// destination shorthand in conjunction with the lowest-priority deliv-ery mode is
    /// model specific. For Pentium 4 and Intel Xeon processors, when this shorthand is
    /// used together with lowest priority delivery mode, the IPI may be redirected back
    /// to the issuing processor
    AllExcludingSelf
}

/// Returns all of the writable APIC registers. This provides all of the necessary
/// registers to save in order to preform a reset during a soft reboot.
pub fn writable_registers() -> [Register; NUM_WRITABLE_REGS] {
    [
        Register::TaskPriority,
        Register::SpuriousInterruptVector,
        Register::LvtCorrectedMachineCheckInterrupt,
        Register::LvtTimer,
        Register::LvtThermalSensor,
        Register::LvtPerformanceMonitoring,
        Register::LvtLint0,
        Register::LvtLint1,
        Register::LvtError,
        Register::TimerInitialCount,
        Register::TimerDivideConfiguration,
    ]
}

/// Bit used to enable the apic
const APIC_ENABLE: u64 = 1 << 11;

/// Bit used to enable the x2apic mode
const APIC_EXTENDED: u64 = 1 << 10;

impl Apic {
    /// Create an APIC rebasing to the provided `base` address. Also, save the original
    /// state of the APIC in order to reset the APIC during a soft reboot
    ///
    /// # Panics
    ///
    /// The given `base` is not a valid 4-KByte aligned physical address
    pub fn new(base: u64) -> Self {
        assert!(base.trailing_zeros() >= 12, 
            "APIC Base address must be a 4-KByte aligned page");

        // Sanity check the base is valid
        assert!(base > 0 && base == (base & 0xffff_f000), "Invalid APIC Base"); 

        // Cache the original state so we can reset during soft reboot
        let mut apic_base_msr = cpu::read_apic_base();

        // Get the BSP bit from the apic base
        let is_bsp = apic_base_msr & (1 << 8) > 0;

        print!("Apic base msr: {:#x} bsp: {}\n", apic_base_msr, is_bsp);

        // Set the ApicBase MSR preserving the BSP bit
        let mut base = base | APIC_ENABLE | (apic_base_msr & (1 << 8));

        // Use x2apic if available in the CPU, otherwise fall back on normal Apic
        let apic_mode = if cpu::has_feature(Feature::X2Apic) {
            // Enable x2apic for the base we are about to write
            base |= APIC_EXTENDED;

            // Enable x2apic on the restore as well if the processor allows for it
            apic_base_msr |= APIC_EXTENDED;

            Mode::X2Apic
        } else {
            panic!("Need a memory manager to give us a page");
            /*
            let mapping = crate::memory_manager::alloc_page()
                .expect("Unable to alloc page for APIC");

            Mode::Apic(&mut *(mapping.0 as *mut [u32; 1024]))
            */
        };

        // Write the new base address
        cpu::write_apic_base(base);

        // Disable the PIC interrupts
        cpu::disable_pic_interrupts();

        // Initialize the original APIC/PIC state to reset to during soft reboot
        let original_state = ResettableState {
            apic_base: apic_base_msr,
            registers: [None; NUM_WRITABLE_REGS],
            primary_pic_interrupt_mask: cpu::get_primary_pic_interrupt_mask(),
            secondary_pic_interrupt_mask: cpu::get_secondary_pic_interrupt_mask(),
        };

        // Return the created APIC
        let mut res = Apic {
            mode: apic_mode,
            original_state,
            is_bsp
        };

        // Save the original state of the APIC
        res.save_state();

        // In order for timer to be enabled by software, we have to tell the APIC that it 
        // can be enabled/disabled via software. Spurious Interrupts will be sent on
        // interrupt vector 0xff.
        res.enable_spurious_interrupt(0xff);

        // Enable timer on interrupt 0xcd
        res.enable_timer(0xcd);

        res
    }

    /// Enable Spurious Interrupt on the given `vector`
    #[inline]
    pub fn enable_spurious_interrupt(&mut self, vector: u8) {
        // Enable bit for spurious interrupt vector
        let software_enable_disable: u32  = 1 << 8;

        unsafe { 
            self.write(Register::SpuriousInterruptVector, 
                software_enable_disable | u32::from(vector));
        }
    }

    /// Get the APIC ID based on the APIC mode
    pub fn id(&self) -> u32 {
        let id = self.read(Register::Id);

        match &self.mode {
            Mode::Apic(_) => {
                // Original APIC has ID in bits 24:27
                (id >> 24) & 0xf
            }
            Mode::X2Apic => {
                // x2APIC has id in bits 0:31
                id
            }
        }
    }

    /// Send an inter-process `interrupt` to the `dest_apic_id` 
    pub fn inter_process_interrupt(&mut self, interrupt: InterruptCommand) -> Result<()> {
        // Make sure we have an APIC_ID for this IPI
        ensure!(interrupt.apic_id.is_some(), &Error::IpiWithoutApicId);

        // Send the given interrupt
        self.write_command_register(interrupt);

        Ok(())
    }

    /// Save the original state of all mutable registers such that we can restore those registers
    /// at a later time
    fn save_state(&mut self) {
        for (i, &reg) in writable_registers().iter().enumerate() {
            self.original_state.registers[i] = Some((reg, self.read(reg)));
        }
    }

    /// Return the Interrupt In-Service registers
    pub fn in_service(&self) -> [u32; 8] {
        [
            self.read(Register::InterruptInService0),
            self.read(Register::InterruptInService1),
            self.read(Register::InterruptInService2),
            self.read(Register::InterruptInService3),
            self.read(Register::InterruptInService4),
            self.read(Register::InterruptInService5),
            self.read(Register::InterruptInService6),
            self.read(Register::InterruptInService7),
        ]
    }

    /// Return the Interrupt Request registers
    pub fn interrupt_request(&self) -> [u32; 8] {
        [
            self.read(Register::InterruptRequest0),
            self.read(Register::InterruptRequest1),
            self.read(Register::InterruptRequest2),
            self.read(Register::InterruptRequest3),
            self.read(Register::InterruptRequest4),
            self.read(Register::InterruptRequest5),
            self.read(Register::InterruptRequest6),
            self.read(Register::InterruptRequest7),
        ]
    }

    /// Restore the state of the APIC to what was read when the APIC was initialized
    pub fn reset(&mut self) {
        // Disable timer to prevent interrupts firing
        self.disable_timer();

        // Restore the original state
        for index in 0..NUM_WRITABLE_REGS {
            if let Some((reg, val)) = self.original_state.registers[index] {
                unsafe { self.write(reg, val); }
            }
        }

        // Disable the APIC by writing a 0 to the spurious interrupt register.
        unsafe { 
            self.write(Register::SpuriousInterruptVector, 0);
        }

        // Write the original base address
        cpu::wrmsr(Msr::ApicBase, self.original_state.apic_base);


        // Attempt to EOI all remaining pending interrupts
        let mut eoi_success = false;
        for _ in 0..100 {
            self.eoi_all();
            if self.in_service() == [0; 8] && self.interrupt_request() == [0; 8] {
                eoi_success = true;
                break;
            }
        }

        // If we failed to EOI all remaining interrupts, print a message and hope it
        // still is fine
        if !eoi_success {
            for _ in 0..10 {
                print!("FAILED TO EOI ALL INTERRUPTS... Good luck on reboot..\n");
            }
        }

        unsafe { 
            // Disable interrupts globally
            cpu::disable_interrupts();

            // Restore primary PIC interrupt mask
            let orig_mask_primary = self.original_state.primary_pic_interrupt_mask;
            cpu::set_primary_pic_interrupt_mask(orig_mask_primary);

            // Restore secondary PIC interrupt mask
            let orig_mask_secondary = self.original_state.secondary_pic_interrupt_mask;
            cpu::set_secondary_pic_interrupt_mask(orig_mask_secondary);
        }
    }

    /// Continuously EOI until all pending interrupts have been serviced
    pub fn eoi_all(&mut self) {
        // EOI all remaining interrupts in the Interrupt Request Registers and the 
        // Interrupt In-Service Register
        'try_again: loop {
            unsafe { 
                cpu::enable_interrupts();
            }

            // Check in-service register for any pending interrupts
            for &reg in &self.in_service() {
                if reg != 0 {
                    self.end_of_interrupt();
                    continue 'try_again;
                }
            }

            // Check request register for any pending interrupts
            for &reg in &self.interrupt_request() {
                if reg != 0 {
                    self.end_of_interrupt();
                    continue 'try_again;
                }
            }

            break;
        }
    }

    /// Get the value for a given Register based on the current mode of the APIC
    fn get_register(&self, reg: Register) -> u32 {
        let val = reg as u32;

        if self.mode == Mode::X2Apic {
            0x800 | (val >> 4)
        } else {
            val
        }
    }

    /// Read the given [`Register`] based on the current mode of the APIC
    pub fn read(&self, reg: Register) -> u32 {
        match &self.mode {
            Mode::Apic(mapping) => {
                // Since the mapping is a [u32] and not a [u8], the index value needs to be
                // divided by 4
                let index = self.get_register(reg) / 4;
                mapping[index as usize]
            }
            Mode::X2Apic => {
                // Get the MSR corresponding to the x2APIC register
                let msr = self.get_register(reg).into();

                // Read the MSR
                cpu::rdmsr_u32(msr)
            }
        }
    }


    /// Send INIT to all cores excluding self
    #[allow(dead_code)]
    pub fn init_all(&mut self) {
        let init_command = InterruptCommand::default()
                            .delivery_mode(DeliveryMode::Init)
                            .level(Level::Assert)
                            .destination_shorthand(DestinationShorthand::AllExcludingSelf);

        // Send INIT to All excluding self
        self.write_command_register(init_command);
    }

    /// Send `INIT` to the given `apic_id`
    pub fn init_id(&mut self, apic_id: u32) -> Result<()> {
        let init_command = InterruptCommand::default()
                                            .delivery_mode(DeliveryMode::Init)
                                            .level(Level::Assert)
                                            .apic_id(apic_id)
                                            .vector(0);

        // Send INIT to All excluding self
        self.inter_process_interrupt(init_command)?;

        Ok(())
    }

    /// Send SIPI to all cores excluding self telling each core to start at physical
    /// address `entry_point`. The entry point must be of the form `0x000XX000`.  
    ///
    /// # Panics
    ///
    /// Panics when the given `entry_point` address is not 4 `KByte` aligned
    #[allow(dead_code)]
    pub fn sipi_all(&mut self, entry_point: u32) {
        assert!(entry_point & 0xfff0_0fff == 0,
            "Invalid entry point address for SIPI_ALL");

        // The entry point is of the following form:
        // 0x000V_V000, where VV is the vector contained in the SIPI message

        // Get the vector number from the entry point address
        let vector = (entry_point >> 12) & 0xff;

        let sipi_command = InterruptCommand::default()
                            .delivery_mode(DeliveryMode::StartUp)
                            .level(Level::Assert)
                            .destination_shorthand(DestinationShorthand::AllExcludingSelf)
                            .vector(vector);

        self.write_command_register(sipi_command);
    }

    /// Send SIPI to the given `apic_id` to start the core at the `entry_point` physical 
    /// address. The entry point must be of the form `0x000XX000`.  
    ///
    /// # Panics
    ///
    /// Panics when the given `entry_point` address is not 4 `KByte` aligned
    pub fn sipi_id(&mut self, apic_id: u32, entry_point: u32) -> Result<()> {
        assert!(entry_point & 0xfff0_0fff == 0,
            "Invalid entry point address for SIPI_ID");

        // The entry point is of the following form:
        // 0x000V_V000, where VV is the vector contained in the SIPI message

        // Get the vector number from the entry point address
        let vector = (entry_point >> 12) & 0xff;

        let sipi_command = InterruptCommand::default()
                                            .delivery_mode(DeliveryMode::StartUp)
                                            .level(Level::Assert)
                                            .apic_id(apic_id)
                                            .vector(vector);

        self.inter_process_interrupt(sipi_command)?;

        Ok(())
    }

    /// Send `INIT SIPI SIPI` to the given `apic_id` starting each core at `entry_point`
    /// physical address
    pub fn init_sipi_sipi_id(&mut self, apic_id: u32, entry_point: u32) -> Result<()> {
        self.init_id(apic_id)?;
        self.sipi_id(apic_id, entry_point)?;
        self.sipi_id(apic_id, entry_point)?;
        Ok(()) 
    }

    /// Send `INIT SIPI SIPI` to all cores excluding self starting each core at
    /// `entry_point` physical address
    #[allow(dead_code)]
    pub fn init_sipi_sipi_all(&mut self, entry_point: u32) {
        self.init_all();
        self.sipi_all(entry_point);
        self.sipi_all(entry_point);
    }

    /// Write the given `val` to the given `Register` based on the current mode of the APIC
    ///
    /// # Safety
    ///
    /// There are no checks on the value being written to the given [`Register`]. The
    /// user must ensure that the value is valid for the given [`Register`].
    pub unsafe fn write(&mut self, reg: Register, val: u32) {
        let reg_val = self.get_register(reg);
        match &mut self.mode {
            Mode::Apic(mapping) => {
                // Since the mapping is a [u32] and not a [u8], the index value needs to 
                // be divided by 4
                let index = reg_val / 4;
                core::ptr::write_volatile(&mut mapping[index as usize], val);
            }
            Mode::X2Apic => {
                let msr = reg_val;
                if reg != Register::EndOfInterrupt {
                    // print!("APIC Write: {:?} {:#x} {:#x}\n", reg, msr, val);
                }

                cpu::wrmsr(msr.into(), u64::from(val));
            }
        }
    }

    /// Write the given `val` into the Interrupt Command Register
    pub fn write_command_register(&mut self, val: InterruptCommand) {
        let val = val.raw(&self.mode);
        let reg_val = self.get_register(Register::InterruptCommand);
        match &mut self.mode {
            Mode::Apic(mapping) => {
                // Write the lower part of the command
                let index = reg_val;
                unsafe {
                    #[allow(clippy::cast_possible_truncation)]
                    core::ptr::write_volatile(&mut mapping[index as usize], val as u32);
                }

                // Write the upper part of the command
                #[allow(clippy::cast_possible_truncation)]
                let val_hi = (val >> 32) as u32;
                let index_hi = index + 0x10;
                unsafe {
                    core::ptr::write_volatile(&mut mapping[index_hi as usize], val_hi);
                }
            }
            Mode::X2Apic => {
                let msr = reg_val;

                cpu::wrmsr(msr.into(), val);
            }
        }
    }

    /// Write the given `val` into the Interrupt Command Register
    #[allow(dead_code)]
    pub fn read_command_register(&mut self) -> u64 {
        unsafe {
            let reg_val = self.get_register(Register::InterruptCommand);
            match &mut self.mode {
                Mode::Apic(mapping) => {
                    // Write the lower part of the command
                    let index = reg_val;
                    let lo = core::ptr::read_volatile(&mapping[index as usize]);

                    let index_hi = index + 0x10;
                    let hi = core::ptr::read_volatile(&mapping[index_hi as usize]);
                    u64::from(hi) << 32 | u64::from(lo)
                }
                Mode::X2Apic => {
                    let msr = reg_val.into();
                    cpu::rdmsr(msr)
                }
            }
        }
    }

    /// Disable the APIC Timer by setting the initial count to zero
    #[inline]
    pub fn disable_timer(&mut self) {
        unsafe {
            // Disable the timer by writing 0 to the initial count. By default, this 
            // should be zero but we are setting it just to be sure
            self.write(Register::TimerInitialCount, 0);
        }
    }

    /// Set the timer divide configuration using the given `config`
    #[inline]
    pub fn set_timer_divide_config(&mut self, config: TimerDivideConfiguration) {
        unsafe { 
            self.write(Register::TimerDivideConfiguration, config as u32);
        }
    }

    /// Set the APIC timer to be `periodic` on the given `interrupt_index`
    #[inline]
    pub fn set_timer_periodic(&mut self, interrupt_index: u8) {
        unsafe { 
            self.write(Register::LvtTimer, 
                    TimerMode::Periodic as u32 | u32::from(interrupt_index));
        }
    }

    /// Set the timer counter value. This is the value that will be decremented and 
    /// when reaches 0, will trigger an interrupt on `interrupt_index`
    #[inline]
    pub fn set_initial_timer_count(&mut self, count: u32) {
        unsafe { 
            self.write(Register::TimerInitialCount, count);
        }
    }

    /// Enables a periodic timer to fire on `interrupt_index`
    #[inline]
    pub fn enable_timer(&mut self, interrupt_index: u8) {
        self.disable_timer();
        self.set_timer_divide_config(TimerDivideConfiguration::DivideBy2);
        self.set_timer_periodic(interrupt_index);
        self.set_initial_timer_count(10_000_000);
    }

    /// Send an EOI to the APIC
    #[inline]
    pub fn end_of_interrupt(&mut self) {
        unsafe { self.write(Register::EndOfInterrupt, 0); }
    }

    /// Send an non-maskable interrupt to the given `apic_id`
    #[allow(unused)]
    #[inline]
    pub fn nmi_id(&mut self, apic_id: u32) {
        let nmi_command = InterruptCommand::default()
                            .delivery_mode(DeliveryMode::NonMaskableInterrupt)
                            .level(Level::Assert)
                            .apic_id(apic_id);

        unsafe { self.write_command_register(nmi_command); }
    }

    /// Send an non-maskable interrupt to all cores excluding self
    #[inline]
    pub fn _nmi_all(&mut self) {
        let nmi_command = InterruptCommand::default()
                            .delivery_mode(DeliveryMode::NonMaskableInterrupt)
                            .level(Level::Assert)
                            .destination_shorthand(DestinationShorthand::AllExcludingSelf);

        self.write_command_register(nmi_command);
    }

    /// Get the current APIC timer count
    pub fn current_timer(&mut self) -> u32 {
        self.read(Register::TimerCurrentCount)
    }
}

/// Current mode the APIC is programmed for: `APIC` or `x2APIC`
#[derive(Debug, PartialEq, Eq)]
pub enum Mode {
    /// Normal APIC mode
    Apic(&'static mut [u32; 1024]),

    /// Apic set in x2apic mode
    X2Apic
}

