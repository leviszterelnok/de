//! Reads `~/LDE/config`. Every setting the desktop has lives there.

use std::{collections::BTreeMap, fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::paths;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Config {
    pub theme: Theme,
    pub session: Session,
    pub dock: Dock,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Theme {
    /// Wallpaper gradient, from top to bottom.
    pub wallpaper_top: String,
    pub wallpaper_bottom: String,
    /// Main accent colour, used for selection and highlights.
    pub accent: String,
    /// Text on top of the wallpaper and the bars.
    pub foreground: String,
    /// Background of the menu bar and dock, with alpha, e.g. "rgba(20, 20, 22, 0.55)".
    pub bar: String,
    /// Corner radius of dock icons and windows, in pixels.
    pub radius: u32,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            wallpaper_top: "#1b2a36".into(),
            wallpaper_bottom: "#0b0c0e".into(),
            accent: "#1e88c7".into(),
            foreground: "#f2f5f8".into(),
            bar: "rgba(20, 22, 26, 0.55)".into(),
            radius: 14,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct Session {
    /// App id (file name without `.toml`) opened by Super+Return and at startup.
    pub terminal: String,
}

impl Default for Session {
    fn default() -> Self {
        Self {
            terminal: "kitty".into(),
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct Dock {
    /// Show the dock at all.
    pub enabled: bool,
}

/// One app, from `config/apps/<id>.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct App {
    /// Taken from the file name, not from the file.
    #[serde(skip)]
    pub id: String,
    pub name: String,
    /// Command and arguments. `$LDE` expands to the desktop root.
    pub exec: Vec<String>,
    /// Name of a built-in icon, see `ui/icons.js`.
    #[serde(default = "default_icon")]
    pub icon: String,
    /// Show in the dock.
    #[serde(default)]
    pub dock: bool,
    /// Extra environment variables for the app.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

fn default_icon() -> String {
    "app".into()
}

pub fn load_config(root: &Path) -> io::Result<Config> {
    let path = paths::config_dir(root).join("lde.toml");
    let text = fs::read_to_string(&path)?;
    toml::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {e}", path.display())))
}

/// Loads every `config/apps/*.toml`, sorted by id.
pub fn load_apps(root: &Path) -> io::Result<Vec<App>> {
    let mut apps = Vec::new();
    for entry in fs::read_dir(paths::apps_config_dir(root))? {
        let path = entry?.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        let Some(id) = path.file_stem().and_then(|s| s.to_str()).map(str::to_owned) else {
            continue;
        };
        let text = fs::read_to_string(&path)?;
        let mut app: App = toml::from_str(&text)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("{}: {e}", path.display())))?;
        app.id = id;
        apps.push(app);
    }
    apps.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(apps)
}

pub fn find_app<'a>(apps: &'a [App], id: &str) -> Option<&'a App> {
    apps.iter().find(|app| app.id == id)
}
