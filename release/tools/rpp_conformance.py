"""Cross-product RPP check: drives both real servers on temp data roots and feeds every
response to the Hub's own parser.

v1.0 is asserted exactly as before. A v1.1 section additionally checks negotiation, the
richer manifest, lifecycle.plan/apply/status/rollback, operation.receipt and the new
typed errors; it is skipped with an explicit message for a server that only advertises
protocol max 1.0.

Acceptance modes: --require-v11 fails if any server skips v1.1; --expect-v10-only
fails unless every server skips v1.1. RESPECTED_ECOSYSTEM_ROOT can select the
ecosystem checkout under test.
"""
import json, os, subprocess, sys, tempfile
from pathlib import Path

from ecosystem_root import ECO as ROOT
PY = sys.executable
tmp = Path(tempfile.mkdtemp(prefix="rpp-conf-"))
SERVERS = {
    "orchestrator": ([PY, "-m", "respected_orchestrator", "--data-root", str(tmp / "orch"), "rpp"],
                     ROOT / "respected-orchestrator" / "src", {}),
    "secondbrain": ([PY, "-m", "respectedbrain", "rpp", "--data-root", str(tmp / "brain")],
                    ROOT / "respected-secondbrain" / "src", {}),
}


def call(name, op, payload=None, operation_id=None, rpp="1.0"):
    cmd, src, _ = SERVERS[name]
    req = {"rpp": rpp, "op": op, "operation_id": operation_id, "payload": payload or {}}
    env = dict(os.environ, PYTHONPATH=str(src), PYTHONDONTWRITEBYTECODE="1")
    out = subprocess.run(cmd, input=json.dumps(req), capture_output=True, text=True, env=env, timeout=60)
    lines = [l for l in out.stdout.splitlines() if l.strip().startswith("{")]
    return lines[-1] if lines else f"<no json; rc={out.returncode}; err={out.stderr[-200:]}>"


def jget(text):
    try:
        return json.loads(text)
    except (ValueError, TypeError):
        return {}


def ver(v):
    try:
        return tuple(int(p) for p in str(v).split("."))
    except ValueError:
        return ()


fails, raw, skipped = [], [], []
require_v11 = "--require-v11" in sys.argv
expect_v10_only = "--expect-v10-only" in sys.argv
def check(cond, msg):
    print(("PASS " if cond else "FAIL ") + msg)
    if not cond:
        fails.append(msg)


def err_ok(j, code, msg):
    e = j.get("error")
    check(j.get("ok") is False and isinstance(e, dict) and {"code", "message", "hint"} <= set(e)
          and e.get("code") == code, msg)


manifests, proto_max = {}, {}
for name in SERVERS:
    m = json.loads(call(name, "manifest"))
    manifests[name] = m
    raw.append(json.dumps(m))
    check(m.get("ok") and set(m) >= {"rpp", "ok", "operation_id", "result", "error"}, f"{name}: manifest envelope")
    r = m.get("result") or {}
    protocol = r.get("protocol") or {}
    check(protocol.get("min") == "1.0" and protocol.get("max") in ("1.0", "1.1") and r.get("product_id") == f"respected-{name}",
          f"{name}: manifest fields")
    proto_max[name] = (r.get("protocol") or {}).get("max")
    h = json.loads(call(name, "health")); raw.append(json.dumps(h))
    check(h.get("ok") and h["result"]["status"] in ("healthy", "degraded", "unhealthy") and h["result"]["checks"], f"{name}: health")
    s = json.loads(call(name, "settings.schema")); raw.append(json.dumps(s))
    fields = s["result"]["fields"]; check(fields, f"{name}: schema empty")
    f0 = next(f for f in fields if not f.get("secret"))
    v = json.loads(call(name, "settings.validate", {"settings": {"__nope__": 1}})); raw.append(json.dumps(v))
    check(v["ok"] and v["result"]["valid"] is False, f"{name}: unknown key not rejected")
    bad = json.loads(call(name, "settings.apply", {"settings": {"__nope__": 1}}, "conf-bad-0001")); raw.append(json.dumps(bad))
    check(bad["ok"] is False and bad["error"]["code"] == "INVALID_SETTINGS", f"{name}: invalid apply code {bad.get('error')}")
    rb0 = json.loads(call(name, "settings.readback")); raw.append(json.dumps(rb0))
    value = f0.get("default")
    a = json.loads(call(name, "settings.apply", {"settings": {f0["key"]: value}}, "conf-ok-00001")); raw.append(json.dumps(a))
    check(a["ok"] and a["operation_id"] == "conf-ok-00001", f"{name}: apply ok {a.get('error')}")
    a2 = call(name, "settings.apply", {"settings": {f0["key"]: value}}, "conf-ok-00001"); raw.append(a2)
    check(json.loads(a2)["result"] == a["result"], f"{name}: replay not idempotent")
    other = {"orchestrator": {"default_timeout_seconds": 777}}.get(name)
    if other:
        c = json.loads(call(name, "settings.apply", {"settings": other}, "conf-ok-00001")); raw.append(json.dumps(c))
        check(c["ok"] is False and c["error"]["code"] == "OPERATION_ID_CONFLICT", f"{name}: conflict code {c.get('error')}")
    rb = json.loads(call(name, "settings.readback")); raw.append(json.dumps(rb))
    check(rb["ok"] and f0["key"] in rb["result"]["settings"], f"{name}: readback missing key")
    check((tmp / ("orch" if name == "orchestrator" else "brain") / "receipts" / "conf-ok-00001.json").is_file(), f"{name}: receipt file missing")
    u = json.loads(call(name, "nope.op")); raw.append(json.dumps(u))
    check(u["ok"] is False and u["error"]["code"] == "UNKNOWN_OP", f"{name}: unknown op")
    raw.append(call(name, "manifest").replace('"1.0"', '"1.0"'))
    pv = subprocess.run(SERVERS[name][0], input=json.dumps({"rpp": "9.0", "op": "manifest", "payload": {}}), capture_output=True, text=True,
                        env=dict(os.environ, PYTHONPATH=str(SERVERS[name][1]), PYTHONDONTWRITEBYTECODE="1"))
    pj = json.loads([l for l in pv.stdout.splitlines() if l.startswith("{")][-1]); raw.append(json.dumps(pj))
    check(pj["ok"] is False and pj["error"]["code"] == "PROTOCOL_UNSUPPORTED", f"{name}: protocol version {pj.get('error')}")
    bj = subprocess.run(SERVERS[name][0], input="not json", capture_output=True, text=True,
                        env=dict(os.environ, PYTHONPATH=str(SERVERS[name][1]), PYTHONDONTWRITEBYTECODE="1"))
    bl = [l for l in bj.stdout.splitlines() if l.startswith("{")]
    check(bl and json.loads(bl[-1])["error"]["code"] == "BAD_REQUEST", f"{name}: bad json -> BAD_REQUEST")
    raw.extend(bl[-1:])


# RPP v1.1 (additive; see docs/decisions/RPP_V1_1.md). Runs only when a server advertises
# protocol max >= 1.1; otherwise it is skipped with an explicit message so a v1.0-only
# server still passes.
LIFECYCLE_ACTIONS = ("install", "update", "repair", "uninstall")
INSTALL_STATES = ("absent", "installed", "update-available", "repair-needed")
RECEIPT_STATES = ("planned", "in_progress", "applied", "failed", "rolled_back")
USAGE_KEYS = ("raw_input_tokens", "cached_input_tokens", "output_tokens", "reasoning_tokens", "estimated_weighted_tokens")

for name in SERVERS:
    if ver(proto_max.get(name)) < (1, 1):
        skipped.append(name)
        print(f"SKIP {name}: v1.1 checks skipped - server advertises protocol max {proto_max.get(name)!r} (needs >= 1.1)")
        continue
    droot = tmp / ("orch" if name == "orchestrator" else "brain")
    recdir, settings_file = droot / "receipts", droot / "settings.json"

    # negotiation: a 1.1 request is accepted and echoed; the same server still answers 1.0.
    m11 = jget(call(name, "manifest", rpp="1.1")); raw.append(json.dumps(m11))
    check(m11.get("ok") and m11.get("rpp") == "1.1", f"{name}: v1.1 negotiate echoes rpp 1.1")
    r11 = m11.get("result") or {}
    check(r11.get("protocol") == {"min": "1.0", "max": "1.1"}, f"{name}: v1.1 protocol min/max")
    check(manifests[name].get("ok") and manifests[name].get("rpp") == "1.0", f"{name}: v1.1 server still answers rpp 1.0")

    # richer manifest (v1.0 keys retained).
    check(isinstance(r11.get("installed"), bool), f"{name}: v1.1 manifest.installed bool")
    check(r11.get("install_state") in INSTALL_STATES, f"{name}: v1.1 manifest.install_state")
    check("product_version" in r11, f"{name}: v1.1 manifest.product_version")
    roots = r11.get("roots")
    check(isinstance(roots, dict) and set(roots) == {"data", "app", "vault"} and all(x is None or isinstance(x, str) for x in roots.values()),
          f"{name}: v1.1 manifest.roots paths-only")
    caps, cd = r11.get("capabilities"), r11.get("capability_details")
    check(isinstance(caps, list) and isinstance(cd, list) and cd, f"{name}: v1.1 manifest.capability_details present")
    check(all(isinstance(c, dict) and {"id", "since", "ops"} <= set(c) and c.get("since") in ("1.0", "1.1") and isinstance(c.get("ops"), list)
              for c in cd or []), f"{name}: v1.1 capability_details shape")
    check(set(caps or []) <= {c.get("id") for c in cd or []}, f"{name}: v1.1 capabilities retained in capability_details")

    # lifecycle.plan is a read-only dry run for every action and writes no state.
    # repair/uninstall plans need an install record (Orchestrator enforces it); seed one first.
    call(name, "lifecycle.apply", {"action": "install", "target": {"app_root": str(tmp / "app")}}, "conf-seed-install-" + name[:4], rpp="1.1")
    rec_before = sorted(p.name for p in recdir.glob("*.json")) if recdir.is_dir() else []
    set_before = settings_file.read_bytes() if settings_file.is_file() else None
    plans = {}
    for action in LIFECYCLE_ACTIONS:
        pl = jget(call(name, "lifecycle.plan", {"action": action, "target": {"app_root": str(tmp / "app")}}, rpp="1.1")); raw.append(json.dumps(pl))
        res = pl.get("result") or {}
        steps = (res.get("plan") or {}).get("steps")
        check(pl.get("ok") and res.get("action") == action and res.get("dry_run") is True and isinstance(steps, list) and steps
              and all({"id", "action", "summary", "reversible", "irreversible_reason"} <= set(s) and isinstance(s.get("reversible"), bool)
                      and (s.get("irreversible_reason") is None) == s.get("reversible") for s in steps),
              f"{name}: v1.1 lifecycle.plan {action}")
        plans[action] = steps or []
    check((sorted(p.name for p in recdir.glob("*.json")) if recdir.is_dir() else []) == rec_before, f"{name}: v1.1 plan wrote no receipt")
    check((settings_file.read_bytes() if settings_file.is_file() else None) == set_before, f"{name}: v1.1 plan touched no settings")

    # shortcut: target is an empty object and repair is the applied action; the spec fixes
    # neither, so revisit when the contract pins a target schema or a safer apply action.
    action = "repair"
    oid = f"conf11-{name[:4]}-apply1"
    ap = jget(call(name, "lifecycle.apply", {"action": action, "target": {"app_root": str(tmp / "app")}}, oid, rpp="1.1")); raw.append(json.dumps(ap))
    check(ap.get("ok") and ap.get("operation_id") == oid, f"{name}: v1.1 lifecycle.apply")
    rpath = recdir / f"{oid}.json"
    check(rpath.is_file(), f"{name}: v1.1 lifecycle receipt file")
    rec = jget(rpath.read_text("utf-8")) if rpath.is_file() else {}
    check(rec.get("operation_id") == oid and rec.get("action") == action and rec.get("status") == "applied",
          f"{name}: v1.1 applied receipt fields")
    usage = rec.get("usage")
    if usage is not None:
        check(isinstance(usage, dict) and all(isinstance(usage.get(k), int) and not isinstance(usage.get(k), bool) and usage.get(k) >= 0
                  for k in USAGE_KEYS), f"{name}: v1.1 receipt usage shape")

    ap2 = jget(call(name, "lifecycle.apply", {"action": action, "target": {"app_root": str(tmp / "app")}}, oid, rpp="1.1")); raw.append(json.dumps(ap2))
    check(ap2.get("ok") and ap2.get("result") == ap.get("result"), f"{name}: v1.1 apply replay idempotent")
    cf = jget(call(name, "lifecycle.apply", {"action": "update" if action != "update" else "install", "target": {"app_root": str(tmp / "app")}}, oid, rpp="1.1"))
    raw.append(json.dumps(cf)); err_ok(cf, "LIFECYCLE_CONFLICT", f"{name}: v1.1 LIFECYCLE_CONFLICT")

    st = jget(call(name, "lifecycle.status", {"operation_id": oid}, rpp="1.1")); raw.append(json.dumps(st))
    rc = jget(call(name, "operation.receipt", {"operation_id": oid}, rpp="1.1")); raw.append(json.dumps(rc))
    check(st.get("ok") and rc.get("ok") and st.get("result") == rc.get("result"), f"{name}: v1.1 lifecycle.status == operation.receipt")
    miss = jget(call(name, "operation.receipt", {"operation_id": "conf11-missing1"}, rpp="1.1")); raw.append(json.dumps(miss))
    err_ok(miss, "RECEIPT_NOT_FOUND", f"{name}: v1.1 RECEIPT_NOT_FOUND")

    # rollback reverses the applied op; an irreversible plan must fail it closed.
    if any(not s.get("reversible") for s in plans.get(action, [])):
        rb = jget(call(name, "lifecycle.rollback", {"operation_id": oid}, rpp="1.1")); raw.append(json.dumps(rb))
        err_ok(rb, "ROLLBACK_UNAVAILABLE", f"{name}: v1.1 ROLLBACK_UNAVAILABLE for irreversible plan")
    else:
        rb = jget(call(name, "lifecycle.rollback", {"operation_id": oid}, rpp="1.1")); raw.append(json.dumps(rb))
        check(rb.get("ok"), f"{name}: v1.1 lifecycle.rollback")
        rb2 = jget(call(name, "lifecycle.rollback", {"operation_id": oid}, rpp="1.1")); raw.append(json.dumps(rb2))
        check(rb2.get("ok") and rb2.get("result") == rb.get("result"), f"{name}: v1.1 rollback idempotent")
        rc2 = jget(call(name, "operation.receipt", {"operation_id": oid}, rpp="1.1")); raw.append(json.dumps(rc2))
        check(rc2.get("ok") and (rc2.get("result") or {}).get("status") == "rolled_back", f"{name}: v1.1 rolled-back receipt status")

    # a v1.1-only op requested under rpp 1.0 must name the required version.
    u11 = jget(call(name, "lifecycle.plan", {"action": "install", "target": {"app_root": str(tmp / "app")}}, rpp="1.0")); raw.append(json.dumps(u11))
    e = u11.get("error") or {}
    check(u11.get("ok") is False and e.get("code") == "UNSUPPORTED_OP" and "1.1" in str(e.get("hint", "")),
          f"{name}: v1.1 op under rpp 1.0 -> UNSUPPORTED_OP with version hint")


# Hub parser is the final judge of every envelope the servers emitted.
(tmp / "raw.json").write_text(json.dumps(raw), encoding="utf8")
hub = ROOT / "respected-hub"
js = (f"import {{parseResponse,negotiate}} from './src/rpp.ts';import fs from 'node:fs';"
      f"const raw=JSON.parse(fs.readFileSync({json.dumps(str(tmp/'raw.json'))},'utf8'));"
      "let bad=raw.filter(t=>parseResponse(t).error?.code==='BAD_RESPONSE');"
      "const man=raw.map(t=>JSON.parse(t)).filter(j=>j.result&&j.result.protocol);"
      "const neg=man.map(j=>negotiate(j.result).compatible);"
      "console.log(JSON.stringify({bad:bad.length,neg}))")
(hub / "_conf.mts").write_text(js, encoding="utf8")
try:
    o = subprocess.run(["node", "_conf.mts"], cwd=hub, capture_output=True, text=True)
finally:
    (hub / "_conf.mts").unlink(missing_ok=True)
try:
    j = json.loads(o.stdout.strip().splitlines()[-1]); check(j["bad"] == 0, f"Hub parser rejected {j['bad']} server envelopes"); check(all(j["neg"]) and j["neg"], "Hub negotiate incompatible")
except Exception:
    fails.append("Hub parse run failed: " + (o.stderr or o.stdout)[-300:])
if require_v11 and skipped:
    check(False, "v1.1 required but skipped for: " + ", ".join(skipped))
if expect_v10_only and len(skipped) != len(SERVERS):
    check(False, "v1.0-only skip required for every server; skipped: " + ", ".join(skipped))
print(json.dumps({"failures": fails, "responses_checked": len(raw), "v11_skipped": skipped}, indent=1))
sys.exit(1 if fails else 0)
