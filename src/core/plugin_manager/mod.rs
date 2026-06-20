//! plugin_manager — Scan, load, activate, and deactivate plugins.

pub mod error;
pub use error::PluginError;
pub use error::PluginOrderError;

pub mod adapter;
pub mod bridge_plugin;
pub mod plugins;
#[cfg(feature = "sandbox")]
pub mod wasm_plugin;
pub use plugins::CorePluginRegistry;
pub mod config;
pub use config::PluginConfig;

pub mod types;
pub use types::{
    list_morn_plugin_metas, register_morn_plugin, toggle_morn_plugin_enabled, MornPluginMeta,
    Plugin, PluginManifest, PluginStatus, PluginType,
};

pub mod context;
pub use context::PluginContext;

pub mod traits;
pub use traits::{MornPlugin, TypedPlugin};

pub mod lifecycle;
pub use lifecycle::{
    disable_plugin, enable_plugin, load_plugins, safe_activate, safe_deactivate, safe_init,
    topological_sort,
};

pub mod manager;
pub use manager::PluginManager;

#[cfg(test)]
mod tests;