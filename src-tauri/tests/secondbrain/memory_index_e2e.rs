//! Real end-to-end proof of reading and refreshing SecondBrain's memory search index through the real engine.
//! Temporary data roots and a fake home only; the vault notes must stay byte-identical.
use respected_hub_lib::secondbrain::memory_index as secondbrain_memory_index;
use respected_hub_lib::rpp::{rpp_call, rpp_call_trusted_timeout};
use respected_hub_lib::Settings;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::time::Duration;
#[path = "../common/mod.rs"]
mod common;
use common::{ecosystem, product_folders, Temp};

fn read_notes(paths: &[PathBuf]) -> Vec<Vec<u8>> { paths.iter().map(|p| std::fs::read(p).unwrap()).collect() }

fn count_markdown(dir: &Path) -> usize {
    std::fs::read_dir(dir).map(|entries| entries.flatten().map(|e| {
        let path = e.path();
        if path.is_dir() { count_markdown(&path) } else if path.extension().map_or(false, |x| x == "md") { 1 } else { 0 }
    }).sum()).unwrap_or(0)
}

#[test]
fn the_search_index_is_read_and_refreshed_through_the_real_engine() {
    let eco = ecosystem();
    let (brain_data, home, vault) = (Temp::new("brain"), Temp::new("home"), Temp::new("vault"));
    let _folders = product_folders(&brain_data.0);
    std::env::set_var("RESPECTED_HUB_TEST_HOME", &home.0);
    let settings = Settings { python: "python".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: eco.join("respected-secondbrain"),
        brain_exe: None, orchestrator_exe: None };
    let call = |p: &str, o: &str, pl: Option<Value>, id: Option<String>| {
        let timeout = if o == "memory.index.refresh" { Duration::from_secs(300) } else { Duration::from_secs(30) };
        rpp_call_trusted_timeout(&settings, p, o, pl, id, timeout)
    };

    // the notes folder is registered with SecondBrain's own command, as a person would without the Hub
    let registered = std::process::Command::new("python").args(["-m", "respectedbrain", "vault", "register"]).arg(&vault.0)
        .env("PYTHONPATH", eco.join("respected-secondbrain").join("src")).env("USERPROFILE", &home.0).env("HOME", &home.0).output().unwrap();
    assert!(registered.status.success(), "{}", String::from_utf8_lossy(&registered.stderr));

    let notes: Vec<PathBuf> = (1..=3).map(|n| vault.0.join(format!("note-{n}.md"))).collect();
    for (n, path) in notes.iter().enumerate() {
        std::fs::write(path, format!("# Not {}\n\nIndeks sınaması için içerik {}.\n", n + 1, n + 1)).unwrap();
    }
    let mut before = read_notes(&notes);
    // count what is there instead of assuming only our notes
    let baseline = count_markdown(&vault.0);

    let status = secondbrain_memory_index::status(&call);
    assert_eq!(status["supported"], true, "{status}");
    assert_eq!(status["state"], "missing", "{status}");

    let first = secondbrain_memory_index::refresh(&call, false).unwrap_or_else(|e| panic!("refresh: {e}"));
    assert_eq!(first["indexed"], json!(baseline), "{first}");
    let status = secondbrain_memory_index::status(&call);
    assert_eq!(status["state"], "fresh", "{status}");
    assert!(status["last_refresh"].is_object(), "{status}");

    // One edited and one new note make the index stale; only those two are indexed again.
    std::fs::write(&notes[0], "# Not 1\n\nDeğişti.\n").unwrap();
    let extra = vault.0.join("note-4.md");
    std::fs::write(&extra, "# Not 4\n\nYeni not.\n").unwrap();
    let stale = secondbrain_memory_index::status(&call);
    assert_eq!(stale["state"], "stale", "{stale}");
    let second = secondbrain_memory_index::refresh(&call, false).unwrap_or_else(|e| panic!("refresh: {e}"));
    assert_eq!((second["indexed"].clone(), second["skipped"].clone()), (json!(2), json!(baseline - 1)), "{second}");

    // Refreshing only writes SecondBrain's own index: the vault notes are untouched.
    let after = read_notes(&notes);
    before[0] = b"# Not 1\n\nDe\xc4\x9fi\xc5\x9fti.\n".to_vec();
    assert_eq!(before, after, "vault notes changed");
    assert_eq!(std::fs::read(&extra).unwrap(), "# Not 4\n\nYeni not.\n".as_bytes());

    // The webview bridge (untrusted) must not reach the index ops.
    for (op, payload) in [("memory.index.refresh", json!({ "confirm": true })), ("memory.index.status", json!({}))] {
        let denied = rpp_call(&settings, "secondbrain", op, Some(payload), Some("idx-e2e-denied-1".into()));
        assert_eq!(denied["ok"], false, "{denied}");
        assert_eq!(denied["error"]["code"], "PERMISSION_DENIED", "{denied}");
    }
}
