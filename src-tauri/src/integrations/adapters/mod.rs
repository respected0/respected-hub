//! Per-client read-only adapters. Each adapter parses only the formats and locations
//! that the client officially documents; it never writes or executes anything.

mod common;
pub mod claude;
pub mod codex;
