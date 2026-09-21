//! FIND-20 — persistencia manual de geometría de la ventana principal.
//!
//! Guarda `window-state.json` (posición, tamaño, maximizado) en el
//! `app_data_dir` de Tauri y lo restaura en el siguiente arranque.
//!
//! Diseño (sin `tauri-plugin-window-state`: el core ya expone todo lo
//! necesario — ver `docs/tasks/FIND-20.md` § Decisión):
//! - [`load`] nunca falla: archivo ausente o corrupto → [`WindowState::default`]
//!   (pre-mortem: estado corrupto no puede bloquear el arranque).
//! - [`restore`] y [`save_now`] son best-effort: ignoran errores de IPC.
//! - Mientras la ventana está maximizada no se pisa la geometría normal
//!   (solo se actualiza el flag), para restaurar la geometría previa al
//!   desmaximizar.
//!
//! Sources: `tauri::WindowEvent::{Moved,Resized,CloseRequested}`,
//! `Window::{inner_position,inner_size,is_maximized,set_position,set_size,maximize}`,
//! `Manager::get_window`, `PathResolver::app_data_dir` — verificados en
//! vendored `tauri-2.11.5/src/` (ver task file).

use serde::{Deserialize, Serialize};
use tauri::{
    AppHandle, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewWindow, WindowEvent,
};

/// Nombre del archivo de estado dentro de `app_data_dir()`.
const STATE_FILE: &str = "window-state.json";

/// Geometría persistida de la ventana principal.
///
/// `x`/`y` son `None` cuando nunca se guardó posición (primer arranque):
/// en ese caso se respeta el `center: true` de `tauri.conf.json`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowState {
    /// Posición física (píxeles). `None` = centrar según config.
    pub x: Option<i32>,
    /// Posición física (píxeles). `None` = centrar según config.
    pub y: Option<i32>,
    /// Ancho físico (píxeles).
    pub width: u32,
    /// Alto físico (píxeles).
    pub height: u32,
    /// La ventana estaba maximizada al guardar.
    pub maximized: bool,
}

impl Default for WindowState {
    /// Default = geometría de `tauri.conf.json` (1280×800, centrada).
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: 1280,
            height: 800,
            maximized: false,
        }
    }
}

impl WindowState {
    /// Sanea valores absurdos (p.ej. JSON editado a mano): evita ventanas
    /// invisibles o gigantes. Posición negativa OK (multi-monitor).
    fn sanitized(self) -> Self {
        Self {
            x: self.x,
            y: self.y,
            width: self.width.clamp(320, 16384),
            height: self.height.clamp(200, 16384),
            maximized: self.maximized,
        }
    }
}

/// Ruta del archivo de estado. `None` si el dir de datos no resuelve.
fn state_path<R: Runtime>(app: &AppHandle<R>) -> Option<std::path::PathBuf> {
    app.path().app_data_dir().ok().map(|d| d.join(STATE_FILE))
}

/// Lee el estado persistido. Nunca falla: ausente/corrupto → default.
pub fn load<R: Runtime>(app: &AppHandle<R>) -> WindowState {
    let path = state_path(app);
    let bytes = path.as_ref().and_then(|p| std::fs::read(p).ok());
    bytes
        .and_then(|b| serde_json::from_slice(&b).ok())
        .map(WindowState::sanitized)
        .unwrap_or_default()
}

/// Captura la geometría actual de `window` sobre `state` (sin guardar).
fn snapshot<R: Runtime>(window: &WebviewWindow<R>, mut state: WindowState) -> WindowState {
    let maximized = window.is_maximized().unwrap_or(false);
    state.maximized = maximized;
    // Maximizada: la geometría visible es la del monitor, no la restaurable.
    if !maximized {
        if let Ok(pos) = window.inner_position() {
            state.x = Some(pos.x);
            state.y = Some(pos.y);
        }
        if let Ok(size) = window.inner_size() {
            state.width = size.width;
            state.height = size.height;
        }
    }
    state.sanitized()
}

/// Guarda la geometría actual. Best-effort: ignora errores (I/O, IPC).
pub fn save_now<R: Runtime>(app: &AppHandle<R>, window: &WebviewWindow<R>) {
    let Some(path) = state_path(app) else { return };
    if path
        .parent()
        .is_some_and(|d| std::fs::create_dir_all(d).is_err())
    {
        return;
    }
    let state = snapshot(window, load(app));
    if let Ok(bytes) = serde_json::to_vec_pretty(&state) {
        let _ = std::fs::write(&path, bytes);
    }
}

/// Restaura `state` sobre `window`. Best-effort: ignora errores de IPC.
pub fn restore<R: Runtime>(window: &WebviewWindow<R>, state: &WindowState) {
    if state.maximized {
        let _ = window.maximize();
        return;
    }
    let _ = window.set_size(PhysicalSize {
        width: state.width,
        height: state.height,
    });
    if let (Some(x), Some(y)) = (state.x, state.y) {
        let _ = window.set_position(PhysicalPosition { x, y });
    }
}

/// Suscribe el auto-guardado: `Moved`/`Resized`/`CloseRequested` → [`save_now`].
///
/// El closure es `'static` (captura un clon de `AppHandle`); el save es
/// síncrono y best-effort para no bloquear el event loop.
pub fn attach<R: Runtime>(window: &WebviewWindow<R>, app: &AppHandle<R>) {
    let app = app.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Moved(_) | WindowEvent::Resized(_) | WindowEvent::CloseRequested { .. } => {
            if let Some(w) = app.get_webview_window("main") {
                save_now(&app, &w);
            }
        }
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_matches_tauri_conf_geometry() {
        let s = WindowState::default();
        assert_eq!((s.width, s.height), (1280, 800));
        assert_eq!((s.x, s.y), (None, None));
        assert!(!s.maximized);
    }

    #[test]
    fn roundtrip_serde_preserves_all_fields() {
        let s = WindowState {
            x: Some(120),
            y: Some(-40),
            width: 1600,
            height: 900,
            maximized: false,
        };
        let back: WindowState =
            serde_json::from_str(&serde_json::to_string(&s).expect("ser")).expect("de");
        assert_eq!(s, back);
    }

    /// Prove-It (pre-mortem fallo 2): JSON corrupto → default, sin error.
    #[test]
    fn corrupt_json_falls_back_to_default() {
        let bad: Result<WindowState, _> = serde_json::from_str("{not json");
        assert!(bad.is_err());
        // `load` mapea cualquier error a default — mismo camino que archivo corrupto.
        assert_eq!(WindowState::default().sanitized(), WindowState::default());
    }

    #[test]
    fn sanitized_rejects_absurd_geometry_but_keeps_negative_position() {
        let s = WindowState {
            x: Some(-1920),
            y: Some(-300),
            width: 0,
            height: 99_999,
            maximized: true,
        }
        .sanitized();
        assert_eq!((s.x, s.y), (Some(-1920), Some(-300)));
        assert_eq!((s.width, s.height), (320, 16384));
        assert!(s.maximized);
    }
}
