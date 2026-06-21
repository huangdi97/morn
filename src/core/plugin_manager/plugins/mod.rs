pub mod backup_plugin;
pub use backup_plugin::BackupPlugin;

pub mod channel_bus_plugin;
pub use channel_bus_plugin::ChannelBusPlugin;

pub mod channel_plugins;
pub use channel_plugins::*;

pub mod data_layer_plugin;
pub use data_layer_plugin::DataLayerPlugin;

pub mod engine_plugin;
pub use engine_plugin::EnginePlugin;

pub mod hub_plugin;
pub use hub_plugin::HubPlugin;

pub mod observability_plugin;
pub use observability_plugin::ObservabilityPlugin;

pub mod registry_plugin;
pub use registry_plugin::RegistryPlugin;

pub mod sandbox_plugin;
pub use sandbox_plugin::SandboxPlugin;

pub mod studio_plugin;
pub use studio_plugin::StudioPlugin;

pub mod supervisor_plugin;
pub use supervisor_plugin::SupervisorPlugin;

pub mod sync_plugin;
pub use sync_plugin::SyncPlugin;

pub mod voice_plugin;
pub use voice_plugin::VoicePlugin;

pub mod registry;
pub use registry::CorePluginRegistry;
