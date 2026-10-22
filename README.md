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

## Status

- [x] Milestone 1: compositor window with a built-in "hello world" background and a software cursor
- [ ] Milestone 2: Wayland socket, client surfaces, xdg-shell windows
- [ ] Milestone 3: built-in panel and launcher
- [ ] Milestone 4: network, audio (PipeWire) and Bluetooth (BlueZ) services
- [ ] Milestone 5: direct TTY/DRM backend, so `start.sh` can take over the whole screen

## Layout

```
src/main.rs     event loop and rendering
src/assets.rs   built-in bitmap font and cursor
start.sh        dependency install, build, launch
```
