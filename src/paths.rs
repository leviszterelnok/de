//! The on-disk layout of the desktop.
//!
//! ```text
//! ~/LDE/
//!   config/          all settings, one source of truth
//!     lde.toml       theme, session, dock
//!     apps/*.toml    one file per app: how to start it, where it shows up
//!   core/            things the system itself needs (shell UI, logs)
//!   user/            the user's data; apps are pointed here through XDG_* variables
//!   apps/            installed applications
//! ```
//!
//! `LDE_HOME` overrides the root, which is handy for testing.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub const DEFAULT_CONFIG: &str = include_str!("../defaults/config/lde.toml");
pub const DEFAULT_KITTY: &str = include_str!("../defaults/config/apps/kitty.toml");
pub const DEFAULT_FILES: &str = include_str!("../defaults/config/apps/files.toml");

/// The root of the desktop, `~/LDE` unless `LDE_HOME` is set.
pub fn root() -> PathBuf {
    if let Some(custom) = std::env::var_os("LDE_HOME") {
        return PathBuf::from(custom);
    }
    let home = std::env::var_os("HOME").unwrap_or_else(|| "/root".into());
    PathBuf::from(home).join("LDE")
}

pub fn config_dir(root: &Path) -> PathBuf {
    root.join("config")
}

pub fn apps_config_dir(root: &Path) -> PathBuf {
    root.join("config").join("apps")
}

pub fn core_dir(root: &Path) -> PathBuf {
    root.join("core")
}

pub fn shell_ui_dir(root: &Path) -> PathBuf {
    root.join("core").join("ui")
}

pub fn user_dir(root: &Path) -> PathBuf {
    root.join("user")
}

pub fn installed_apps_dir(root: &Path) -> PathBuf {
    root.join("apps")
}

/// Environment variables that point XDG-aware programs at `~/LDE/user`.
///
/// Redirecting `HOME` itself would break ssh, git, shells and most tooling,
/// so only the XDG base directories are moved.
pub fn user_env(root: &Path) -> Vec<(&'static str, PathBuf)> {
    let user = user_dir(root);
    vec![
        ("XDG_CONFIG_HOME", user.join("config")),
        ("XDG_DATA_HOME", user.join("data")),
        ("XDG_CACHE_HOME", user.join("cache")),
        ("XDG_STATE_HOME", user.join("state")),
    ]
}

/// Creates the layout and writes default configuration where none exists yet.
/// Existing files are never overwritten.
pub fn ensure_layout(root: &Path) -> io::Result<()> {
    let dirs = [
        config_dir(root),
        apps_config_dir(root),
        core_dir(root).join("logs"),
        user_dir(root).join("config"),
        user_dir(root).join("data"),
        user_dir(root).join("cache"),
        user_dir(root).join("state"),
        installed_apps_dir(root),
    ];
    for dir in dirs {
        fs::create_dir_all(dir)?;
    }

    write_if_missing(&config_dir(root).join("lde.toml"), DEFAULT_CONFIG)?;
    write_if_missing(&apps_config_dir(root).join("kitty.toml"), DEFAULT_KITTY)?;
    write_if_missing(&apps_config_dir(root).join("files.toml"), DEFAULT_FILES)?;
    Ok(())
}

fn write_if_missing(path: &Path, contents: &str) -> io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    fs::write(path, contents)
}
