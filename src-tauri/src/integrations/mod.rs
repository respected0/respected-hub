//! Read-only inventory of third-party AI client integrations.
//!
//! M1/M2 contract: scanning never writes to a client configuration and never executes third-party code.

pub mod model;
pub mod api;
pub mod security;
pub mod discovery;
pub mod adapters;
pub mod skills;
pub mod plugins;
pub mod resolver;
pub mod catalog;
pub mod inventory;
pub mod planner;
pub mod apply;
pub mod projects;
