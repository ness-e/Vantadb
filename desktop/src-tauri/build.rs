// NOTE (DESKTOP-04): this crate compiles standalone with leaf deps (serde, serde_json,
// thiserror, async-trait). The full Tauri app (DESK-02) will restore
// `tauri_build::build()` here once the `tauri` crate is added to Cargo.toml.
fn main() {}
