use thiserror::Error;

/// Specific error type for file parsing operations.
#[derive(Error, Debug)]
pub enum FileParsingError {
    #[error("Invalid OBJ file format: {message}")]
    InvalidObjFormat { message: String },

    #[error("Missing vertex position data at line {line}")]
    MissingVertexPosition { line: usize },

    #[error("Missing vertex normal data at line {line}")]
    MissingVertexNormal { line: usize },

    #[error("Invalid face data at line {line}: {message}")]
    InvalidFaceData { line: usize, message: String },

    #[error("Invalid coordinate value '{value}' at line {line}")]
    InvalidCoordinate { value: String, line: usize },

    #[error("File not found: {path}")]
    FileNotFound { path: String },
}
