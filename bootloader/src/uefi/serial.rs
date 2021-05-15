//! UEFI Serial IO Protocol
//!
//! Reference: [`12.8 Serial I/O Protocol`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=465)

use errchain::prelude::*;
use super::{boot_services, Guid, Status, Error}; 
use crate::print;

/// Definition of the `EFI_SERIAL_IO_PROTOCOL` Guid
const EFI_SERIAL_IO_PROTOCOL_GUID: Guid = Guid(
    0xbb25_cf6f, 
    0xf1d4, 
    0x11d2, 
    [0x9a, 0x0c, 0x00, 0x90, 0x27, 0x3f, 0xc1, 0xfd]
);

/// Attempt to get the currently loaded `SerialIo` protocol
pub fn get() -> &'static SerialIo {
    let addr = boot_services().expect("Failed to get boot services")
        .locate_protocol(&EFI_SERIAL_IO_PROTOCOL_GUID).expect("Failed to locate serial");

    unsafe { &*(addr.cast::<SerialIo>()) }
}

/// A collection of services that are needed for multiprocessor management.
///
/// Reference: [`13.4 MP Services Protocol`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=464)
#[repr(C)]
#[allow(clippy::module_name_repetitions)]
pub struct SerialIo {
    /// The revision to which the `EFI_SERIAL_IO_PROTOCOL` adheres. All future revisions
    /// must be backwards compatible. If a future version is not back wards compatible,
    /// it is not the same GUID.
    ///
    /// Reference: [`Reset()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=466)
    revision: u32,

    /// Resets the hardware device.
    ///
    /// # Returns
    ///
    /// * [`Status::Success`]: The serial device was reset
    /// * [`Status::DeviceError`]: The serial device could not be reset
    ///
    /// Reference: [`Reset()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=471)
    reset: unsafe extern fn(this: &SerialIo) -> Status,

    /// Sets the baud rate, receive FIFO depth, transmit/receive time out, parity, data
    /// bits, and stop bits on a serial device.
    ///
    /// # Arguments
    ///
    /// * `this`: A pointer to the EFI_SERIAL_IO_PROTOCOL instance
    /// * `baud_rate`: The requested baud rate. A `baud_rate` of `0` will use the
    ///                device's default interface speed.
    /// * `receive_fifo_depth`: The requested depth of the FIFO on the receive side of
    ///                         the serial interface. A `receive_fifo_speed` of `0` will 
    ///                         use the device's default FIFO depth
    /// * `timeout`: The requested time out for a single character in microseconds. The
    ///              timeout applies to both the transmit and receive side of the
    ///              interface. A `timeout` value of `0` will use the device's default
    ///              time out value.
    /// * `parity`: The type of parity to use on this serial device. A `parity` value of
    ///             `DefaultParity` will use the device’s default parity value. 
    /// * `data_bits`: The number of data bits to use on this serial device. A 
    ///                `data_bits` value of `0` will use the device’s default data bit 
    ///                setting.
    /// * `stop_bits`: The number of stop bits to use on this serial device. A `stop_bits` 
    ///                value of `DefaultStopBits` will use the device’s default number of 
    ///                stop bits. 
    ///
    /// # Returns
    ///
    /// * [`Status::Success`]: The new attributes were set on the serial device.
    /// * [`Status::InvalidParameter`]: One ore more of the attributes has an unsupported
    ///                                 value.
    /// * [`Status::DeviceError`]: The serial device is not functioning correctly.
    ///
    /// Reference: [`SetAttributes()`](../../../../../references/UEFI_PI_Spec_1_7.pdf#page=471)
    set_attributes: unsafe extern fn(
        this: &SerialIo,
        baud_rate: u64,
        receive_fifo_depth: u64,
        timeout: u32,
        partity: Parity,
        data_bits: u8,
        stop_bits: StopBits
    ) -> Status,

    /// 
    set_control: unsafe extern fn() -> Status,

    /// 
    get_control: unsafe extern fn() -> Status,

    /// Writes data to a serial device.
    ///
    /// # Arguments
    ///
    /// * `this`: A pointer to the EFI_SERIAL_IO_PROTOCOL instance
    /// * `buffer_size`: On input, the size of the `buffer`. On output, the amount of
    ///                  data actually written.
    /// * `buffer`: The buffer of data to write
    ///
    /// # Returns
    ///
    /// * [`Status::Success`]: The data was written
    /// * [`Status::DeviceError`]: The device reported an error
    /// * [`Status::Timeout`]: The data write was stopped due to a timeout
    write: unsafe extern fn(
        this:        &SerialIo,
        buffer_size: &mut usize,
        buffer:      *const u8
    ) -> Status,

    /// Reads data from a serial device.
    ///
    /// # Arguments
    ///
    /// * `this`: A pointer to the EFI_SERIAL_IO_PROTOCOL instance
    /// * `buffer_size`: On input, the size of the `buffer`. On output, the amount of
    ///                  data returned in `buffer`.
    /// * `buffer`: The buffer to return the data into.
    ///
    /// # Returns
    ///
    /// * [`Status::Success`]: The data was read
    /// * [`Status::DeviceError`]: The device reported an error
    /// * [`Status::Timeout`]: The data write was stopped due to a timeout or overrun
    read: unsafe extern fn() -> Status,

    /// Pointer to the Serial [`Mode`]
    pub mode: *const Mode,

    /// Pointer to a [`Guid`] identifying the device connected to the serial port. This
    /// field is NULL when the protocol is installed by the serial port driver and may be
    /// populated by a platform driver for a serial port with a known device attached.
    device_type_guid: &'static Guid,
}

impl SerialIo {
    /// Get the mode of the found serial port
    pub fn _mode(&self) -> Mode {
        unsafe { *self.mode }
    }

    /// Write the given `data` bytes to the serial port
    pub fn write_bytes(&self, data: &[u8]) -> Result<()> {
        // Create the data length variable used to get the output length
        let mut data_len = data.len();

        unsafe {
        // Write the bytes
            let ret = (self.write)(self, &mut data_len, data.as_ptr());

            // Ensure the write succeeded
            if ret != Status::Success {
                print!("[serial::write_bytes] Error: {:?}\n", ret);
                return err!(&Error::SerialWriteFailed);
            }
        }

        Ok(())
    }

    /// Write the given `data` str
    pub fn write(&self, data: &str) -> Result<()> {
        self.write_bytes(data.as_bytes())
    }
}

impl core::fmt::Write for SerialIo {
    fn write_str(&mut self, string: &str) -> core::fmt::Result {
        let _ = self.write(string);
        core::result::Result::Ok(())
    }
}

/// Serial I/O Mode
///
/// Reference: [`SERIAL_IO_MODE`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=467)
#[derive(Debug, Copy, Clone)]
pub struct Mode {
    /// A mask of the control bits that the device supports.
    control_mask: u32,

    /// If applicable, the number of microseconds to wait before timing out a Read or
    /// Write operation.
    timeout: u32,

    /// If applicable, the current baud rate setting of the device; otherwise, baud rate
    /// has the value of zero to indicate that device runs at the device's designed
    /// speed.
    baud_rate: u64,

    /// The number of characters the device will buffer on input.
    receive_fifo_depth: u32,

    /// The number of data bits in each character.
    data_bits: u32,

    /// If applicable, this is the `EFI_PARITY_TYPE` that is computed or checked as each
    /// character is transmitted or received. If the device does not support parity the
    /// value is the default parity value.
    parity: u32,

    /// If applicable, the `EFI_STOP_BITS_TYPE` number of stop bits per character. If the
    /// device does not support stop bits the value is the default stop bit value.
    stop_bits: u32
}

/// `EFI_PARITY_TYPE`
///
/// Reference: [`EFI_PARITY_TYPE`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=628)
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum Parity {
    /// Device default parity
    Default = 0,

    /// No parity
    None,

    /// Even parity
    Even,

    /// Odd parity
    Odd,

    /// Mask parity
    Mask,

    /// Space parity
    Space
}

/// `EFI_STOP_BITS_TYPE`
///
/// Reference: [`EFI_STOP_BITS_TYPE`](../../../../../../references/UEFI_PI_Spec_1_7.pdf#page=628)
#[derive(Debug, Copy, Clone)]
#[allow(dead_code)]
pub enum StopBits {
    /// Device default stop bits
    Default = 0,

    /// 1 stop bit
    One,

    /// 1.5 stop bits
    OneFive,

    /// 2 stop bits
    Two,
}
