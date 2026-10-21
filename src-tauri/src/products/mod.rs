//! Product-specific parts of the Hub. SecondBrain: lifecycle through its own engine (`brain`) and the memory features the
//! Hub offers on top of it (`memory_access`, `memory_index`, `memory_semantic`). Orchestrator: MCP access for AI clients
//! (`mcp_access`). The Hub core (lifecycle, catalog, registration, trust, RPP client) stays at the crate root.
pub mod brain;
pub mod mcp_access;
pub mod memory_access;
pub mod memory_index;
pub mod memory_semantic;
