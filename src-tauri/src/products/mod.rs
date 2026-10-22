//! Product-specific parts of the Hub, one folder per product and never importing each other. Brain: lifecycle through its
//! own engine (`brain`) and the memory features the Hub offers on top of it (`brain::memory_*`). Orchestrator: its
//! engine and lifecycle (`orchestrator`) and MCP access for AI clients (`orchestrator::mcp`). The Hub core (`hub`, `lifecycle`,
//! `trust`, `rpp`) sits beside this folder.
pub mod orchestrator;
pub mod brain;
