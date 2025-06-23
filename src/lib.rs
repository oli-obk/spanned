mod error;
mod span;

pub use error::*;
pub use span::*;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[macro_export]
macro_rules! bail {
    ($span:expr, $msg:expr) => {
        return Err($crate::Error::str("error reported here").wrap_str($crate::Spanned::as_ref(&$span).map(|_| format!($msg))));
    };

    ($span:expr, $fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::str("error reported here").wrap_str($crate::Spanned::as_ref(&$span).map(|_| format!($fmt, $($arg)*))));
    };
}

#[macro_export]
macro_rules! bail_here {
    ($msg:expr) => {
        return Err($crate::Error::str(format!($msg)));
    };

    ($fmt:expr, $($arg:tt)*) => {
        return Err($crate::Error::str(format!($fmt, $($arg)*)));
    };
}
