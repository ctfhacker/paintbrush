//! UEFI DHCP Services 
//!
//! Reference: [`29.2 EFI DHCPv4 Protocol`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1534)

use errchain::prelude::*;
use super::{boot_services, Guid, Status, Error, Event}; 

/// Definition of the EFI DHCP PROTOCOL GUID
const EFI_DHCP_PROTOCOL_GUID: Guid = Guid(
    0x9d9a_39d8, 
    0xbd42, 
    0x4a73, 
    [0xa4, 0xd5, 0x8e, 0xe9, 0x4b, 0xe1, 0x13, 0x80]
);

/// Attempt to get the currently loaded `DhcpService` protocol
pub fn get() -> Result<DhcpService> {
    let addr = boot_services()?.locate_protocol(&EFI_DHCP_PROTOCOL_GUID)?;

    unsafe { 
       Ok(&*(addr.cast::<DhcpServices>()))
    }
}

/// A collection of services that are needed for DHCP
///
//! Reference: [`29.2 EFI DHCPv4 Protocol`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1534)
#[repr(C)]
pub struct DhcpService {
    get_mode_data: unsafe extern fn(
        this: &DhcpService,
        mode_data: &mut ModeData
    ),
}

pub struct ModeData {
    /// The EFI DHCPv4 Protocol driver operating state. 
    state: State,

    /// The configuration data of the current EFI DHCPv4 Protocol driver instance. 
    config_data: ConfigData,

    /// The client IP address that was acquired from the DHCP server. If it is zero, the
    /// DHCP acquisition has not completedyet and the following fields in this structure
    /// are undefined.
    client_ipv4: [u8; 4],
    
    /// The local hardware address.
    client_mac:  [u8; 6],

    /// The server IP address that is providing the DHCP service to this client.
    server_ipv4: [u8; 4],

    /// The router IP address that was acquired from the DHCP server. May be zero if the
    /// server does not offer this address. 
    router_ipv4: [u8; 4],

    /// The subnet mask of the connected network that was acquired from the DHCP server. 
    subnet_mask: [u8; 4],

    /// The lease time (in 1-second units) of the configured IP address. The value
    /// `0xFFFFFFFF` means that the lease time is infinite. A default lease of 7 days is
    /// used if the DHCP server does not provide a value.
    lease_time: u32,

    /// The cached latest `DHCPACK` or `DHCPNAK` or `BOOTP REPLY` packet. May be `NULL` 
    /// if no packet is cached.
    reply_packet: usize
}

/// DHCP operational states
pub enum State {
   /// The EFI DHCPv4 Protocol driver is stopped and [`DhcpServer.configure()`] needs to 
   /// be called. The rest of the `ModeData` structure is undefined in this 
   /// state
   Stopped = 0x0, 

   /// The EFI DHCPv4 Protocol driver is inactive and [`DhcpServer.start()`] needs to be 
   /// called. The rest of the [`ModeData`] structure is undefined in this state.
   Init = 0x1, 

   /// The EFI DHCPv4 Protocol driver is collecting DHCP offer packets from DHCP servers.
   /// The rest of the [`ModeData`] structure is undefined in this state.
   Selecting = 0x2, 

   /// The EFI DHCPv4 Protocol driver has sent the request to the DHCP server and is
   /// waiting for a response. The rest of the [`ModeData`] structure is undefined
   /// in this state.
   Requesting = 0x3, 

   /// The DHCP configuration has completed. All of the fields in the [`ModeData`]
   /// structure are defined.
   Bound = 0x4,

   /// The DHCP configuration is being renewed and another request has been sent out, but
   /// it has not received a response from the server yet. All of the fields in the
   /// [`ModeData`] structure are available but may change soon
   Renewing = 0x5, 

   /// The DHCP configuration has timed out and the EFI DHCPv4 Protocol driver is trying
   /// to extend the lease time. The rest of the [`ModeData`] structure is undefined in
   /// this state.
   Rebinding = 0x6, 

   /// The EFI DHCPv4 Protocol driver is initialized with a previously allocated or known
   /// IP address. [`DhcpService.start()`] needs to be called to start the
   /// configuration process. The rest of the [`ModeData`] structure is undefined
   /// in this state.
   InitReboot = 0x7, 

   /// The EFI DHCPv4 Protocol driver is seeking to reuse the previously allocated IP
   /// address by sending a request to the DHCP server. The rest of the
   /// [`ModeData`] structure is undefined in this state.
   Rebooting = 0x8,
}

pub struct ConfigData {
    discover_try_count: u32,
    discover_timeout: *mut u32,
    request_try_count: u32,
    request_timeout: *mut u32,
    client_address: [u8; 4],
    callback: usize,
    callback_context: usize,
    option_count: u32,
    option_list: *mut usize
}
