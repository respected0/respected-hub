//! Brain features the Hub shows (memory index, semantic search) through the product's own RPP; installing, settings and
//! removal belong to Brain's installer and commands.
use serde_json::Value;

pub mod memory_index;
pub mod memory_semantic;

/// One RPP call: (product, op, payload, operation id) -> response envelope.
pub type CallFn<'a> = &'a dyn Fn(&str, &str, Option<Value>, Option<String>) -> Value;

/// Whether the installed Brain offers `op`: its manifest lists the operations it serves.
pub fn offers(manifest: &Value, op: &str) -> bool {
    manifest["ok"] == true && manifest["result"]["capabilities"].as_array().map_or(false, |ops| ops.iter().any(|c| *c == op))
}
