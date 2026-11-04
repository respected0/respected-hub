//! Per-client read-only adapters. Each adapter parses only the formats and locations
//! that the client officially documents; it never writes or executes anything.

pub mod claude;
