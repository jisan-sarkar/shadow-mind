fn main() {
    let commands = &["snapshot", "update_preferences", "session_action"];
    let attributes = tauri_build::Attributes::new()
        .app_manifest(tauri_build::AppManifest::new().commands(commands));
    tauri_build::try_build(attributes).expect("configure ShadowMind application permissions");
}
