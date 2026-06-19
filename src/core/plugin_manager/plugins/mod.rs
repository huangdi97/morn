pub mod channel_bus_plugin;
pub use channel_bus_plugin::ChannelBusPlugin;

pub mod data_layer_plugin;
pub use data_layer_plugin::DataLayerPlugin;

pub mod engine_plugin;
pub use engine_plugin::EnginePlugin;

pub mod registry_plugin;
pub use registry_plugin::RegistryPlugin;

pub mod sandbox_plugin;
pub use sandbox_plugin::SandboxPlugin;

pub mod studio_plugin;
pub use studio_plugin::StudioPlugin;

pub mod supervisor_plugin;
pub use supervisor_plugin::SupervisorPlugin;

pub mod registry;
pub use registry::CorePluginRegistry;
