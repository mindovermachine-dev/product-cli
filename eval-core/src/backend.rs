//! Which store a tool writes to, chosen by configuration rather than by code.
//!
//! The point of naming the backends in one enum is that moving from disk to
//! object storage is an edit to configuration, not to a call site. Every tool
//! resolves its store the same way, so they move together or not at all.

use std::path::PathBuf;

use product_core::error::{ProductError, Result};
use serde::{Deserialize, Serialize};

use crate::blobs::{Blobs, DiskBlobs};

/// Where a tool's evaluation store lives.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Backend {
    /// Files under a directory in the working tree.
    Disk {
        /// The store root. Relative paths resolve against the repo.
        root: PathBuf,
    },
    /// Blobs in an Azure storage container.
    ///
    /// Named here before it is built, so adopting it is a configuration change
    /// and not a redesign. [`Backend::open`] says plainly that it is not built
    /// rather than falling back to disk — a tool that silently writes somewhere
    /// other than where it was told is worse than one that stops.
    Azure {
        /// The storage account.
        account: String,
        /// The container within it.
        container: String,
        /// A key prefix inside the container, so one container holds several tools.
        #[serde(default)]
        prefix: String,
    },
}

/// The variable naming the backend, for a tool configured from the environment.
pub const BACKEND_ENV: &str = "EVAL_STORE";

impl Backend {
    /// Disk, under a repo-relative directory.
    pub fn disk(root: impl Into<PathBuf>) -> Self {
        Self::Disk { root: root.into() }
    }

    /// Read the backend from the environment, falling back to a disk default.
    ///
    /// `EVAL_STORE` is either a path — taken as a disk root — or
    /// `azure:<account>/<container>[/<prefix>]`. Anything else is refused
    /// rather than guessed at.
    pub fn from_env(default_root: impl Into<PathBuf>) -> Result<Self> {
        let Some(raw) = std::env::var_os(BACKEND_ENV) else {
            return Ok(Self::disk(default_root));
        };
        let raw = raw.to_string_lossy().into_owned();
        Self::parse(&raw)
    }

    /// Parse the spelling `EVAL_STORE` uses.
    pub fn parse(raw: &str) -> Result<Self> {
        let Some(rest) = raw.strip_prefix("azure:") else {
            return Ok(Self::disk(raw));
        };
        let mut parts = rest.splitn(3, '/');
        match (parts.next(), parts.next(), parts.next()) {
            (Some(account), Some(container), prefix)
                if !account.is_empty() && !container.is_empty() =>
            {
                Ok(Self::Azure {
                    account: account.to_string(),
                    container: container.to_string(),
                    prefix: prefix.unwrap_or_default().to_string(),
                })
            }
            _ => Err(ProductError::ConfigError(format!(
                "`{raw}` is not a store; use a path, or azure:<account>/<container>[/<prefix>]"
            ))),
        }
    }

    /// Open the backend, or say why it could not be opened.
    pub fn open(&self) -> Result<Box<dyn Blobs>> {
        match self {
            Self::Disk { root } => Ok(Box::new(DiskBlobs::new(root.clone()))),
            Self::Azure { account, container, .. } => Err(ProductError::ConfigError(format!(
                "the azure backend is declared but not built — \
                 configured for {account}/{container}.\n  \
                 The store's shape does not change with it: the same keys, the same records. \
                 Until it is built, set {BACKEND_ENV} to a path."
            ))),
        }
    }

    /// What to call this backend in a log line.
    pub fn describe(&self) -> String {
        match self {
            Self::Disk { root } => format!("disk at {}", root.display()),
            Self::Azure { account, container, prefix } if prefix.is_empty() => {
                format!("azure {account}/{container}")
            }
            Self::Azure { account, container, prefix } => {
                format!("azure {account}/{container}/{prefix}")
            }
        }
    }
}

#[path = "backend_tests.rs"]
#[cfg(test)]
mod tests;
