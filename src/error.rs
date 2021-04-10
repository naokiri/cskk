use thiserror::Error;

// General error that haven't determined what this is.
// This might be an internal error, or might be something that should be exposed out of library in Rust interface.
// TODO: Proper error structure when needed. Exposed ones and internal ones might have to be different
#[derive(Error, Debug)]
pub enum CskkError {
    #[error("Some kind of error: {0}")]
    Error(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
