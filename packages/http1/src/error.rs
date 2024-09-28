use std::error::Error;

/// An error type.
pub type BoxError = Box<dyn Error + Send + Sync + 'static>;
