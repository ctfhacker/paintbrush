//! Collection of Error Types available to be handled

use crate::ErrorType;

/*
use crate::rangeset::RangeSetError;
use crate::stackvec::StackVecError;

/// Types of errors that can be propagated. For now, each new error type must be added
/// to this enum directly, rather than having a fully generic Error type like `anyhow`
#[derive(Debug, Copy, Clone)]
pub enum ErrorType {
    /// A static str reference error message for the [ErrorChain](super::ErrorChain)
    String(&'static str),

    /// None error from [`Option`]
    NoneError,

    // /// Acpi errors
    // Acpi(crate::acpi::Error),
    
    /// RangeSet errors
    RangeSet(RangeSetError),

    /// Efi errors
    Uefi(crate::uefi::Error),

    /// StackVec errors
    StackVec(StackVecError),

    /// An arithmetic error such as overflow
    Arithmetic(NumericalError),


    /// APIC errors
    #[cfg(target_arch = "x86_64")]
    Apic(crate::intel::apic::Error),

    /* External Error Types */

    /// Page table errors
    PageTable(page_table::Error),

    /// PE parsing errors
    Pe(pe::Error),
}
*/

/// Various numerical errors that can occur
#[derive(Debug, Copy, Clone)]
pub enum NumericalError {
    /// Error caused by an overflow during addition
    AddOverflow,

    /// Error caused by an underflow during subtraction
    SubUnderflow,

    /// Error caused by an overflow during multiplication
    MulOverflow
}

impl ErrorType for NumericalError {}

/*
impl core::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            // Print the strings as strings
            ErrorType::String(s) => write!(f, "{}", s),

            // Everything else, print the Debug version of the `ErrorType`
            _                    => write!(f, "{:?}", self)
        }
    }
}
*/
