use thiserror::Error;

#[derive(Error, Debug)]
pub enum CskkError {
    // General error that haven't determined what this is.
    // This might be an internal error, or might be something that should be exposed out of library in Rust interface.
    // TODO: Proper error structure when needed.
    #[error("Some kind of error: {0}")]
    Error(String),
}