//! UEFI Multiprocessor Services 
//!
//! Reference: [`13.4 MP Services Protocol`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=464)

use errchain::prelude::*;
use super::{boot_services, Guid, Status, Error, Event}; 
use crate::print;

/// Definition of the EFI MP SERVICES PROTOCOL GUID
const EFI_MP_SERVICE_PROTOCOL_GUID: Guid = Guid(
    0x3fdd_a605, 
    0xa76e, 
    0x4f46, 
    [0xad, 0x29, 0x12, 0xf4, 0x53, 0x1b, 0x3d, 0x08]
);

/// Attempt to get the currently loaded `MpService` protocol
fn mp_services() -> Result<&'static MpServices> {
    let addr = boot_services()?.locate_protocol(&EFI_MP_SERVICE_PROTOCOL_GUID)?;

    unsafe { 
       Ok(&*(addr.cast::<MpServices>()))
    }
}

/// Return the [`ProcessorCount`] for this platform
pub fn cpu_count() -> Result<ProcessorCount> {
    mp_services()?.get_number_of_processors()
}

/// Execute the given `func` with the given `arg` on the given processor number
/// `proc_num`. The execution of `func` does not block the `BSP`.
pub fn startup_this_ap(proc_num: usize, func: *const fn(usize), arg: usize) -> Result<()> {
    mp_services()?.startup_this_ap(proc_num, func, arg)
}

/// Execute the given `func` with the given `arg` on the given processor number
/// `proc_num`. The execution of `func` does not block the `BSP`.
pub fn _startup_all_aps(func: *const fn(usize), arg: usize) -> Result<()> {
    mp_services()?._startup_all_aps(func, arg)
}

/// Forcibly disable the core with `proc_num`
pub fn _disable_core(proc_num: usize) -> Result<()> {
    mp_services()?._disable_core(proc_num)
}

/// Returns `true` is core with `proc_num` is enabled, `false` otherwise
pub fn _is_core_enabled(proc_num: usize) -> Result<bool> {
    mp_services()?._is_core_enabled(proc_num)
}

/// A collection of services that are needed for multiprocessor management.
///
/// Reference: [`13.4 MP Services Protocol`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=464)
#[repr(C)]
pub struct MpServices {
    /// Gets the number of logical processors and the number of enabled logical
    /// processors in the system
    ///
    /// # Arguments
    ///
    /// * `this`: A pointer to the EFI_MP_SERVICES_PROTOCOL instance
    /// * `num_processors`: Pointer to the total number of logical processors in the
    ///                      system, including the BSP and all enabled and disabled APs.
    /// * `num_enabled_processors`: Pointer to the number of logical processors in the
    ///                             platform including the BSP that are currently enabled
    ///
    /// # Returns
    ///
    /// * [`Status::Success`]: Number of logical processors and enabled logical
    ///                        processors was retrieved
    /// * [`Status::DeviceError`]: Calling processor is an AP.
    /// * [`Status::InvalidParameter`]: `num_processors` or `num_enabled_processors` is
    ///                                  `null`
    ///
    /// Reference: [`GetNumberOfProcessors()`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=466)
    get_number_of_processors: unsafe extern fn(
        this: &MpServices,
        num_processors: &mut usize,
        num_enabled_processors: &mut usize
    ) -> Status,

    /// Gets detailed information on the requested processor at the instant this call is
    /// made
    ///
    /// # Arguments
    ///
    /// * `this`: A pointer to the EFI_MP_SERVICES_PROTOCOL instance
    /// * `proc_num`: The handle number of processor. The range is from 0 to the total
    ///               number of local processors minus 1. The total number of processors 
    ///               can be retrieved by [`MpServices._get_number_of_processors`]
    /// * `proc_info`: A pointer to the buffer where information for the requested
    ///                processor is deposited.
    ///
    /// # Returns
    /// 
    /// * [`Status::Success`]: Processor information was returned.
    /// * [`Status::DeviceError`]: The calling processor is an AP.
    /// * [`Status::InvalidParameter`]: `proc_info` was `NULL`
    /// * [`Status::NotFound`]: The processor with `proc_num` does not exist
    ///
    /// Reference: [`GetProcessorInfo()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=468)
    get_processor_info: unsafe extern fn(
        this: &MpServices,
        proc_num: usize,
        proc_info: &mut ProcessorInformation
    ) -> Status,

    /// Starts up all the enabled APs in the system to run the function provided by the
    /// caller
    ///
    /// # Arguments
    ///
    /// * `this`: A pointer to the EFI_MP_SERVICES_PROTOCOL instance
    /// * `procedure`: A pointer to the function to be run on enabled APs of the system
    /// * `single_thread`: If `true`, then all enabled APs execute the function specified
    ///                    by `procedure` one by one, in ascending order of processor 
    ///                    handle number
    /// * `wait_event`: The event created by the caller with `CreateEvent()` service.
    ///                 If `NULL`, then execute in blocking mode. BSP waits until all APs
    ///                 finish or `timeout` expires.
    ///                 If not `NULL`, then execute in non-blocking mode. BSP requests
    ///                 the function specified by `procedure` to be started on all
    ///                 enabled APs and go on executing immediately. If all return from
    ///                 `procedure` or `timeout` expires, this event is signaled. The BSP
    ///                 can use `CheckEvent` or `WaitForEvent` services to check the
    ///                 state of the event.
    /// * `timeout`: Indicates the time limit in microseconds for APs to return from
    ///              `procedure`. Zero indicates infinity.
    ///              If the timeout expires before all APs return from `procedure`, then
    ///              `procedure` on the failed APs is terminated. All enabled APs are
    ///              available for next function assigned by `_startup_all_aps` or
    ///              `startup_this_ap`.
    ///              If the timeout expires in blocking mode, BSP returns
    ///              [`Status::Timeout`].
    ///              If the timeout expires in non-blocking mode, `WaitEvent` is signaled
    ///              with `SignalEvent()`
    /// * `procedure_argument`: The parameter passed into `procedure` for all APs.
    /// * `failed_cpu_list`: If `NULL`, this parameter is ignored.
    ///                      Otherwise, if all APs finish successfully, then its content
    ///                      is set to `NULL`. If not all APs finish before timeout
    ///                      expires, then its content is set to the address of the
    ///                      buffer holding handle numbers of the failed APs. The buffer
    ///                      is allocated by MP Service Protocol and it's the caller's
    ///                      responsibility to free the buffer with `FreePool()` service.
    ///
    /// # Returns
    ///
    /// * [`Status::Success`]: In blocking mode, all APs finished before timeout expired.
    /// * [`Status::Success`]: In non-blocking mode, fnuction has been dispatched to all
    ///                        enabled APs
    /// * [`Status::Unsupported`]: A non-blocking mode request was made after the UEFI
    ///                            event `EFI_EVENT_GROUP_READY_TO_BOOT` was signaled.
    /// * [`Status::DeviceError`]: Caller process is AP.
    /// * [`Status::NotStarted`]: No enabled APs exist in the system.
    /// * [`Status::NotReady`]: Any enabled APs are bysy.
    /// * [`Status::Timeout`]: In blocking mode, the timeout expired before all enabled
    ///                        APs have finished
    /// * [`Status::InvalidParameter`]: `procedure` is `NULL`
    ///
    /// Reference: [`StartupAllAPs()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=471)
    startup_all_aps: unsafe extern fn(
        this: &MpServices,
        procedure: *const fn(usize),
        single_thread: bool,
        wait_event: Event,
        timeout: usize,
        procedure_argument: usize,
        failed_cpu_list: *mut &mut usize
    ) -> Status,

    /// Starts up the requested AP to run the function provided by the caller.
    ///
    /// # Arguments
    ///
    /// * `this`: A pointer to the EFI_MP_SERVICES_PROTOCOL instance
    /// * `procedure`: A pointer to the function to be run on enabled APs of the system
    /// * `proc_num`: The handle number of processor. The range is from 0 to the total
    ///               number of local processors minus 1. The total number of processors 
    ///               can be retrieved by [`MpServices._get_number_of_processors`]
    /// * `wait_event`: The event created by the caller with `CreateEvent()` service.
    ///                 If `NULL`, then execute in blocking mode. BSP waits until all APs
    ///                 finish or `timeout` expires.
    ///                 If not `NULL`, then execute in non-blocking mode. BSP requests
    ///                 the function specified by `procedure` to be started on all
    ///                 enabled APs and go on executing immediately. If all return from
    ///                 `procedure` or `timeout` expires, this event is signaled. The BSP
    ///                 can use `CheckEvent` or `WaitForEvent` services to check the
    ///                 state of the event.
    /// * `timeout`: Indicates the time limit in microseconds for APs to return from
    ///              `procedure`. Zero indicates infinity.
    ///              If the timeout expires before all APs return from `procedure`, then
    ///              `procedure` on the failed APs is terminated. All enabled APs are
    ///              available for next function assigned by `_startup_all_aps` or
    ///              `startup_this_ap`.
    ///              If the timeout expires in blocking mode, BSP returns
    ///              [`Status::Timeout`].
    ///              If the timeout expires in non-blocking mode, `WaitEvent` is signaled
    ///              with `SignalEvent()`
    /// * `procedure_argument`: The parameter passed into `procedure` for all APs.
    /// * `finished`: If `NULL`, this parameter is ignored.
    ///               In blocking mode, this parameter is ignored.
    ///               In non-blocking mode, if AP returns from `procedure` before the
    ///               timeout expires, its content is set to `true`. Otherwise, the value
    ///               is set to `false`. The caller can determine if the AP returned from
    ///               `procedure` by evaluating this value.
    ///
    /// # Returns
    ///
    /// * [`Status::Success`]: In blocking mode, all APs finished before timeout expired.
    /// * [`Status::Success`]: In non-blocking mode, fnuction has been dispatched to all
    ///                        enabled APs
    /// * [`Status::Unsupported`]: A non-blocking mode request was made after the UEFI
    ///                            event `EFI_EVENT_GROUP_READY_TO_BOOT` was signaled.
    /// * [`Status::DeviceError`]: Caller process is AP.
    /// * [`Status::Timeout`]: In blocking mode, the timeout expired before all enabled
    ///                        APs have finished
    /// * [`Status::NotReady`]: Any enabled APs are bysy.
    /// * [`Status::NotFound`]: The processor with `proc_num` does not exist
    /// * [`Status::InvalidParameter`]: `proc_num` specifies the BSP or disabled AP.
    /// * [`Status::InvalidParameter`]: `procedure` is `NULL`
    ///
    /// Reference: [`StartupThisAP()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=475)
    startup_this_ap: unsafe extern fn(
        this: &MpServices,
        procedure: *const fn(usize),
        proc_num: usize,
        wait_event: Event,
        timeout: usize,
        procedure_argument: usize,
        finished: *mut &mut bool
    ) -> Status,

    /// witches the requested AP to be the BSP from that point onward.  This service
    /// changes the BSP for all purposes.
    ///
    /// Reference: [`SwitchBSP()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=478)
    _switch_bsp: unsafe extern fn(),

    /// Enables and disables the given AP from that point onward.
    ///
    /// # Arguments
    ///
    /// * `this`: A pointer to the EFI_MP_SERVICES_PROTOCOL instance
    /// * `proc_num`: The handle number of processor. The range is from 0 to the total
    ///               number of local processors minus 1. The total number of processors 
    ///               can be retrieved by [`MpServices._get_number_of_processors`]
    /// * `enable`: Specifies the new state for the processor specified by `proc_num`.
    ///             `true` for enabled, `false` for disabled
    /// * `health_flag`: If not `NULL`, a pointer to the value that specifies the new
    ///                  health status of the AP. Only `PROCESSOR_HEALTH_STATUS_BIT` is
    ///                  used.
    ///
    /// # Returns
    ///
    /// * [`Status::Success`]: In blocking mode, all APs finished before timeout expired.
    /// * [`Status::Success`]: In non-blocking mode, fnuction has been dispatched to all
    ///                        enabled APs
    /// * [`Status::Unsupported`]: Enabling or disabling an AP cannot be completed prior
    ///                            to this service returning.
    /// * [`Status::Unsupported`]: Enabling or disabling an AP is not supported.
    /// * [`Status::DeviceError`]: Caller process is AP.
    /// * [`Status::NotFound`]: The processor with `proc_num` does not exist
    /// * [`Status::InvalidParameter`]: `proc_num` specifies the BSP
    ///
    /// Reference: [`EnableDisableAP()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=480)
    enable_disable_ap: unsafe extern fn(
        this: &MpServices,
        proc_num: usize,
        enable: bool,
        health_flag: u32
    ) -> Status,

    /// Gets the handle number of the caller processor.
    ///
    /// # Arguments 
    ///
    /// * `this`: A pointer to the EFI_MP_SERVICES_PROTOCOL instance
    /// * `proc_num`: The handle number of processor. The range is from 0 to the total
    ///               number of local processors minus 1. The total number of processors 
    ///               can be retrieved by [`MpServices._get_number_of_processors`]
    /// # Returns
    ///
    /// * [`Status::Success`]: The current processor handle number was returned in
    ///                        `proc_num`
    /// * [`Status::InvalidParameter`]: `proc_num` specifies the BSP
    ///
    /// Reference: [`WhoAmI()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=482)
    _whoami: unsafe extern fn(
        this: &MpServices,
        proc_num: &mut usize,
    ) -> Status,
}

/// Structure for returning the number of processors back from the
/// [`MpServices::get_number_of_processors`] service
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct ProcessorCount {
    /// Total number of logical processors
    pub total: usize,

    /// Total number of `enabled` logical processors
    pub enabled: usize
}

impl MpServices {
    /// Get the total number of logical processors and total number of enabled processors
    ///
    /// # Returns
    ///
    /// [`ProcessorCount`] containing total and enabled logical processors
    ///
    /// # Errors
    ///
    /// Any error thrown returned by [`MpServices::get_number_of_processors`] will be
    /// returned from this function
    #[allow(dead_code)]
    pub fn get_number_of_processors(&self) -> Result<ProcessorCount> {
        let mut total   = 0;
        let mut enabled = 0;

        // Call the `get_number_of_processors` service
        unsafe { 
            let ret = (self.get_number_of_processors)(self, &mut total, &mut enabled);

            // Ensure the function call succeeded
            if ret != Status::Success {
                print!("[GetNumberOfProcessors] failed: {:?}\n", ret);
                return err!(&Error::GetNumberOfProcessorsFailed);
            }
        }

        // Return the processor count
        Ok(ProcessorCount { total, enabled })
    }

    /// Wrapper around the given `startup_this_ap` service from `MpServices`. Execute the
    /// given `func` with the given `arg` on the given processor number `cpu_num`
    pub fn _startup_all_aps(&self, func: *const fn(usize), arg: usize) 
            -> Result<()> {
        // Call the procedure with the following options:
        // * No wait event
        // * No timeout
        unsafe { 
            let ret = (self.startup_all_aps)(
            /* this:               */ self,
            /* procedure:          */ func,
            /* single_thread:      */ false,
            /* wait_event:         */ Event::NotifyWait,
            /* timeout:            */ 0,
            /* procedure_argument: */ arg,
            /* failed_cpu_list:    */ core::ptr::null_mut()
            );

            // Ensure the function call succeeded
            if ret != Status::Success {
                print!("[StartupAllAps] failed: {:?}\n", ret);
                return err!(&Error::StartupAllAPsFailed);
            }
        }

        Ok(())
    }

    /// Wrapper around the given `startup_this_ap` service from `MpServices`. Execute the
    /// given `func` with the given `arg` on the given processor number `cpu_num`
    pub fn startup_this_ap(&self, cpu_num: usize, func: *const fn(usize), arg: usize) 
            -> Result<()> {
        // Call the procedure with the following options:
        // * No wait event
        // * No timeout
        unsafe { 
            let ret = (self.startup_this_ap)(
                /* this:       */ self,
                /* procedure:  */ func,
                /* proc_num:   */ cpu_num, 
                /* wait_event: */ Event::NotifyWait,
                /* timeout:    */ 0,
                /* procedure_argument: */ arg,
                /* finished:   */ core::ptr::null_mut()
            );

            // Ensure the function call succeeded
            if ret != Status::Success {
                print!("[StartupThisAp] failed: {:?}\n", ret);
                return err!(&Error::StartupThisApFailed);
            }
        }

        Ok(())
    }

    /// Wrapper around `enable_disable_ap` set to disable the given cpu with `cpu_num`
    pub fn _disable_core(&self, cpu_num: usize) -> Result<()> {
        unsafe {
            let ret = (self.enable_disable_ap)(
                /* this:        */ self,
                /* proc_num:    */ cpu_num,
                /* enable:      */ false,
                /* health_flag: */ 0
            );

            // Ensure the function call succeeded
            if ret != Status::Success {
                print!("[DisableCore] failed: {:?}\n", ret);
                return err!(&Error::DisableCoreFailed);
            }
        }

        Ok(())
    }

    /// Returns `true` is core with `proc_num` is enabled, `false` otherwise
    pub fn _is_core_enabled(&self, cpu_num: usize) -> Result<bool> {
        let mut info = ProcessorInformation::default();

        unsafe {
            let ret = (self.get_processor_info)(
                /* this:      */ self,
                /* proc_num:  */ cpu_num,
                /* proc_info: */ &mut info
            );

            // Ensure the function call succeeded
            if ret != Status::Success {
                print!("[IsCoreEnabled] failed: {:?}\n", ret);
                return err!(&Error::GetProcessorInfoFailed);
            }
        }

        Ok(info._is_enabled())
    }
}

/// Processor information returned from [`MpServices::get_processor_info`]
#[derive(Default)]
#[repr(C)]
struct ProcessorInformation {
    /// The unique processor ID determined by hardware.
    proc_id: u64,

    /// Flags indicating if the processor is BSP or AP, if the processor is enabled or
    /// disabled, and if the processor is healthy.
    status_flag: u32,

    /// The physical location of the processor, including the physical package number
    /// that identifies the cartridge, the physical core number within the package, and
    /// logical thread number within core.
    location: CpuPhysicalLocation
}

impl ProcessorInformation {
    /// Returns `true` if the processor is enabled, `false` otherwise
    ///
    /// Reference: [`StatusFlag Bits Definition`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=469)
    pub fn _is_bsp(&self) -> bool {
        self.status_flag & 0x1 > 0
    }

    /// Returns `true` if the processor is enabled, `false` otherwise
    ///
    /// Reference: [`StatusFlag Bits Definition`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=469)
    pub fn _is_enabled(&self) -> bool {
        self.status_flag & 0x2 > 0
    }

    /// Returns `true` if the processor is healthy, `false` otherwise
    ///
    /// Reference: [`StatusFlag Bits Definition`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=469)
    pub fn _is_healthy(&self) -> bool {
        self.status_flag & 0x4 > 0
    }
}

/// CPU Processor location returned from [`MpServices::get_processor_info`]
#[derive(Default)]
#[repr(C)]
struct CpuPhysicalLocation {
    /// Zero-based physical package number that identifies the cartridge of the processor
    package: u32,

    /// Zero-based physical core number within package of the processor
    core: u32,

    /// Zero-based logical thread number within core of the processor
    thread: u32
}
