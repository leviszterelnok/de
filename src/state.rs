//! Compositor state and the Wayland protocol handlers it implements.

use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_seat, delegate_shm, delegate_xdg_shell,
    input::{pointer::CursorImageStatus, Seat, SeatHandler, SeatState},
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::{wl_buffer, wl_seat, wl_surface::WlSurface},
            Client,
        },
    },
    utils::{Logical, Serial, Size, SERIAL_COUNTER},
    wayland::{
        buffer::BufferHandler,
        compositor::{CompositorClientState, CompositorHandler, CompositorState},
        shell::xdg::{PopupSurface, PositionerState, ToplevelSurface, XdgShellHandler, XdgShellState},
        shm::{ShmHandler, ShmState},
    },
};

/// Per-client data required by the compositor protocol.
#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}

    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

pub struct State {
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub seat_state: SeatState<Self>,
    pub seat: Seat<Self>,

    /// Toplevel windows, oldest first. The last one is on top and has focus.
    pub windows: Vec<ToplevelSurface>,
    /// Size of the output, in logical pixels.
    pub output_size: Size<i32, Logical>,
    /// Last known pointer position in compositor space.
    pub pointer_location: smithay::utils::Point<f64, Logical>,
    /// Set by the keybinding filter, handled by the main loop.
    pub spawn_requested: bool,
}

impl State {
    /// Gives keyboard focus to the topmost window, if there is one.
    pub fn focus_top_window(&mut self) {
        let Some(keyboard) = self.seat.get_keyboard() else {
            return;
        };
        let focus = self.windows.last().map(|w| w.wl_surface().clone());
        keyboard.set_focus(self, focus, SERIAL_COUNTER.next_serial());
    }

    /// Sends the current output size to every window.
    fn configure_all(&self) {
        for window in &self.windows {
            window.with_pending_state(|state| state.size = Some(self.output_size));
            window.send_pending_configure();
        }
    }

    pub fn set_output_size(&mut self, size: Size<i32, Logical>) {
        self.output_size = size;
        self.configure_all();
    }
}

impl XdgShellHandler for State {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        // Every window is full screen for now, so the user only sees one at a time.
        let size = self.output_size;
        surface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Activated);
            state.size = Some(size);
        });
        surface.send_configure();
        self.windows.push(surface);
        self.focus_top_window();
    }

    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        self.windows.retain(|w| w.wl_surface() != surface.wl_surface());
        self.focus_top_window();
    }

    fn new_popup(&mut self, _surface: PopupSurface, _positioner: PositionerState) {}

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {}

    fn reposition_request(&mut self, _surface: PopupSurface, _positioner: PositionerState, _token: u32) {}
}

impl CompositorHandler for State {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
    }
}

impl BufferHandler for State {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl ShmHandler for State {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

impl SeatHandler for State {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<Self> {
        &mut self.seat_state
    }

    fn focus_changed(&mut self, _seat: &Seat<Self>, _focused: Option<&WlSurface>) {}

    fn cursor_image(&mut self, _seat: &Seat<Self>, _image: CursorImageStatus) {}
}

delegate_xdg_shell!(State);
delegate_compositor!(State);
delegate_shm!(State);
delegate_seat!(State);
