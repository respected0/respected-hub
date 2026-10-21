//! The Hub polls SecondBrain over RPP every few seconds while a lifecycle operation runs. An install must still
//! finish (no deadlock between the polls and the installer) and the polls must never run from the program folder.
use respected_hub_lib::brain::{self, BrainDirs};
use respected_hub_lib::lifecycle::Roots;
use respected_hub_lib::rpp::rpp_call;
use respected_hub_lib::{hub_apply, hub_plan, Settings};
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct Temp(PathBuf);
impl Temp {
    fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let path = std::env::temp_dir().join(format!("hub-conc-{label}-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&path).unwrap();
        Temp(path)
    }
}
impl Drop for Temp { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }

fn ecosystem() -> PathBuf {
    std::env::var_os("RESPECTED_ECOSYSTEM_ROOT").map(PathBuf::from).unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join(".."))
}

#[test]
fn an_install_completes_while_status_polls_and_wizard_plans_keep_running() {
    let eco = ecosystem();
    let (managed, catalog, work) = (Temp::new("managed"), Temp::new("catalog"), Temp::new("work"));
    let home = work.0.join("home");
    std::fs::create_dir_all(&home).unwrap();
    for name in ["USERPROFILE", "HOME"] { std::env::set_var(name, &home); }
    let dirs = BrainDirs { app: work.0.join("app"), data: work.0.join("data") };
    std::env::set_var("RESPECTED_HUB_BRAIN_APP_DIR", &dirs.app);
    std::env::set_var("RESPECTED_HUB_BRAIN_DATA_DIR", &dirs.data);
    assert!(Command::new("python").arg("-B").arg(eco.join("respected-hub").join("release").join("tools").join("build_bundle.py")).args(["secondbrain", "--out"]).arg(&catalog.0).status().unwrap().success());
    let roots = Roots { managed: managed.0.clone(), catalog: catalog.0.clone() };
    let settings = Settings { python: "none".into(), orchestrator_root: eco.join("respected-orchestrator"), brain_root: work.0.join("none"),
        data_root: work.0.join("orch"), brain_data_root: None, brain_exe: None, brain_app_dir: None, orchestrator_exe: None };
    let vault = work.0.join("Vault");
    let params = json!({ "vault_path": vault, "desired": { "global": true, "mcp": true } });

    // Stage once so the polls use the staged launcher, exactly like the app after the wizard's first plan.
    hub_plan(&roots, &dirs, &settings, "secondbrain", "install", &params).unwrap_or_else(|e| panic!("plan: {}", e.message));

    let stop = Arc::new(AtomicBool::new(false));
    let polls = Arc::new(AtomicUsize::new(0));
    let mut threads = Vec::new();
    for _ in 0..3 {
        let (stop, polls, roots, dirs, settings) = (stop.clone(), polls.clone(), roots.clone(), dirs.clone(), settings.clone());
        threads.push(std::thread::spawn(move || {
            while !stop.load(Ordering::SeqCst) {
                let mut live = settings.clone();
                brain::overlay(&roots, &dirs, &mut live);
                for op in ["manifest", "health", "settings.readback"] { let _ = rpp_call(&live, "secondbrain", op, None, None); }
                polls.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(500));
            }
        }));
    }
    let planner = { let (stop, roots, dirs, settings, params) = (stop.clone(), roots.clone(), dirs.clone(), settings.clone(), params.clone());
        std::thread::spawn(move || while !stop.load(Ordering::SeqCst) { let _ = hub_plan(&roots, &dirs, &settings, "secondbrain", "install", &params); std::thread::sleep(Duration::from_millis(400)); }) };

    std::thread::sleep(Duration::from_secs(3));
    let (tx, rx) = std::sync::mpsc::channel();
    { let (roots, dirs, settings, params) = (roots.clone(), dirs.clone(), settings.clone(), params.clone());
      std::thread::spawn(move || { let _ = tx.send(hub_apply(&roots, &dirs, settings, "secondbrain", "install", &params, "e2e-conc-install-1")); }); }
    let started = Instant::now();
    let outcome = rx.recv_timeout(Duration::from_secs(400));
    stop.store(true, Ordering::SeqCst);
    for t in threads { let _ = t.join(); }
    let _ = planner.join();
    if outcome.is_err() { eprintln!("TIMEOUT: install did not finish; exiting without joining the stuck threads"); std::process::exit(7); }
    let result = outcome.unwrap_or_else(|_| panic!("install did not finish within 400 s while polls ran ({} polls done)", polls.load(Ordering::SeqCst)));
    assert_eq!(result.unwrap_or_else(|e| panic!("install failed: {} {}", e.message, e.hint))["status"], "applied");
    assert!(polls.load(Ordering::SeqCst) > 0);
    eprintln!("install finished in {:?} with {} polls", started.elapsed(), polls.load(Ordering::SeqCst));
}
