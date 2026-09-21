// NOTE (DESKTOP-04): this crate compiles standalone with leaf deps (serde, serde_json,
// thiserror, async-trait). The full Tauri app (DESK-02) will restore
// `tauri_build::build()` here once the `tauri` crate is added to Cargo.toml.
//
// VS-16: `tauri` (v2) is now a dependency, so the restore happens here. Without
// `tauri_build::build()` Tauri never emits the `desktop`/`mobile` cfgs, and the
// single-instance/deep-link plugin registration (gated `#[cfg(desktop)]`) is
// silently compiled out — deep links would never be delivered on Windows.
fn main() {
    tauri_build::build()
}
