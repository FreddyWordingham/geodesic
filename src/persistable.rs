use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs,
    io::{self, Read, Write},
    path::Path,
};

/// A trait for converting back‐and‐forth between a struct and a JSON string or file.
pub trait Persistable: Serialize + DeserializeOwned + Sized {
    /// Serialize `self` to a JSON string.
    fn to_str(&self) -> serde_json::Result<String> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize an instance from a JSON string.
    fn from_str(s: &str) -> serde_json::Result<Self> {
        serde_json::from_str(s)
    }

    /// Save (serialize) `self` to the given file path (overwrites if exists).
    fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), io::Error> {
        let json = self.to_str().map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;
        let mut file = fs::File::create(path)?;
        file.write_all(json.as_bytes())
    }

    /// Load (deserialize) an instance from the given file path.
    fn load<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let mut file = fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Self::from_str(&contents).map_err(|err| io::Error::new(io::ErrorKind::Other, err))
    }
}

// Blanket‐implement for any `T: Serialize + DeserializeOwned`.
impl<T> Persistable for T where T: Serialize + DeserializeOwned + Sized {}
