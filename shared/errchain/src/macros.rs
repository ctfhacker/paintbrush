//! Various macros for helping wrap `Option` into `errchain::Result`

use super::*;

/*
/// Checked addition wrapping the result in an [`ErrorChain`] context in case of overflow
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
            ErrorChain::new_with_debug(file!(), line!(), 
                ErrorType::Arithmetic(NumericalError::AddOverflow)))?;
    }
}
*/

