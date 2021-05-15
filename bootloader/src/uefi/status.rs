//! UEFI Status codes

/// EFI Status Code error bit is always set to the high bit
///
/// Reference: [`EFI_STATUS Error Codes`](../../../../../references/UEFI_Spec_2_8_final.pdf#page=2286)
const ERROR_BIT: usize = 1 << (usize::BITS - 1);

/// EFI Status Codes
///
/// Reference: [`EFI_STATUS Error Codes`](../../../../../references/UEFI_Spec_2_8_final.pdf#page=2286)
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(usize)]
#[allow(dead_code, clippy::enum_clike_unportable_variant)]
pub enum Status {
    /// The operation completed successfully
    Success = 0,

    /// The string contained one or more characters that the device could not render and 
    /// were skipped.
    WarningUnknownGlyph = 1,

    /// The handle was closed, but the file was not deleted.
    WarningDeleteFailure = 2,

    /// The handle was closed, but the data to the file was not flushed properly
    WarningWriteFailure = 3,

    /// The resulting buffer was too small, and the data was truncated to the buffer size.
    WarningBufferTooSmallWarn = 4,

    /// The data has not been updated within the timeframe set by localpolicy for this 
    /// type of data.
    WarningStaleData = 5,

    /// The resulting buffer contains UEFI-compliant file system.
    WarningFileSystem = 6,

    /// The operation will be processed across a system reset.
    WarningResetRequired = 7,

    /// The image failed to load
    LoadError = ERROR_BIT | 1,

    /// A parameter was incorrect
    InvalidParameter = ERROR_BIT | 2,

    /// The operation is not supported
    Unsupported = ERROR_BIT | 3,

    /// The buffer was not the proper size for the request
    BadBufferSize = ERROR_BIT | 4,

    /// The buffer is not large enough to hold the requested data. The required buffer 
    /// size is returned in the appropriate parameter when this error occurs
    BufferTooSmallError = ERROR_BIT | 5,

    /// There is no data pending upon return.
    NotReady = ERROR_BIT | 6,

    /// The physical device reported an error while attempting the operation
    DeviceError = ERROR_BIT | 7,

    /// The device cannot be written to
    WriteProteted = ERROR_BIT | 8,

    /// A resource has run ou
    OutOfResources = ERROR_BIT | 9,

    /// An inconstancy was detected on the file system causing the operating to fail
    VolumeCorrupted = ERROR_BIT | 10,

    /// There is no more space on the file system.
    VolumeFull = ERROR_BIT | 11,

    /// The device does not contain any medium to perform the operation
    NoMedia = ERROR_BIT | 12,

    /// The medium in the device has changed since the last access
    MediaChanged = ERROR_BIT | 13,

    /// The item was not found
    NotFound = ERROR_BIT | 14,

    /// Access was denied
    AccessDenied = ERROR_BIT | 15,

    /// The server was not found or did not respond to the request.
    NoResponse = ERROR_BIT | 16,

    /// A mapping to a device does not exist
    NoMapping = ERROR_BIT | 17,

    /// The timeout time expired. 
    Timeout = ERROR_BIT | 18,

    /// The protocol has not been started.
    NotStarted = ERROR_BIT | 19,

    /// The protocol has already been started.
    AlreadyStarted = ERROR_BIT | 20,
    
    /// The operation was aborted
    Aborted = ERROR_BIT | 21,

    /// An ICMP error occurred during the network operation
    IcmpError = ERROR_BIT | 22,

    /// A TFTP error occurred during the network operation
    TftpError = ERROR_BIT | 23,

    /// A protocol error occurred during the network operation
    ProtocolError = ERROR_BIT | 24,

    /// The function encountered an internal version that was incompatible with a version 
    /// requested by the caller
    IncompatibleVersion = ERROR_BIT | 25,

    /// The function was not performed due to a security violation
    SecurityViolation = ERROR_BIT | 26,

    /// A CRC error was detected
    CrcError = ERROR_BIT | 27,

    /// Beginning or end of media was reached
    EndOfMedia = ERROR_BIT | 28,

    /// The end of the file was reached
    EndOfFile = ERROR_BIT | 31,

    /// The language specified was invalid.
    InvalidLanguage = ERROR_BIT | 32,

    /// The security status of the data is unknown or compromised and the data must be 
    /// updated or replaced to restore a valid security status.
    CompromisedData = ERROR_BIT | 33,

    /// There is an address conflict address allocation
    IpAddressConflict = ERROR_BIT | 34,

    /// A HTTP error occurred during the network operation
    HttpError = ERROR_BIT | 35,
}

