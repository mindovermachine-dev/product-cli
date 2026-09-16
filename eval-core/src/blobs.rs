//! The seam a store sits on: keyed bodies, with no opinion about where.
//!
//! Three methods, deliberately. Everything that knows the format lives above
//! this line, so a second backend implements addressing rather than
//! reimplementing records — and a store swapped by configuration is a store
//! that cannot quietly disagree with the one it replaced.
//!
//! A key is a store-relative path with `/` separators. That is a file path on
//! disk and a blob name in object storage, which is why the layout was chosen
//! to be key-shaped before there was anything but disk.

use std::path::{Path, PathBuf};

use product_core::error::{ProductError, Result};

/// Somewhere keyed bodies can be put, got and listed.
pub trait Blobs: Send + Sync {
    /// Store a body under a key, returning a locator a person can follow.
    fn put(&self, key: &str, body: &str) -> Result<String>;

    /// Read a body back, or nothing when the key holds none.
    fn get(&self, key: &str) -> Result<Option<String>>;

    /// Every key under a prefix, in no guaranteed order.
    ///
    /// Returns what it can. A store is measurement, and one unreadable corner
    /// must not hide the rest of it.
    fn list(&self, prefix: &str) -> Vec<String>;
}

/// A boxed backend is still a backend.
///
/// [`crate::Backend::open`] hands back a `Box<dyn Blobs>` because which one it
/// is was decided by configuration. Without this, every caller would have to
/// know the concrete type it asked not to know.
impl<B: Blobs + ?Sized> Blobs for Box<B> {
    fn put(&self, key: &str, body: &str) -> Result<String> {
        (**self).put(key, body)
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        (**self).get(key)
    }

    fn list(&self, prefix: &str) -> Vec<String> {
        (**self).list(prefix)
    }
}

/// Keyed bodies as files under a root directory.
#[derive(Debug, Clone)]
pub struct DiskBlobs {
    root: PathBuf,
}

impl DiskBlobs {
    /// A store rooted at a directory, created on first write.
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// The root this store writes under.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// A key as a path beneath the root.
    ///
    /// A key that climbs out of the root is refused rather than normalised: a
    /// store that can be talked into writing elsewhere is not a store.
    fn path_of(&self, key: &str) -> Result<PathBuf> {
        if key.split('/').any(|segment| segment == ".." || segment.is_empty()) {
            return Err(ProductError::ConfigError(format!("`{key}` is not a key")));
        }
        Ok(self.root.join(key))
    }
}

impl Blobs for DiskBlobs {
    fn put(&self, key: &str, body: &str) -> Result<String> {
        let path = self.path_of(key)?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| ProductError::IoError(format!("{}: {e}", parent.display())))?;
        }
        std::fs::write(&path, body)
            .map_err(|e| ProductError::IoError(format!("{}: {e}", path.display())))?;
        Ok(path.display().to_string())
    }

    fn get(&self, key: &str) -> Result<Option<String>> {
        let path = self.path_of(key)?;
        match std::fs::read_to_string(&path) {
            Ok(body) => Ok(Some(body)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(ProductError::IoError(format!("{}: {e}", path.display()))),
        }
    }

    fn list(&self, prefix: &str) -> Vec<String> {
        let Ok(dir) = self.path_of(prefix) else {
            return Vec::new();
        };
        let Ok(entries) = std::fs::read_dir(&dir) else {
            return Vec::new();
        };
        entries
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().is_file())
            .filter_map(|e| e.file_name().into_string().ok())
            .map(|name| format!("{}/{name}", prefix.trim_end_matches('/')))
            .collect()
    }
}

#[path = "blobs_tests.rs"]
#[cfg(test)]
mod tests;
