//! UEFI Event types

/// An event that can be signaled by UEFI
#[derive(Debug)]
#[repr(u32)]
#[allow(dead_code)]
pub enum Event {
    /// Empty event
    None = 0, 

    /// If an event of this type is not already in the signaled state, then the event’s
    /// NotificationFunction will be queued at the event’s NotifyTpl whenever the event
    /// is being waited on via `EFI_BOOT_SERVICES.WaitForEvent()` or
    /// `EFI_BOOT_SERVICES.CheckEvent()`.
    NotifyWait           = 0x100,

    /// The event’s NotifyFunction is queued whenever the event is signaled.
    NotifySignal         = 0x200,

    /// This event is to be notified by the system when `ExitBootServices()` is invoked.
    /// This event is of type `EVT_NOTIFY_SIGNAL` and should not be combined with any
    /// other event types. The notification function for this event is not allowed to use
    /// the Memory Allocation Services, or call any functions that use the Memory
    /// Allocation Services and must only call functions that are known not to use Memory
    /// Allocation Services, because these services modify the current memory map.The
    /// notification function must not depend on timer events since timer services will
    /// be deactivated before any notification functions are called.
    ExitBootServices     = 0x201,

    /// The event is a timer event and may be passed to `EFI_BOOT_SERVICES.SetTimer()`.
    /// Note that timers only function during boot services time
    Timer                = 0x8000_0000,

    /// The event is to be notified by the system when `SetVirtualAddressMap()` is
    /// performed. This event type is a composite of `EVT_NOTIFY_SIGNAL`, `EVT_RUNTIME`, 
    /// and `EVT_RUNTIME_CONTEXT` and should not be combined with any other event types. 
    VirtualAddressChange = 0x6000_0202,

    /// The event is allocated from runtime memory. If an event is to be signaled after
    /// the call to `EFI_BOOT_SERVICES.ExitBootServices()`, the event’s data structure
    /// and notification function need to be allocated from runtime memory. For more
    /// information, see `SetVirtualAddressMap()`.
    Runtime              = 0x4000_0000,
}
