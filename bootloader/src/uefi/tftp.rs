//! UEFI TFTP Services 
//!
//! Reference: [`30.3 EFI MTFTPv4 Protocol`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1703)

use errchain::prelude::*;
use super::{boot_services, Guid, Status, Error, Event}; 
use crate::print;

/// Definition of the EFI TFTP PROTOCOL GUID
const EFI_TFTP_PROTOCOL_GUID: Guid = Guid(
    0x7824_7c57, 
    0x63db, 
    0x4708, 
    [0x99, 0xc2, 0xa8, 0xb4, 0xa9, 0xa6, 0x1f, 0x6b]
);

/// Attempt to get the currently loaded `TftpService` protocol
pub fn get() -> Result<&'static TftpServices> {
    // Get the TFTP Services from boot services
    let addr = boot_services()?.locate_protocol(&EFI_TFTP_PROTOCOL_GUID)?;

    // Cast the found address into the `TftpServices` protocol
    unsafe { 
       Ok(&*(addr.cast::<TftpServices>()))
    }
}

/// Download the file with `filename` into the given `buffer` from the TFTP server
pub fn read_file(filename: &str, buffer: &mut [u8]) -> Result<()> {
    // Get the TftpServices instance
    let tftp = get()?;

    // Configure the TftpServices instance
    tftp.configure()?;

    // Read the file
    tftp.read_file(filename, buffer)
}

/// A collection of services that are needed for TFTP.
///
/// Reference: [`30.3 EFI MTFTPv4 Protocol`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1703)
#[repr(C)]
#[allow(clippy::module_name_repetitions)]
pub struct TftpServices {
    /// Reads the current operational settings.
    ///
    /// Reference: [`GetModeData()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1705)
    _get_mode_data: unsafe extern fn(),

    /// Initializes, changes, or resets the operational settings for this instance of the
    /// EFI MTFTPv4 Protocol driver. 
    ///
    /// Reference: [`Configure()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1707)
    configure: unsafe extern fn(
        this: &TftpServices,
        config_data: *const ConfigData
    ) -> Status,

    /// Retrieves information about a file from an MTFTPv4 server. 
    ///
    /// Reference: [`GetInfo()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1708)
    _get_info: unsafe extern fn(),

    /// Parses the options in an MTFTPv4 OACK (options acknowledgement) packet. 
    ///
    /// Reference: [`ParseOptionw()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1717)
    _parse_options: unsafe extern fn(),

    /// Downloads a file from an MTFTPv4 server.
    /// 
    /// # Arguments
    ///
    /// * `this`: Pointer to the [`TftpServices`] instance.
    /// * `token`: Pointer to the [`Token`] structure to provide the parameters that 
    ///            are used in this operation.
    ///
    /// # Returns
    ///
    /// [`Status::Success`]: The data file is being downloaded.
    /// [`Status::InvalidParameter`]: One or more of the parameters is not valid:
    ///   * `this` is `NULL`
    ///   * `token` is `NULL`
    ///   * `token.filename` is `NULL`
    ///   * `token.option_count` is not zero and `token.option_list` is `NULL`
    ///   * One or more options in `token.option_list` have wrong format.
    ///   * `token.buffer` and `token.check_packet` are both `NULL`.
    ///   * One or more IPv4 addresses in `token.override_data` are not valid unicast
    ///     IPv4 addresses if `token.override_data` is not `NULL` and the addresses are 
    ///     not set to all zero.
    /// [`Status::Unsupported`]: The EFI MTFTPv4 Protocol driver has not been started.
    /// [`Status::NotStarted`]: When using a default address, configuration (DHCP, BOOTP, 
    ///                         RARP, etc.) is not finished yet.
    /// [`Status::AlreadyStarted`]: This `token` is being used in another MTFTPv4 session
    /// [`Status::AccessDenied`]: The previous operation has not completed yet.
    /// [`Status::OutOfResources`]: Required system resources could not be allocated.
    /// [`Status::DeviceError`]: An unexpected network error or system error occurred.
    /// [`Status::NoMedia`]: There was a media error.
    ///
    /// Reference: [`ReadFile()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1718)
    read_file: unsafe extern fn(
        this: &TftpServices,
        token: &Token
    ) -> Status,

    /// Uploads a file to an MTFTPv4 server.
    ///
    /// Reference: [`WriteFile()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1723)
    _write_file: unsafe extern fn(),

    /// Downloads a related file “directory” from an MTFTPv4 server. 
    ///
    /// Reference: [`WriteFile()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1725)
    _read_directory: unsafe extern fn(),

    /// Polls for incoming data packets and processes outgoing data packets. 
    ///
    /// Reference: [`WriteFile()`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1727)
    _poll: unsafe extern fn(),
}

impl TftpServices {
    /// Safe wrapper around `configure` from the [`TftpServices`]
    pub fn configure(&self) -> Result<()> {
        // Create the configuration settings for the TFTP instance
        let config_data = ConfigData {
            use_default_setting: false,
            station_ip:    [192, 168, 2, 201],
            subnet_mask:   [255, 255, 255, 0],
            local_port:    0,
            gateway_ip:    [192, 168, 2, 2],
            server_ip:     [192, 168, 2, 2],
            initial_server_port: 0,
            try_count:     5,
            timeout_value: 2
        };

        // Call the `configure` callback
        unsafe { 
            let ret = (self.configure)(self, &config_data);

            // Ensure the `read_file` succeeded
            if ret != Status::Success {
                print!("[tftp::configure] Error: {:?}\n", ret);
                return err!(&Error::TftpConfigureFailed);
            }
        }

        Ok(())
    }

    /// Safe wrapper around `read_file` from the [`TftpServices`]
    pub fn read_file(&self, filename: &str, buffer: &mut [u8]) -> Result<()> {
        // Create the override data to specify the TFTP server
        let mut data = OverrideData {
            gateway_ip:    [192, 168, 2, 2],
            server_ip:     [192, 168, 2, 2],
            server_port:   69,
            try_count:     5,
            timeout_value: 5
        };

        // Enable 8k block sizes for faster TFTP transfer
        let options = OptionValue {
            option: "blksize\0".as_ptr(),
            value:  "8192\0".as_ptr(),
        };

        // Create a null terminated filename from the given `filename`
        let mut file = [0_u8; 1024];
        file[..filename.len()].copy_from_slice(filename.as_bytes());

        // Get the buffer size into a stack address in order to receive the number of
        // read bytes
        let mut buffer_size = buffer.len() as u64;

        // Create the token used for the `read_file` callback
        let token = Token {
            // Junk status
            status:           Status::NoMedia,
            event:            Event::None,
            override_data:    &mut data,
            filename:         file.as_ptr(),
            mode_str:         core::ptr::null(),
            option_count:     1,
            option_list:      &options,
            buffer_size:      &mut buffer_size,
            buffer:           buffer.as_mut_ptr(),
            context:          core::ptr::null(),
            check_packet:     0,
            timeout_callback: 0,
            packet_needed:    0,
        };

        // Call the `read_file` callback
        unsafe { 
            let ret = (self.read_file)(self, &token);
        
            // Ensure the `read_file` succeeded
            if ret != Status::Success {
                print!("[tftp::read_file] Error: {:?}\n", ret);
                return err!(&Error::TftpReadFileFailed);
            }
        }

        // Success return
        Ok(())
    }
}

/// Operational state of this TFTP Instance
///
/// Reference: [`EFI_MTFTP4_MODE_DATA`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1705)
#[repr(C)]
pub struct ConfigData {
    /// Set to `true` to use the default station address/subnet mask and the default
    /// route table information.
    use_default_setting: bool,

    /// If `use_default_setting` is `false`, indicates the station address to use.
    station_ip: [u8; 4],

    /// If `use_default_setting` is `false`, indicates the subnet mask to use
    subnet_mask: [u8; 4],

    /// Local port number. Set to zero to use the automatically assigned port number.
    local_port: u16,

    /// If `use_default_setting` is `false`, indicates the gateway IP address to use. 
    gateway_ip: [u8; 4],

    /// The IP address of the MTFTPv4 server.
    server_ip: [u8; 4],

    /// The initial MTFTPv4 server port number. Request packets are sent to this port.
    /// This number is almost always 69 and using zero defaults to 69
    initial_server_port: u16,

    /// The number of times to transmit MTFTPv4 request packets and wait for a response.
    try_count: u16,

    /// The number of seconds to wait for a response after sending the MTFTPv4 request
    /// packet. 
    timeout_value: u16
}


/// TFTP Token with configuration information for [`TftpServices.read_file`]
///
/// Reference: [`EFI_MTFTP4_TOKEN`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1719)
#[derive(Debug)]
#[repr(C)]
pub struct Token {
    /// The status that is returned to the caller at the end of the operation to indicate
    /// whether this operation completed successfully.
    status: Status,

    /// The event that will be signaled when the operation completes. 
    /// If set to NULL, the corresponding function will wait until the read or write 
    /// operation finishes. 
    /// The type of Event must be EVT_NOTIFY_SIGNAL. The Task Priority Level (TPL) of 
    /// Event must be lower than or equal to TPL_CALLBACK.
    event: Event,

    /// If not `NULL`, the data that will be used to override the existing configure
    /// data. Type `EFI_MTFTP4_OVERRIDE_DATA` is defined in [`TftpService.get_info()`]
    override_data: *const OverrideData,

    /// Pointer to the null-terminated ASCII file name string.
    filename: *const u8,

    /// Pointer to the null-terminated ASCII mode string. If `NULL`, `octet` is used.
    mode_str: *const u8,

    /// Number of option/value string pairs
    option_count: u32,

    /// Pointer to an array of option/value string pairs. Ignored if `OptionCount` is 
    /// zero. Both a remote server and this driver implementation should support these 
    /// options. If one or more options are unrecognized by this implementation, it is
    /// sent to the remote server without being changed. Type EFI_MTFTP4_OPTION is
    /// defined in [`TftpService.GetInfo`].
    option_list: *const OptionValue,

    /// On input, the size, in bytes, of Buffer. On output, the number of bytes
    /// transferred
    buffer_size: *mut u64,

    /// Pointer to the data buffer. Data that is downloaded from the MTFTPv4 server is
    /// stored here. Data that is uploaded to the MTFTPv4 server is read from here.
    /// Ignored if `buffer_size` is zero.
    buffer: *mut u8,

    /// Pointer to the context that will be used by `check_packet`, `timeout_callback`
    /// and `packet_needed`.
    context: *const u8,

    /// Pointer to the callback function to check the contents of the received packet. 
    check_packet: usize,

    /// Pointer to the function to be called when a timeout occurs.
    timeout_callback: usize,

    /// Pointer to the function to provide the needed packet contents.
    packet_needed: usize
}

/// Used to override the existing parameters that were set by the
/// [`TftpService.configure()`] function.
///
/// Reference: [`EFI_MTFTP_OVERRIDE_DATA`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1709)
#[derive(Debug)]
#[repr(C)]
pub struct OverrideData {
    /// IP address of the gateway. If set to `0.0.0.0`, the default gateway address that 
    /// was set by the [`TftpService.configure()`] function will not be overridden
    gateway_ip: [u8; 4],

    /// IP address of the MTFTPv4. If set to `0.0.0.0`, is will use the value that 
    /// was set by the [`TftpService.configure()`] function will not be overridden
    server_ip: [u8; 4],
    
    /// MTFTPv4 server port number. If set to zero, it will use the value that was set by
    /// the [`TftpServer.configure()`] function. 
    server_port: u16,

    /// Number of times to transmit MTFTPv4 request packets and wait for a response. If
    /// set to zero, it will use the value that was set by the
    /// [`TftpServer.configure()`] function.
    try_count: u16,

    /// Number of seconds to wait for a response after sending the MTFTPv4 request
    /// packet. If set to zero, it will use the value that was set by the
    /// [`TftpServer.configure()`] function.
    timeout_value: u16
}

/// `TFTPv4` option/value pair
///
/// Reference: [`EFI_MTFTP4_OPTION`](../../../../../../references/UEFI_Spec_2_8_final.pdf#page=1710)
#[derive(Debug)]
#[repr(C)]
pub struct OptionValue {
    /// Pointer to the null-terminated ASCII MTFTPv4 option string.
    option: *const u8,

    /// Pointer to the null-terminated ASCII MTFTPv4 value string.
    value: *const u8
}
