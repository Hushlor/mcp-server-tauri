// The published crate keeps its Hushlor package name, but Tauri derives ACL
// identifiers from CARGO_PKG_NAME. Override that build-time input with the
// upstream package name; tauri-plugin's Builder removes the `tauri-plugin-`
// prefix when it generates the official `mcp-bridge` namespace. `links`
// remains the same official name for consuming apps.
const RUNTIME_PLUGIN_PACKAGE_NAME: &str = "tauri-plugin-mcp-bridge";

fn main() {
    std::env::set_var("CARGO_PKG_NAME", RUNTIME_PLUGIN_PACKAGE_NAME);

    tauri_plugin::Builder::new(&[
        "capture_native_screenshot",
        "emit_event",
        "execute_command",
        "execute_js",
        "get_backend_state",
        "get_ipc_events",
        "get_window_info",
        "list_windows",
        "report_ipc_event",
        "request_script_injection",
        "script_result",
        "start_ipc_monitor",
        "stop_ipc_monitor",
    ])
    .build();
}
