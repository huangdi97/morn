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

pub mod mcp_plugin;
pub use mcp_plugin::McpPlugin;

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

pub mod model_router_plugin;
pub use model_router_plugin::ModelRouterPlugin;

pub mod memory_plugin;
pub use memory_plugin::MemoryPlugin;

pub mod security_plugin;
pub use security_plugin::SecurityPlugin;

pub mod intent_parser_plugin;
pub use intent_parser_plugin::IntentParserPlugin;

pub mod task_engine_plugin;
pub use task_engine_plugin::TaskEnginePlugin;

pub mod approval_plugin;
pub use approval_plugin::ApprovalPlugin;

pub mod workflow_plugin;
pub use workflow_plugin::WorkflowPlugin;

pub mod orchestrator_plugin;
pub use orchestrator_plugin::OrchestratorPlugin;

pub mod data_flow_plugin;
pub use data_flow_plugin::DataFlowPlugin;

pub mod computer_use_plugin;
pub use computer_use_plugin::ComputerUsePlugin;

pub mod scheduler_plugin;
pub use scheduler_plugin::SchedulerPlugin;

pub mod proactive_plugin;
pub use proactive_plugin::ProactivePlugin;

pub mod delegation_plugin;
pub use delegation_plugin::DelegationPlugin;

pub mod consensus_plugin;
pub use consensus_plugin::ConsensusPlugin;

pub mod agent_pool_plugin;
pub use agent_pool_plugin::AgentPoolPlugin;

pub mod code_tool_plugin;
pub use code_tool_plugin::CodeToolPlugin;

pub mod type_registry_plugin;
pub use type_registry_plugin::TypeRegistryPlugin;

pub mod assembly_plugin;
pub use assembly_plugin::AssemblyPlugin;

pub mod registry;
pub use registry::CorePluginRegistry;
