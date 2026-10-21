//! Product-specific parts of the Hub. SecondBrain: lifecycle through its own engine (`secondbrain`) and the memory features the
//! Hub offers on top of it (`secondbrain_memory_access`, `secondbrain_memory_index`, `secondbrain_memory_semantic`). Orchestrator: its engine and lifecycle (`orchestrator`) and MCP access for
//! AI clients (`orchestrator_mcp`). The Hub core (lifecycle, catalog, registration, trust, RPP client) stays at the crate root.
pub mod secondbrain;
pub mod orchestrator_mcp;
pub mod orchestrator;
pub mod secondbrain_memory_access;
pub mod secondbrain_memory_index;
pub mod secondbrain_memory_semantic;
