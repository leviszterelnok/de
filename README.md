# de

A Wayland desktop environment written in Rust on top of [smithay](https://github.com/Smithay/smithay).
Everything is built into this repository: no config language, no external panels, no webview.

## Run

```sh
git clone https://github.com/leviszterelnok/de
cd de
./start.sh
```

`start.sh` installs the system libraries it needs (apt, pacman or dnf), installs Rust through rustup if
it is missing, builds the project and launches it.

- **Inside a running session:** the DE opens as a window.
- **No running session:** it requests a fullscreen window.

Keys:

- `Super+Return` opens a [kitty](https://sw.kovidgoyal.net/kitty/) terminal. One is also started at launch.

## Status

- [x] Milestone 1: compositor window with a built-in "hello world" background and a software cursor
- [x] Milestone 2: Wayland socket, xdg-shell windows, keyboard and pointer input, kitty launches inside the DE
- [ ] Milestone 2b: more windows (tiling or stacking), focus by click, resize handling
- [x] Milestone 3a: web-based shell (menu bar, dock) with Apple-style glass UI, themed from ~/LDE/config
- [ ] Milestone 3b: panel as a layer-shell surface instead of a window, launcher
- [ ] Milestone 4: network, audio (PipeWire) and Bluetooth (BlueZ) services
- [ ] Milestone 5: direct TTY/DRM backend, so `start.sh` can take over the whole screen

## Layout

Everything the desktop uses lives under `~/LDE` (override with `LDE_HOME`):

```
~/LDE/
  config/           all settings: lde.toml (theme, terminal, dock) and apps/*.toml
  core/             the desktop's own parts: de, lde-shell, ui/ (installed by start.sh)
  user/             your data; apps get XDG_CONFIG/DATA/CACHE/STATE pointed here
  apps/             installed applications
```

`HOME` itself is not moved, because ssh, git and shells depend on it. The XDG variables cover
most apps.

## Source layout

```
src/main.rs       compositor: event loop, input, rendering, starts the shell and terminal
src/state.rs      compositor state and Wayland protocol handlers
src/assets.rs     built-in bitmap font and cursor
src/lib.rs        shared code: paths, config, app launching
src/config.rs     reads ~/LDE/config
src/paths.rs      the ~/LDE layout
src/launch.rs     starts apps with the desktop's environment
src/bin/lde-shell.rs  the shell window (WebKit, built with --features shell)
ui/               the shell's HTML/CSS/JS, installed to ~/LDE/core/ui
defaults/config/  default configuration written on first run
start.sh          dependency install, build, install into ~/LDE/core, run
```
