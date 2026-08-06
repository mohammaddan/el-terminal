//! Startup CLI options (`--working-directory` / `--dir`, `--command` / `-e`, `--new-window`).

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

#[derive(Clone, Debug, Default)]
pub struct LaunchOptions {
    /// Absolute (or best-effort absolute) directory for the first PTY.
    pub working_directory: Option<String>,
    pub command: Option<String>,
    /// Accepted for CLI compatibility; every launch is already a new window.
    pub new_window: bool,
}

static OPTIONS: OnceLock<LaunchOptions> = OnceLock::new();

pub fn store(opts: LaunchOptions) {
    let _ = OPTIONS.set(opts);
}

pub fn take() -> LaunchOptions {
    OPTIONS.get().cloned().unwrap_or_default()
}

/// Resolve a user-supplied directory to an absolute path suitable for VTE.
///
/// Prefer `canonicalize` (follows symlinks). If that fails but the path is a
/// directory, fall back to making it absolute without resolving links.
pub fn normalize_working_directory(dir: &str) -> Result<String, String> {
    let path = Path::new(dir);
    let absolute: PathBuf = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("cannot resolve relative path: {e}"))?
            .join(path)
    };

    if let Ok(canon) = absolute.canonicalize() {
        if canon.is_dir() {
            return path_to_string(canon);
        }
        return Err(format!("not a directory: {}", canon.display()));
    }

    if absolute.is_dir() {
        return path_to_string(absolute);
    }

    Err(format!("working directory is not a directory: {dir}"))
}

fn path_to_string(path: PathBuf) -> Result<String, String> {
    path.into_os_string()
        .into_string()
        .map_err(|_| "working directory path is not valid UTF-8".into())
}
