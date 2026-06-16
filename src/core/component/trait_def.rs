//! trait_def — Standard Component Interface for all atomic components (§2.2 of DESIGN.md)
//!
//! All components in the Morn system implement these traits, ensuring
//! any component can be freely connected — like a fifth-wheel coupling on a truck.
use crate::core::error::MornError;
pub use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};
