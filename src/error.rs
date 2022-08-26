use std::ffi::NulError;
use thiserror::Error;

// General error that haven't determined what this is.
// This might be an internal error, or might be something that should be exposed out of library in Rust interface.
// TODO: Proper error structure when needed. Exposed ones and internal ones might have to be different
#[derive(Error, Debug)]
pub enum CskkError {
    #[error("Some kind of error: {0}")]
    Error(String),
    #[error("Rule error: {0}")]
    RuleError(String),
    #[error("Failed to parse: {0}")]
    ParseError(String),
    #[error(transparent)]
    TomlFileLoadError {
        #[from]
        source: toml::de::Error,
    },
    #[error(transparent)]
    IoError {
        #[from]
        source: std::io::Error,
    },
    #[error(transparent)]
    XDGBaseDirectoryError {
        #[from]
        source: xdg::BaseDirectoriesError,
    },
    #[error(transparent)]
    InfallibleError {
        // Error that never can happens.
        #[from]
        source: core::convert::Infallible,
    },
    #[error(transparent)]
    FFIError {
        #[from]
        source: NulError,
    },
}
