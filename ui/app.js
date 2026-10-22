// LDE shell UI: theme, menu bar clock, dock.
//
// The Rust shell (src/bin/lde-shell.rs) injects `window.LDE` before this runs:
//   { theme: {...}, dock: { enabled, apps: [{ id, name, icon }] } }
// Clicking a dock item sends "launch:<id>" back through window.ipc.

const state = window.LDE || { theme: {}, dock: { enabled: true, apps: [] } };
const icons = window.LDE_ICONS || {};
const root = document.documentElement;

// ---- theme ----

function applyTheme(theme) {
  const set = (name, value) => value && root.style.setProperty(name, value);
  set("--accent", theme.accent);
  set("--ink", theme.foreground);
  set("--glass-tint", theme.bar);
  set("--wall-top", theme.wallpaper_top);
  set("--wall-bottom", theme.wallpaper_bottom);
  if (theme.radius !== undefined) {
    set("--radius-lg", `${Math.round(theme.radius * 1.5)}px`);
    set("--radius-icon", `${theme.radius}px`);
  }
}
applyTheme(state.theme || {});

// ---- clock ----

const clockEl = document.getElementById("clock");

function renderClock() {
  const now = new Date();
  const day = now.toLocaleDateString(undefined, { weekday: "short", day: "numeric", month: "short" });
  const time = now.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
  clockEl.textContent = `${day}  ${time}`;
}
renderClock();
setInterval(renderClock, 15_000);

// ---- toast ----

const toastEl = document.getElementById("toast");
let toastTimer = null;

function toast(message) {
  toastEl.textContent = message;
  toastEl.classList.add("is-visible");
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => toastEl.classList.remove("is-visible"), 1800);
}

// ---- launching ----

function launch(app) {
  if (!window.ipc) {
    toast("Launching needs the LDE shell");
    return;
  }
  window.ipc.postMessage(`launch:${app.id}`);
}

// ---- dock ----

const dockEl = document.getElementById("dock");

function buildDock(dock) {
  if (!dock || !dock.enabled) return;
  dockEl.hidden = false;

  const apps = dock.apps || [];
  apps.forEach((app, index) => {
    if (index === apps.length - 1 && apps.length > 1) {
      // Visual break before the last item, a common dock convention.
      const sep = document.createElement("span");
      sep.className = "dock-separator";
      dockEl.appendChild(sep);
    }

    const el = document.createElement("button");
    el.type = "button";
    el.className = "dock-item";
    el.dataset.id = app.id;
    el.setAttribute("aria-label", app.name);
    el.innerHTML = `
      <span class="dock-label">${app.name}</span>
      <span class="dock-icon">${icons[app.icon] || icons.app || ""}</span>`;
    el.addEventListener("click", () => launch(app));
    dockEl.appendChild(el);
  });

  // Magnify the hovered item and, a little, its neighbours.
  const items = () => [...dockEl.querySelectorAll(".dock-item")];
  dockEl.addEventListener("pointerover", (event) => {
    const hovered = event.target.closest(".dock-item");
    const list = items();
    const hoveredIndex = list.indexOf(hovered);
    list.forEach((el, i) => {
      el.classList.toggle("is-hover", el === hovered);
      el.classList.toggle("is-near", hoveredIndex !== -1 && el !== hovered && Math.abs(i - hoveredIndex) === 1);
    });
  });
  dockEl.addEventListener("pointerleave", () => {
    items().forEach((el) => el.classList.remove("is-hover", "is-near"));
  });
}

buildDock(state.dock);
