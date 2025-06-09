use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::File,
    io::{self, Read, Write},
    path::Path,
};

/// A trait for (de)serializing a struct to/from JSON.
pub trait Persistable: Serialize + DeserializeOwned + Sized {
    /// Serialize `self` to a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the object cannot be serialized to JSON.
    fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize an instance from a JSON string.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON string is invalid or does not match the expected structure.
    fn from_json(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    /// Save (serialize) `self` to the given file path (overwrites if exists).
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be created, written to, or the object cannot be serialized to JSON.
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let json = self.to_json().map_err(io::Error::other)?;
        let mut file = File::create(path)?;
        file.write_all(json.as_bytes())
    }

    /// Load (deserialize) an instance from the given file path.
    ///
    /// # Errors
    ///
    /// Returns an error if the file does not exist, cannot be read, or the contents are not valid JSON.
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        let _num_bytes_read = file.read_to_string(&mut contents)?;
        Self::from_json(&contents).map_err(io::Error::other)
    }
}

// Automatic blanket implementation for any types satisfying the trait bounds.
impl<T> Persistable for T where T: Serialize + DeserializeOwned + Sized {}
