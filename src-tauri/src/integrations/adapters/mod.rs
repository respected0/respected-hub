//! Per-client read-only adapters. Each adapter parses only the formats and locations
//! that the client officially documents; it never writes or executes anything.

pub(crate) mod common;
pub mod cursor;
pub mod gemini;
pub mod opencode;
pub mod claude;
pub mod codex;
