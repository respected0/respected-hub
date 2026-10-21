//! Hub-managed Orchestrator MCP registration on a fake HOME and temporary roots: plan -> approval-bound apply ->
//! the registered command works as a client would start it (no Hub running) -> disable/rollback -> foreign entries intact.
use respected_hub_lib::lifecycle::{self, Roots};
use respected_hub_lib::{mcp_access, Settings};
use serde_json::{json, Value};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;
mod common;
use common::Temp;

// Both tests repoint process-wide HOME variables.
static ENV: Mutex<()> = Mutex::new(());

fn read_json(path: &Path) -> Value { serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap() }

/// Start the registered command exactly as a client does and call a tool over stdio.
fn client_call(entry: &Value) -> Value {
    let mut command = Command::new(entry["command"].as_str().unwrap());
    command.args(entry["args"].as_array().unwrap().iter().map(|a| a.as_str().unwrap())).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().unwrap();
    let lines = [json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}),
        json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"orchestrator_status","arguments":{}}})];
    child.stdin.take().unwrap().write_all((lines[0].to_string() + "\n" + &lines[1].to_string() + "\n").as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    text.lines().filter_map(|l| serde_json::from_str::<Value>(l).ok()).find(|v| v["id"] == 2).unwrap_or_else(|| panic!("no tool reply: {text} / {}", String::from_utf8_lossy(&out.stderr)))
}

#[test]
fn orchestrator_mcp_registration_is_planned_approved_working_and_reversible() {
    let _env = ENV.lock().unwrap_or_else(|e| e.into_inner());
    let eco = std::env::var_os("RESPECTED_ECOSYSTEM_ROOT").map(std::path::PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".."));
    let (managed, work) = (Temp::new("managed"), Temp::new("work"));
    let home = work.0.join("home");
    std::fs::create_dir_all(home.join(".codex")).unwrap();
    let foreign_json = json!({ "numStartups": 3, "mcpServers": { "someone-elses": { "command": "x", "args": [] } } });
    std::fs::write(home.join(".claude.json"), serde_json::to_vec_pretty(&foreign_json).unwrap()).unwrap();
    let foreign_toml = "# mine\nmodel = \"m\"\n\n[mcp_servers.other]\ncommand = \"y\"\n";
    std::fs::write(home.join(".codex/config.toml"), foreign_toml).unwrap();
    // Only this test touches these variables; they confine the product to the fake HOME.
    for name in ["USERPROFILE", "HOME"] { std::env::set_var(name, &home); }
    std::env::set_var("APPDATA", home.join("AppData"));
    let roots = Roots { managed: managed.0.clone(), catalog: work.0.join("catalog") };
    let settings = Settings { python: "python".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: work.0.join("none"),
        data_root: work.0.join("orch"), brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None };

    let status = mcp_access::plan(&roots, &settings, &json!({})).unwrap();
    let state = |plan: &Value, client: &str| plan["clients"].as_array().unwrap().iter().find(|c| c["client"] == client).unwrap().clone();
    assert_eq!(state(&status, "claude-code")["state"], "absent");
    assert_eq!(state(&status, "cursor")["detected"], false);

    let desired = json!({ "claude-code": true, "codex": true });
    let planned = mcp_access::plan(&roots, &settings, &desired).unwrap();
    assert_eq!(planned["writes_nothing"], true);
    assert_eq!(state(&planned, "codex")["action"], "create");
    assert_eq!(read_json(&home.join(".claude.json")), foreign_json, "plan wrote nothing");

    let stale = mcp_access::apply(&roots, &settings, &desired, &"0".repeat(64), "e2e-mcp-stale-1").unwrap_err();
    assert_eq!(stale.code, "PLAN_CHANGED");
    assert_eq!(read_json(&home.join(".claude.json")), foreign_json);

    let digest = planned["plan_digest"].as_str().unwrap();
    let applied = mcp_access::apply(&roots, &settings, &desired, digest, "e2e-mcp-on-0001").unwrap();
    assert_eq!(applied["status"], "applied");
    let after = read_json(&home.join(".claude.json"));
    assert_eq!(after["numStartups"], 3);
    assert_eq!(after["mcpServers"]["someone-elses"], foreign_json["mcpServers"]["someone-elses"]);
    let toml = std::fs::read_to_string(home.join(".codex/config.toml")).unwrap();
    assert!(toml.contains("# mine") && toml.contains("[mcp_servers.other]") && toml.contains("[mcp_servers.respected-orchestrator]"));

    // The Hub is not involved from here: the client starts the registered command by itself.
    let reply = client_call(&after["mcpServers"]["respected-orchestrator"]);
    assert_eq!(reply["result"]["isError"], false, "{reply}");
    assert!(reply["result"]["content"][0]["text"].as_str().unwrap().contains("respected-orchestrator"));

    // Disconnect one client; the other and all foreign settings stay.
    let off = json!({ "claude-code": false });
    let off_plan = mcp_access::plan(&roots, &settings, &off).unwrap();
    mcp_access::apply(&roots, &settings, &off, off_plan["plan_digest"].as_str().unwrap(), "e2e-mcp-off-0001").unwrap();
    assert_eq!(read_json(&home.join(".claude.json")), foreign_json);
    assert!(std::fs::read_to_string(home.join(".codex/config.toml")).unwrap().contains("respected-orchestrator"));

    // Roll the first operation back: codex returns to its original content (claude-code was already off, so it is refused as changed).
    assert_eq!(mcp_access::rollback(&roots, &settings, "e2e-mcp-on-0001").unwrap_err().code, "ROLLBACK_UNAVAILABLE");
    let off_codex = json!({ "codex": false });
    let p = mcp_access::plan(&roots, &settings, &off_codex).unwrap();
    mcp_access::apply(&roots, &settings, &off_codex, p["plan_digest"].as_str().unwrap(), "e2e-mcp-off-0002").unwrap();
    assert_eq!(std::fs::read_to_string(home.join(".codex/config.toml")).unwrap().trim(), foreign_toml.trim());
    assert_eq!(lifecycle::read_journal(&roots, "e2e-mcp-on-0001").unwrap()["status"], "applied");

    // A pre-existing foreign entry with our name is never overwritten.
    let mut taken = foreign_json.clone();
    taken["mcpServers"]["respected-orchestrator"] = json!({ "command": "mine" });
    std::fs::write(home.join(".claude.json"), serde_json::to_vec(&taken).unwrap()).unwrap();
    let conflict = mcp_access::plan(&roots, &settings, &json!({ "claude-code": true })).unwrap();
    assert_eq!(state(&conflict, "claude-code")["action"], "refuse");
    let refused = mcp_access::apply(&roots, &settings, &json!({ "claude-code": true }), conflict["plan_digest"].as_str().unwrap(), "e2e-mcp-on-0002").unwrap_err();
    assert_eq!(refused.code, "LIFECYCLE_CONFLICT");
    assert_eq!(read_json(&home.join(".claude.json")), taken);
}
