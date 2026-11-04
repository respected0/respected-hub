use respected_hub_lib::integrations::catalog::{entries, is_verified_source_url, parse_catalog};
use serde_json::json;

fn record(id: &str, url: &str) -> serde_json::Value {
    json!({
        "id": id,
        "name": id,
        "type": "mcp",
        "description": "test",
        "category": "test",
        "source_url": url,
        "supported_clients": ["claude"]
    })
}

#[test]
fn empty_catalog_is_valid() {
    assert!(parse_catalog("[]").unwrap().is_empty());
}

#[test]
fn one_hundred_valid_records_parse() {
    let records: Vec<_> = (0..100).map(|i| record(&format!("entry-{i}"), &format!("https://github.com/owner/repo-{i}"))).collect();
    let parsed = parse_catalog(&serde_json::to_string(&records).unwrap()).unwrap();
    assert_eq!(parsed.len(), 100);
}

#[test]
fn duplicate_ids_are_rejected() {
    let records = [record("same", "https://github.com/owner/one"), record("same", "https://github.com/owner/two")];
    let error = parse_catalog(&serde_json::to_string(&records).unwrap()).unwrap_err();
    assert!(error.contains("duplicate catalog id"), "{error}");
}

#[test]
fn only_https_github_repository_urls_are_accepted() {
    for url in [
        "http://github.com/owner/repo",
        "https://gitlab.com/owner/repo",
        "https://github.com",
        "https://github.com/",
        "https://github.com/owner",
        "https://github.com//repo",
        "https://github.com.evil.example/owner/repo",
        "https://github.com/owner/..",
        "https://github.com/owner/repo with space",
        "https://github.com/@owner/repo",
    ] {
        assert!(!is_verified_source_url(url), "accepted {url}");
    }
    assert!(is_verified_source_url("https://github.com/owner/repo"));
    assert!(is_verified_source_url("https://github.com/owner/repo/tree/main/skill"));
    assert!(is_verified_source_url("https://github.com/owner/repo.git"));
}

#[test]
fn corrupt_json_is_rejected() {
    assert!(parse_catalog("{not json").is_err());
    let unknown = serde_json::to_string(&[record("x", "https://github.com/owner/repo")]).unwrap().replace("test", "test\", \"unexpected\": true");
    assert!(parse_catalog(&unknown).is_err());
}

#[test]
fn shipped_catalog_is_curated_and_needs_no_network() {
    let first = entries().unwrap();
    let second = entries().unwrap();
    assert!(first.len() >= 50, "only {} curated records", first.len());
    assert_eq!(first, second, "reading the compiled-in catalog is deterministic and offline");
    let mut ids = std::collections::HashSet::new();
    for entry in &first {
        assert!(ids.insert(entry.id.clone()), "duplicate id {}", entry.id);
        assert!(is_verified_source_url(&entry.source_url), "unverified URL for {}", entry.id);
    }
}