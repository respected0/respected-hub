//! Static, compiled-in directory of third-party integrations.
//!
//! Separate from `lifecycle::catalog` (product installers). This catalog has no network access and
//! never installs anything; it only describes verified GitHub sources for the Discover view.

use crate::integrations::model::ClientId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Curated records shipped inside the binary, so the list works with the network offline.
pub const CATALOG_JSON: &str = include_str!("../../assets/third-party-catalog.json");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CatalogKind {
    Mcp,
    Skill,
    Plugin,
    Rule,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CatalogEntry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub kind: CatalogKind,
    pub description: String,
    pub category: String,
    pub source_url: String,
    pub supported_clients: Vec<ClientId>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub provenance: Option<String>,
}

/// Only a canonical GitHub repository link is accepted in MVP (no other host, no plain HTTP).
pub fn is_verified_source_url(url: &str) -> bool {
    let Some(path) = url.strip_prefix("https://github.com/") else { return false };
    if url.chars().any(|c| c.is_whitespace() || c.is_control()) || path.contains('@') || path.contains('\\') {
        return false;
    }
    let mut parts = path.split('/');
    let (Some(owner), Some(repo)) = (parts.next(), parts.next()) else { return false };
    let repo = repo.split(['?', '#']).next().unwrap_or(repo);
    !owner.is_empty()
        && !repo.is_empty()
        && owner != "."
        && owner != ".."
        && repo != "."
        && repo != ".."
        && owner.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
        && repo.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Parse and validate a catalog document. Duplicate IDs and untrusted links make the whole document invalid.
pub fn parse_catalog(json: &str) -> Result<Vec<CatalogEntry>, String> {
    let entries: Vec<CatalogEntry> = serde_json::from_str(json).map_err(|e| format!("invalid catalog JSON: {e}"))?;
    let mut ids = HashSet::new();
    for entry in &entries {
        if entry.id.trim().is_empty() || entry.name.trim().is_empty() || entry.category.trim().is_empty() {
            return Err("catalog entry has an empty id, name or category".into());
        }
        if !ids.insert(entry.id.as_str()) {
            return Err(format!("duplicate catalog id: {}", entry.id));
        }
        if !is_verified_source_url(&entry.source_url) {
            return Err(format!("catalog entry {} has an unverified source URL", entry.id));
        }
        if entry.supported_clients.is_empty() || entry.supported_clients.contains(&ClientId::Unknown) {
            return Err(format!("catalog entry {} has no valid supported client", entry.id));
        }
    }
    Ok(entries)
}

/// The shipped catalog. Reading it performs no filesystem or network I/O.
pub fn entries() -> Result<Vec<CatalogEntry>, String> {
    parse_catalog(CATALOG_JSON)
}