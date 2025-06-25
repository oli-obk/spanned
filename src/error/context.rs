use std::{fmt::Display, path::Path};

use crate::{Error, Span, Spanned};

pub trait Context<T> {
    fn with_context<U: Display + Send + Sync + 'static>(
        self,
        f: impl FnOnce() -> Spanned<U>,
    ) -> Result<T, Error>;
    fn with_path_context<U: Display + Send + Sync + 'static>(
        self,
        path: &Path,
        msg: U,
    ) -> Result<T, Error>;
    fn with_str_context<U: Display + Send + Sync + 'static>(self, msg: U) -> Result<T, Error>;
}

impl<T, E: std::error::Error + Send + Sync + 'static> Context<T> for Result<T, E> {
    #[track_caller]
    fn with_context<U: Display + Send + Sync + 'static>(
        self,
        f: impl FnOnce() -> Spanned<U>,
    ) -> Result<T, Error> {
        self.map_err(|e| Error::new_str(f()).wrap(Spanned::here(e)))
    }

    #[track_caller]
    fn with_path_context<U: Display + Send + Sync + 'static>(
        self,
        path: &Path,
        msg: U,
    ) -> Result<T, Error> {
        self.map_err(|e| {
            Error::new_str(Spanned::new(msg, Span::new(path, 0..0))).wrap(Spanned::here(e))
        })
    }
    #[track_caller]
    fn with_str_context<U: Display + Send + Sync + 'static>(self, msg: U) -> Result<T, Error> {
        self.map_err(|e| Error::new_str(Spanned::here(msg)).wrap(Spanned::here(e)))
    }
}
