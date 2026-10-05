//! The inbox configuration: the local clones a holder reviews over.
//!
//! ```yaml
//! repositories:
//!   - name: billing            # the label rows and the batch file carry
//!     path: ../billing         # a local clone; relative to this file
//!     remote: origin           # optional, default `origin`
//!     default_branch: main     # optional; else the remote's HEAD, else `main`
//! ```

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Every clone the inbox reads.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub repositories: Vec<Repository>,
}

/// One clone.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repository {
    pub name: String,
    pub path: PathBuf,
    #[serde(default = "origin")]
    pub remote: String,
    #[serde(default)]
    pub default_branch: Option<String>,
}

fn origin() -> String {
    "origin".to_string()
}

/// Read the configuration, resolving clone paths against the file's own
/// directory. Two clones under one name, or none at all, is refused.
pub fn load(path: &Path) -> Result<Config, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut config: Config = serde_yaml::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    if config.repositories.is_empty() {
        return Err(format!("{}: lists no repository", path.display()));
    }
    let base = path.parent().unwrap_or(Path::new("."));
    let mut names = std::collections::BTreeSet::new();
    for r in &mut config.repositories {
        if !names.insert(r.name.clone()) {
            return Err(format!("{}: repository `{}` is listed twice", path.display(), r.name));
        }
        if r.path.is_relative() {
            r.path = base.join(&r.path);
        }
    }
    Ok(config)
}

/// Where the configuration lives when no `--config` is given.
pub fn default_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("ledger").join("inbox.yml")
}
