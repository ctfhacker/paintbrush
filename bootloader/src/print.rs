//! Provides a [print!] macro wrapper around the 
//! [`efi::output_string`](crate::uefi::output_string) function

use core::fmt::{Result, Write};

/// Empty struct to impl [`Write`] on
pub struct SerialWriter;

impl Write for SerialWriter {
    fn write_str(&mut self, string: &str) -> Result {
        crate::uefi::output_string(string);
        crate::uefi::serial::get().write(string);
        Ok(())
    }
}

/// Standard `print!` macro
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        let _ = <$crate::print::SerialWriter as core::fmt::Write>::write_fmt(
            &mut $crate::print::SerialWriter, 
            format_args!($($arg)*)
        );
    }
}
