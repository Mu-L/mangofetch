# 🔌 Plugin Development SDK

MangoFetch supports dynamic extension via plugins. You can compile your own custom features and extractors into shared libraries (`.so`, `.dll`, or `.dylib`) and load them at runtime.

The plugin system relies on the **`mangofetch-plugin-sdk`** crate, which defines the standard interfaces and connection protocols.

## 🏗️ The `MangoFetchPlugin` Trait

Any plugin must implement the `MangoFetchPlugin` trait:

```rust
use std::sync::Arc;
use mangofetch_plugin_sdk::PluginHost;

pub trait MangoFetchPlugin: Send + Sync {
    /// Unique identifier for the plugin (e.g., "my-extractor")
    fn id(&self) -> &str;

    /// User-friendly name of the plugin
    fn name(&self) -> &str;

    /// SemVer version string
    fn version(&self) -> &str;

    /// Called by the host application during plugin startup
    fn initialize(&mut self, host: Arc<dyn PluginHost>) -> anyhow::Result<()>;

    /// Called by the host application during shutdown (cleanup)
    fn shutdown(&self) {}

    /// Executes custom commands sent from the host CLI or GUI
    fn handle_command(
        &self,
        command: String,
        args: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send + 'static>
    >;

    /// Lists the commands supported by this plugin
    fn commands(&self) -> Vec<String>;
}
```

To export the plugin so the core engine can discover it, use the `export_plugin!` macro:

```rust
use mangofetch_plugin_sdk::{MangoFetchPlugin, export_plugin};

struct MyPlugin;

impl MangoFetchPlugin for MyPlugin {
    // Implement trait methods...
}

export_plugin!(MyPlugin);
```

---

## 📡 The `PluginHost` Trait

When initialized, the host passes a thread-safe `PluginHost` reference to the plugin. This interface allows the plugin to interact with the host environment:

```rust
pub trait PluginHost: Send + Sync {
    /// Emit events back to the host UI (TUI or GUI)
    fn emit_event(&self, name: &str, payload: serde_json::Value) -> anyhow::Result<()>;

    /// Display a visual notification toast in the UI
    fn show_toast(&self, toast_type: &str, message: &str) -> anyhow::Result<()>;

    /// Get the persistent data storage directory allocated for this plugin
    fn plugin_data_dir(&self, plugin_id: &str) -> PathBuf;

    /// Get the frontend asset directory for custom UI routes
    fn plugin_frontend_dir(&self, plugin_id: &str) -> PathBuf;

    /// Retrieve the current JSON settings for this plugin
    fn get_settings(&self, plugin_id: &str) -> serde_json::Value;

    /// Save updated JSON settings for this plugin
    fn save_settings(&self, plugin_id: &str, settings: serde_json::Value) -> anyhow::Result<()>;

    /// Query proxy configuration configured in the host
    fn proxy_config(&self) -> Option<ProxyConfig>;

    /// Get path to checked helper binaries (e.g. "ffmpeg" or "yt-dlp")
    fn tool_path(&self, tool: &str) -> Option<PathBuf>;

    /// Query the user's default download destination
    fn default_output_dir(&self) -> PathBuf;
}
```

---

## 📄 The Plugin Manifest (`manifest.json`)

To load correctly, a plugin must be accompanied by a `manifest.json` file in its folder, mapping metadata and capabilities.

Example structure:
```json
{
  "id": "my-custom-plugin",
  "name": "My Custom Extractor",
  "version": "1.0.0",
  "description": "Extracts video media from proprietary educational APIs.",
  "author": "Developer Name",
  "min_mangofetch_version": "0.7.0",
  "license": "GPL-3.0-or-later",
  "capabilities": ["custom_extractors", "custom_ui"],
  "nav": [
    {
      "route": "/my-plugin-settings",
      "label": {
        "en": "Custom Extractor Settings",
        "es": "Configuración del Extractor"
      },
      "group": "secondary",
      "order": 10
    }
  ]
}
```

---

## 🛠️ Compilation and Deployment

1. Set your crate configuration type to `cdylib` in `Cargo.toml`:
   ```toml
   [lib]
   crate-type = ["cdylib"]
   ```
2. Build in release mode:
   ```bash
   cargo build --release
   ```
3. Copy the output binary (`.so` on Linux, `.dylib` on macOS, `.dll` on Windows) along with `manifest.json` to the host application plugins directory:
   - **Linux**: `~/.config/mangofetch/plugins/` (or your OS equivalent config folder).
