use thiserror::Error;

#[derive(Error, Debug)]
pub enum NumericError {
    #[error("Type conversion failed: cannot convert {from_type} to {to_type}")]
    TypeConversion { from_type: String, to_type: String },

    #[error("Numeric bounds unavailable for type {type_name}")]
    BoundsUnavailable { type_name: String },

    #[error("Arithmetic operation failed: {operation}")]
    ArithmeticFailed { operation: String },
}
