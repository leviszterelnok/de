//! The shell: menu bar and dock, drawn with HTML/CSS/JS in a WebKit view.
//!
//! The web UI lives in `~/LDE/core/ui`. The config from `~/LDE/config` is handed to it
//! as `window.LDE`, and it talks back through `window.ipc.postMessage`.

use std::{error::Error, path::PathBuf, thread};

use de::{config, launch, paths};
use serde_json::json;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    platform::unix::WindowExtUnix,
    window::{Fullscreen, WindowBuilder},
};
use wry::{WebViewBuilder, WebViewBuilderExtUnix};

fn main() -> Result<(), Box<dyn Error>> {
    let root = paths::root();
    let cfg = config::load_config(&root)?;
    let apps = config::load_apps(&root)?;

    let ui_index = paths::shell_ui_dir(&root).join("index.html");
    if !ui_index.exists() {
        return Err(format!("{} is missing, run ./start.sh to install the UI", ui_index.display()).into());
    }

    // Everything the UI needs, from config. The UI never reads files itself.
    let state = json!({
        "theme": cfg.theme,
        "dock": {
            "enabled": cfg.dock.enabled,
            "apps": apps.iter().filter(|a| a.dock).map(|a| json!({
                "id": a.id,
                "name": a.name,
                "icon": a.icon,
            })).collect::<Vec<_>>(),
        },
    });
    let init_script = format!("window.LDE = {};", state);

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("LDE Shell")
        .with_decorations(false)
        .with_inner_size(LogicalSize::new(1280, 800))
        .with_fullscreen(Some(Fullscreen::Borderless(None)))
        .build(&event_loop)?;

    let url = format!("file://{}", ui_index.display());
    let root_for_ipc = root.clone();
    let apps_for_ipc = apps.clone();

    let webview = WebViewBuilder::new()
        .with_url(&url)
        .with_initialization_script(&init_script)
        .with_ipc_handler(move |request| {
            let message = request.body();
            if let Some(id) = message.strip_prefix("launch:") {
                launch_app(&root_for_ipc, &apps_for_ipc, id);
            }
        })
        .build_gtk(window.default_vbox().ok_or("no GTK container for the window")?)?;

    // Keep the webview alive for the lifetime of the event loop.
    let _webview = webview;

    event_loop.run(move |event, _target, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = ControlFlow::Exit;
        }
    });
}

/// Starts an app from its config. The child is waited on in its own thread,
/// so exited apps do not stay behind as zombies.
fn launch_app(root: &PathBuf, apps: &[config::App], id: &str) {
    let Some(app) = config::find_app(apps, id) else {
        eprintln!("unknown app {id:?}");
        return;
    };
    let socket = std::env::var_os("WAYLAND_DISPLAY");
    match launch::spawn_app(root, app, socket.as_deref()) {
        Ok(mut child) => {
            thread::spawn(move || {
                let _ = child.wait();
            });
        }
        Err(err) => eprintln!("could not start {}: {err}", app.name),
    }
}
