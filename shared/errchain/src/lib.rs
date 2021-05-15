//! Error chaining library for the kernel that enables a call stack of where the error
//! occurs.
//!
//! This module implements a custom `Result` type called [`ErrorChainResult`]. This is
//! because we can't overwrite the [`Try`] impl for [`core::result::Result`], so we had to
//! create our own type. 
//!
//! ## Example
//!
//! ```
//! use crate::errchain::{Err, Ok, Result, Context, ErrorChain, ErrorType};
//!
//! /// Safely add two numbers together. 
//! pub fn safe() -> Result<u64> {
//!     // `add!` is a wrapper around handling [`checked_add`]
//!     Ok(add!(4u64, 3u64))
//! }
//!
//! /// Crash due to an underflow
//! pub fn crash() -> Result<u64> {
//!     // Likewise, `sub!` is a wrapper around handling [`checked_sub`]
//!     Ok(sub!(1u64, u64::MAX))
//! }
//!
//! pub fn test_crash() -> Result<u64> {
//!     // Example of adding a comment to the call stack
//!     Ok(crash().context_str("crash function failed")?)
//! }
//!
//! /// Main entry point that can return a [Result]
//! fn try_main() -> Result<()> {
//!     // Example of using the `context` syntax to add a message into the `ErrorChain`
//!     safe().context_str("Crashed during the safe call!")?;
//!
//!     // No context being used will just store the file!() and line!() of this crash on Err
//!     test_crash()?;
//!
//!     print!("We didn't crash!\n");
//!
//!     Ok(())
//! }
//!
//! pub fn always_err() -> Result<()> {
//!     return Err(err_str!("This always errors"));
//! }
//!
//! fn main() {
//!     match try_main() {
//!         Err(err) => {
//!             print!("--- MAIN ERROR ---");
//!             print!("{:?}\n", err);
//!             panic!("MAIN ERROR");
//!         }
//!         Ok(_) => print!("Main success!\n")
//!     }
//! }
//! ```
//!
//! ## Results
//!
//! ```text
//! --- MAIN ERROR ---
//! src/main.rs:10: Arithmetic(SubUnderflow)
//! src/main.rs:14: crash function failed
//! src/main.rs:23: ...
//!
//! thread 'main' panicked at 'MAIN ERROR', src/main.rs:35:13
//! ```

#![no_std]
#![feature(trait_alias)]
#![feature(try_trait)]
#![feature(associated_type_bounds)]
#![feature(const_fn_unsize)]
#![feature(const_fn_trait_bound)]

use core::ops::Try;
use core::fmt::Debug;

pub mod prelude;

mod types;
pub use types::NumericalError;


// pub use types::*;

/// Maximum length call stack
pub const MAX_CHAIN_LEN: usize = 8;

#[derive(Copy, Clone)]
pub enum Error {
    Empty,
    Continue
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "...")
    }
}

pub trait ErrorType: core::fmt::Debug {}

impl ErrorType for str {}
impl ErrorType for Error {}

/// Each of the descriptions for the locations of a given error
#[derive(Debug)]
pub struct Message {
    /// File name of the current error
    file:  &'static str,

    /// Line in the file of the current error
    line:  u32,

    /// Actual error
    error: &'static dyn Debug
}

impl Message {
    pub const fn empty() -> Self {
        Self {
            file:  "",
            line:  0,
            error: &Error::Empty
        }
    }
}

/// Error struct that holds the current chain of contexts that caused the given error
pub struct ErrorChain {
    /// Chain of messages 
    chain: [Message; MAX_CHAIN_LEN],

    /// Current length of the error chain
    chain_len: usize,

    /// Maximum length of `[file:line]` string for the current chain. Used in padding the 
    /// format string
    max_padding: usize,
}

impl ErrorChain {

    /// Create a new chain using the current `Location` information
    #[track_caller]
    #[allow(dead_code)]
    pub fn new(error: &'static dyn Debug) -> Self {
        let caller = core::panic::Location::caller();

        Self::new_with_debug(caller.file(), caller.line(), error)
    }

    /// Create a new chain using the given `file`, `line`, and [`ErrorType`]
    // pub fn new_with_debug(file: &'static str, line: u32, error: &'static ErrorType) -> Self {
    pub fn new_with_debug(file: &'static str, line: u32, error: &'static dyn Debug) -> Self {
        const VAL: Message = Message::empty();

        // Create a new chain using debug information
        // let mut chain = [Message::empty(); MAX_CHAIN_LEN];
        let mut chain = [VAL; MAX_CHAIN_LEN];

        // Insert the given error into the chain
        chain[0] = Message { file, line, error };

        // Calculate the number of digits in the line to know the padding needed to
        // pretty print the call stack on panic
        let line_len = match line {
                  0..=9       => 1,
                 10..=99      => 2,
                100..=999     => 3,
               1000..=9999    => 4,
              10000..=99999   => 5,
            100_000..=999_999 => 6,
            _ => panic!("Why do you have a file with 1000000 lines?!")
        };

        // Return the newly created error
        ErrorChain { 
            chain, 
            chain_len: 1, 
            max_padding: file.len() + line_len 
        }
    }

    /// Get the last element added to the chain
    pub fn last(&self) -> Option<&Message> {
        // Return None if there are no elements in the chain
        if self.chain_len == 0 {
            return None;
        }

        // Return the last element found in the chain
        Some(&self.chain[self.chain_len - 1])
    }

    /// Get the first element added to the chain
    pub fn first(&self) -> Option<&Message> {
        // Return None if there are no elements in the chain
        if self.chain_len == 0 {
            return None;
        }

        // Return the first element found in the chain
        Some(&self.chain[0])
    }

    #[track_caller]
    fn extend_chain(mut self, file: &'static str, line: u32, error: &'static dyn Debug) 
        -> ErrorChain {
        // If the chain is full, we can't add anymore, return what we have thus far
        if self.chain_len == MAX_CHAIN_LEN {
            return self;
        }

        // Add the new message to the chain
        self.chain[self.chain_len] = Message { file, line, error };

        // Increase the length of the chain
        self.chain_len += 1;

        // Calculate the length of the line number since we can't allocate in errchain
        let line_len = match line {
                  0..=9       => 1,
                 10..=99      => 2,
                100..=999     => 3,
               1000..=9999    => 4,
              10000..=99999   => 5,
            100_000..=999_999 => 6,
            _ => panic!("Why do you have a file with 1000000 lines?!")
        };

        // Adjust the max padding if the new element is largest thus far
        self.max_padding = core::cmp::max(self.max_padding, file.len() + line_len);

        // Return the newly modify error
        self
    }
}

impl core::fmt::Debug for ErrorChain {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let _ = write!(f, "\n");

        // Take only the messages from the chain len
        for Message { file, line, error } in self.chain.iter().take(self.chain_len) {
            // Write the file:line prefix
            let _ = write!(f, "{}:{}:", file, line);

            // Calculate the length of the line number
            let line_len = match line {
                0..=9 => 1,
                10..=99 => 2,
                100..=999 => 3,
                1000..=9999 => 4,
                10000..=99999 => 5,
                100_000..=999_999 => 6,
                _ => panic!("Why do you have a file with 1000000 lines?!")
            };

            let max_len = self.max_padding
                                .saturating_sub(file.len())
                                .saturating_sub(line_len) + 1;

            // Write the padding to vertically align the messages
            for _ in 0..max_len {
                let _ = write!(f, " ");
            }

            // Write the message
            let _ = write!(f, "{:?}\n", error);
        }

        core::fmt::Result::Ok(())
    }
}

/// [`Result`] type that represents success or starts an `ErrorChain` on failure
#[allow(clippy::large_enum_variant)]
pub enum ErrorChainResult<T> {
    /// Success type
    Ok(T),

    /// Failure type
    Err(ErrorChain)
}

impl<T> ErrorChainResult<T> {
    /// Returns the contained `Ok` value, consuming the `self` value. Panics on `Err`
    pub fn expect(self, error_str: &str) -> T {
        match self {
            Ok(t)    => t,
            Err(err) => panic!("{}\n{:?}", error_str, err)
        }
    }
}

/// Custom trait in order to impl [`Try`]. We can't overwrite the impl [`Try`] on
/// [`core::result::Result`], so a new [`Result`] type is needed
pub type Result<T> = ErrorChainResult<T>;

/// Publically export [`ErrorChainResult::Ok`] so that [`Ok`] can be used as normal 
pub use ErrorChainResult::Ok;

/// Publically export [`ErrorChainResult::Err`] so that [`Err`] can be used as normal 
pub use ErrorChainResult::Err;

/// Implement the `Context` trait in order to give `Result` the `.context()` function.
/// This function will take an error, add it to the current error chain in place, and 
/// then return the modified Result to be propagated backward.
pub trait Context<T> {
    /// Add the given [`ErrorType`] to the current [`ErrorChain`]
    fn context(self, error: &'static dyn Debug) -> Result<T>;

    // /// Add the given `str` to the current [`ErrorChain`]. This mostly has uses as adding
    // /// descriptions when handling `Error`s.
    // fn context_str<E: ErrorType>(self, error: &'static str) -> Result<T, E>;
}

/// Custom `Try` implementation to automatically add the context of the failing location
/// in source. We can't  
impl<T> core::ops::Try for ErrorChainResult<T> {
    type Ok = T;
    type Error = ErrorChain;

    fn into_result(self) -> core::result::Result<<ErrorChainResult<T> as Try>::Ok, Self::Error> {
        match self {
            ErrorChainResult::Ok(val)  => core::result::Result::Ok(val),
            ErrorChainResult::Err(val) => core::result::Result::Err(val),
        }
    }

    #[track_caller]
    fn from_error(err: Self::Error) -> Self {
        let caller = core::panic::Location::caller();

        let curr_file = caller.file();
        let curr_line = caller.line();

        // This function can be called twice when converting a `core::result::Result`
        // into an `ErrorChainResult`. This can result in the same file/line being added
        // twice to the chain. To prevent this, we check if the last element in the chain
        // is the same file/line that we are attempting to add. If so, quick return out
        // without adding a new dummy entry
        if let Some(Message { file: last_file, line: last_line, ..}) = err.last() {
            if *last_line == curr_line && *last_file == curr_file {
                return ErrorChainResult::Err(err);
            }
        }

        let err = err.extend_chain(curr_file, curr_line, &Error::Continue);
        ErrorChainResult::Err(err)
    }

    #[track_caller]
    fn from_ok(v: <ErrorChainResult<T> as Try>::Ok) -> Self {
        ErrorChainResult::Ok(v)
    }
}

impl<T> Context<T> for ErrorChainResult<T> {
    #[track_caller]
    fn context(self, error: &'static dyn Debug) -> Result<T> {
        let caller = core::panic::Location::caller();
        match self {
            ErrorChainResult::Ok(_)  => self,
            ErrorChainResult::Err(err) => {
                let err = err.extend_chain(caller.file(), caller.line(), error);
                ErrorChainResult::Err(err)
            }
        }
    }

    /*
    #[track_caller]
    fn context_str(self, error: &'static str) -> Result<T, E> {
        let caller = core::panic::Location::caller();
        match self {
            ErrorChainResult::Ok(_)  => self,
            ErrorChainResult::Err(err) => {
                // let error = ErrorType::String(error);
                let err = err.extend_chain(caller.file(), caller.line(), error);
                ErrorChainResult::Err(err)
            }
        }
    }
    */
}

impl<T> From<core::result::Result<T, ErrorChain>> for ErrorChainResult<T> {
    fn from(val: core::result::Result<T, ErrorChain>) -> ErrorChainResult<T> {
        match val {
            core::result::Result::Ok(val)  => ErrorChainResult::Ok(val),
            core::result::Result::Err(val) => ErrorChainResult::Err(val),
        }
    }
}

impl<T> Context<T> for core::result::Result<T, ErrorChain> {
    #[track_caller]
    fn context(self, error: &'static dyn Debug) -> Result<T> {
        let caller = core::panic::Location::caller();
        self.map_err(|err| err.extend_chain(caller.file(), caller.line(), error))
            .into()
    }

    /*
    #[track_caller]
    fn context_str(self, error: &'static str) -> Result<T, E> {
        let caller = core::panic::Location::caller();
        // let error = ErrorType::String(error);
        self.map_err(|err| err.extend_chain(caller.file(), caller.line(), error))
            .into()
    }
    */
}

/*
impl<T> Context<T, E> for Option<T> {
    #[track_caller]
    fn context(self, error: E) -> Result<T, E> {
        let caller = core::panic::Location::caller();
        self.ok_or_else(|| 
                ErrorChain::new_with_debug(caller.file(), caller.line(), error))
            .into()
    }

    #[track_caller]
    fn context_str(self, error: &'static str) -> Result<T, E> {
        let caller = core::panic::Location::caller();
        // let error = ErrorType::String(error);
        self.ok_or_else(|| 
                ErrorChain::new_with_debug(caller.file(), caller.line(), error))
            .into()
    }
}
*/

/*
impl From<page_table::Error> for ErrorChain {
    #[track_caller]
    fn from(err: page_table::Error) -> ErrorChain {
        let caller = core::panic::Location::caller();
        ErrorChain::new_with_debug(caller.file(), caller.line(), ErrorType::PageTable(err))
    }
}

impl From<pe::Error> for ErrorChain {
    #[track_caller]
    fn from(err: pe::Error) -> ErrorChain {
        let caller = core::panic::Location::caller();
        ErrorChain::new_with_debug(caller.file(), caller.line(), ErrorType::Pe(err))
    }
}

impl From<core::option::NoneError> for ErrorChain {
    #[track_caller]
    fn from(_err: core::option::NoneError) -> ErrorChain {
        let caller = core::panic::Location::caller();
        ErrorChain::new_with_debug(caller.file(), caller.line(), ErrorType::NoneError)
    }
}
*/

/// Shorter name for `ErrorChain::new`
#[macro_export]
macro_rules! err {
    ($i:expr) => {
        Err(ErrorChain::new($i))
    }
}

/*
/// Shorter name for `ErrorChain::new` that always uses `ErrorType::String`
#[macro_export]
macro_rules! err_str {
    ($i:expr) => {
        ErrorChain::new(ErrorType::String($i))
    }
}
*/

/// Checked addition wrapping the result in an `ErrorChain` context in case of overflow
///
/// # Example
///
/// ```
/// add!(1, u64::MAX)
/// ```
#[macro_export]
macro_rules! add {
    ($a:expr, $b:expr) => {
        $a.checked_add($b).ok_or_else(|| 
            ErrorChain::new_with_debug(file!(), line!(), &NumericalError::AddOverflow))?;
    }
}

/// Check the given condition. If the condition fails, create an `ErrorChain` with the
/// `$msg` as the first link in the chain
///
/// # Example
///
/// ```
/// ensure!(table.is_valid(), "Invalid table");
/// ```
#[macro_export]
macro_rules! ensure_str {
    ($check:expr, $msg:literal) => {
        if !$check {
            return Err(ErrorChain::new_with_debug(file!(), line!(), ErrorType::String($msg)));
        }
    };
}

#[macro_export]
macro_rules! ensure {
    ($check:expr, $err:expr) => {
        if !$check {
            return Err(ErrorChain::new_with_debug(file!(), line!(), $err));
        }
    };
}

/// Checked subtraction wrapping the result in an `ErrorChain` context in case of overflow
///
/// # Example
///
/// ```
/// sub!(0xdeadbeef, u64::MAX)
/// ```
#[macro_export]
macro_rules! sub {
    ($a:expr, $b:expr) => {
        $a.checked_sub($b).ok_or_else(|| 
            ErrorChain::new_with_debug(file!(), line!(), &NumericalError::SubUnderflow))?;
    }
}

/// Checked multiply wrapping the result in an `ErrorChain` context in case of overflow
///
/// # Example
///
/// ```
/// mul!(0xdeadbeef, u64::MAX)
/// ```
#[macro_export]
macro_rules! mul {
    ($a:expr, $b:expr) => {
        $a.checked_mul($b).ok_or_else(|| 
            ErrorChain::new_with_debug(file!(), line!(), 
                ErrorType::Arithmetic(NumericalError::MulOverflow)))?;
    }
}

/// Checked multiply wrapping the result in an `ErrorChain` context in case 
/// of overflow
///
/// # Example
///
/// ```
/// div!(0xdeadbeef, 0)
/// ```
#[macro_export]
macro_rules! div {
    ($a:expr, $b:expr) => {
        $a.checked_div($b).ok_or_else(|| 
            ErrorChain::new_with_debug(file!(), line!(), 
                ErrorType::Arithmetic(NumericalError::Div)))?;

    }
}
