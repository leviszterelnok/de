//! Starting apps from their config, always inside the desktop's environment.

use std::{
    ffi::OsStr,
    io,
    path::Path,
    process::{Child, Command},
};

use crate::{config::App, paths};

/// Starts `app`. `socket` is the Wayland display the app should connect to.
///
/// The app gets the XDG directories under `~/LDE/user`, and `DISPLAY` is removed
/// so that toolkits pick Wayland instead of a host X server.
pub fn spawn_app(root: &Path, app: &App, socket: Option<&OsStr>) -> io::Result<Child> {
    let expand = |s: &str| s.replace("$LDE", &root.to_string_lossy());

    let (program, args) = app
        .exec
        .split_first()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, format!("{}: empty exec", app.id)))?;

    let mut command = Command::new(expand(program));
    command.args(args.iter().map(|a| expand(a)));
    command.env_remove("DISPLAY");

    for (key, dir) in paths::user_env(root) {
        command.env(key, dir);
    }
    if let Some(socket) = socket {
        command.env("WAYLAND_DISPLAY", socket);
    }
    for (key, value) in &app.env {
        command.env(key, expand(value));
    }

    command.spawn()
}
