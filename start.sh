#!/usr/bin/env bash
# Build and launch the DE.
#
#   ./start.sh          build (if needed) and run
#   ./start.sh --deps   only install the system libraries
#
# If a Wayland/X session is already running, the DE opens as a window inside it.
# Otherwise it asks for a fullscreen window. A bare TTY/DRM backend is not written yet.
# Super+Return opens a kitty terminal inside the DE.
set -euo pipefail
cd "$(dirname "$0")"

log() { printf '==> %s\n' "$*"; }
die() { printf 'error: %s\n' "$*" >&2; exit 1; }

install_system_deps() {
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get install -y build-essential pkg-config libwayland-dev libxkbcommon-dev \
            libudev-dev libinput-dev libgbm-dev libseat-dev libegl-dev libgles-dev \
            libdrm-dev libxcursor-dev
    elif command -v pacman >/dev/null 2>&1; then
        sudo pacman -S --needed --noconfirm base-devel pkgconf wayland libxkbcommon \
            systemd-libs libinput mesa seatd libdrm libxcursor
    elif command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y gcc pkgconf-pkg-config wayland-devel libxkbcommon-devel \
            systemd-devel libinput-devel mesa-libgbm-devel libseat-devel \
            mesa-libEGL-devel mesa-libGLES-devel libdrm-devel libXcursor-devel
    else
        die "unknown package manager: install wayland, libxkbcommon, libinput, libseat, libdrm and mesa dev packages by hand"
    fi
}

install_terminal() {
    if command -v apt-get >/dev/null 2>&1; then
        sudo apt-get install -y kitty
    elif command -v pacman >/dev/null 2>&1; then
        sudo pacman -S --needed --noconfirm kitty
    elif command -v dnf >/dev/null 2>&1; then
        sudo dnf install -y kitty
    fi
}

ensure_system_deps() {
    local pkgs=(wayland-server xkbcommon libinput libseat libudev gbm egl glesv2 libdrm xcursor)
    if ! pkg-config --exists "${pkgs[@]}" 2>/dev/null; then
        log "installing system libraries"
        install_system_deps
    fi
    if ! command -v kitty >/dev/null 2>&1; then
        log "installing kitty"
        install_terminal
    fi
}

ensure_rust() {
    if [[ -f "$HOME/.cargo/env" ]]; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
    if ! command -v cargo >/dev/null 2>&1; then
        log "installing Rust via rustup"
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env"
    fi
}

if [[ "${1:-}" == "--deps" ]]; then
    ensure_system_deps
    exit 0
fi

ensure_system_deps
ensure_rust

log "building"
cargo build --release

if [[ -n "${WAYLAND_DISPLAY:-}" || -n "${DISPLAY:-}" ]]; then
    log "running as a window inside the current session"
    exec ./target/release/de
else
    log "no running session found, requesting fullscreen"
    exec ./target/release/de --fullscreen
fi
