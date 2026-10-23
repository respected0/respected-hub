"""Product contract conformance (docs/PRODUCT_CONTRACT.md): the same checks run against every product.

Runs from respected-hub with the standard library only:  python -m unittest discover -s tools/conformance -v (the three repos sit side by side)
Default target is each product's source checkout. To test a built program set
RESPECTED_CONFORMANCE_<PRODUCT>_EXE (PRODUCT = BRAIN or ORCHESTRATOR).
Every check points the product at temporary folders (RESPECTED_APP_DIR / RESPECTED_DATA_DIR); nothing touches a real install.
"""
import json
import os
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HUB = Path(__file__).resolve().parents[2]
ROOT = HUB.parent  # the folder holding the three repos
PRODUCTS = {
    "brain": ("respected-brain/src", "respected_brain"),
    "orchestrator": ("respected-orchestrator/src", "respected_orchestrator"),
}
SHARED_COMMANDS = ("setup", "repair", "uninstall", "config", "health", "rpp", "mcp")
REQUIRED_OPS = ("manifest", "health", "settings.schema", "settings.validate", "settings.apply", "settings.readback")
RETIRED_OPS = ("lifecycle.plan", "lifecycle.apply", "lifecycle.status", "lifecycle.rollback", "operation.receipt")
RESPONSE_KEYS = {"rpp", "ok", "operation_id", "result", "error"}
# a setting each product has, with a valid value and an invalid one
SAMPLE = {"brain": ("summary_provider", "codex", "nope"), "orchestrator": ("default_timeout_seconds", "1200", "5")}
INSTALLERS = {"brain": "respected-brain/packaging/windows/brain_setup.iss",
              "orchestrator": "respected-orchestrator/packaging/windows/orchestrator_setup.iss"}


def launcher(product):
    exe = os.environ.get(f"RESPECTED_CONFORMANCE_{product.upper()}_EXE")
    if exe:
        return [exe], dict(os.environ)
    src, module = PRODUCTS[product]
    return [sys.executable, "-m", module], dict(os.environ, PYTHONPATH=str(ROOT / src), PYTHONDONTWRITEBYTECODE="1")


class ProductContract(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)

    def run_product(self, product, args, stdin=None):
        command, env = launcher(product)
        env.update(RESPECTED_APP_DIR=str(self.root / product / "app"), RESPECTED_DATA_DIR=str(self.root / product / "data"),
                   USERPROFILE=str(self.root / "home"), HOME=str(self.root / "home"))
        return subprocess.run(command + args, input=stdin, capture_output=True, text=True, encoding="utf-8", env=env, timeout=120, cwd=self.root)

    def rpp(self, product, op, payload=None, version="1"):
        request = json.dumps({"rpp": version, "op": op, "payload": payload or {}, "operation_id": "operation-0001"})
        done = self.run_product(product, ["rpp"], request)
        self.assertTrue(done.stdout.strip(), f"{product} {op}: empty stdout, stderr={done.stderr[:300]}")
        return json.loads(done.stdout.strip().splitlines()[-1])

    def json_out(self, done):
        return json.loads(done.stdout)

    def test_version_names_the_product_and_installs_nothing(self):
        for product in PRODUCTS:
            with self.subTest(product=product):
                done = self.run_product(product, ["--version"])
                self.assertEqual(done.returncode, 0, done.stderr[:300])
                self.assertRegex(done.stdout.strip(), rf"^respected-{product} \S+$")
                self.assertFalse((self.root / product / "data").exists())

    def test_shared_commands_exist_and_retired_ones_do_not(self):
        for product in PRODUCTS:
            for name in SHARED_COMMANDS:
                with self.subTest(product=product, command=name):
                    self.assertEqual(self.run_product(product, [name, "--help"]).returncode, 0)
            for retired in (["update"], ["uninstall", "--keep-data"], ["uninstall", "--purge-data"]):
                with self.subTest(product=product, retired=retired):
                    self.assertEqual(self.run_product(product, retired).returncode, 2)

    def test_config_and_rpp_settings_are_the_same_settings(self):
        for product, (key, good, bad) in SAMPLE.items():
            with self.subTest(product=product):
                done = self.run_product(product, ["config", "set", f"{key}={good}"])
                self.assertEqual(done.returncode, 0, done.stdout + done.stderr)
                written = self.json_out(done)[key]
                self.assertEqual(self.json_out(self.run_product(product, ["config", "get", key])), {key: written})
                self.assertEqual(self.json_out(self.run_product(product, ["config"]))[key], written)
                self.assertEqual(self.rpp(product, "settings.readback")["result"]["settings"][key], written)
                refused = self.run_product(product, ["config", "set", f"{key}={bad}"])
                self.assertEqual(refused.returncode, 2)
                self.assertEqual(set(self.json_out(refused)["error"]), {"code", "message", "hint"})
                self.assertEqual(self.json_out(self.run_product(product, ["config", "get", key])), {key: written}, "a refused change writes nothing")

    def test_usage_errors_share_one_shape_and_exit_code(self):
        for product in PRODUCTS:
            with self.subTest(product=product):
                done = self.run_product(product, ["config", "set", "definitely_not_a_setting=1"])
                self.assertEqual(done.returncode, 2)
                self.assertEqual(set(self.json_out(done)), {"error"})
                self.assertEqual(set(self.json_out(done)["error"]), {"code", "message", "hint"})

    def test_schema_fields_carry_turkish_label_and_help(self):
        for product in PRODUCTS:
            with self.subTest(product=product):
                for field in self.rpp(product, "settings.schema")["result"]["fields"]:
                    self.assertTrue({"key", "type", "label", "help", "default", "secret"} <= set(field), field)
                    self.assertIn(field["type"], ("string", "integer", "boolean", "enum"))

    def test_rpp_speaks_one_version_with_one_envelope(self):
        for product in PRODUCTS:
            with self.subTest(product=product):
                for version in ("1.0", "1.1", "2"):
                    response = self.rpp(product, "manifest", version=version)
                    self.assertEqual((set(response), response["error"]["code"]), (RESPONSE_KEYS, "UNSUPPORTED_VERSION"))
                unknown = self.rpp(product, "does.not.exist")
                self.assertEqual((set(unknown), unknown["rpp"], unknown["error"]["code"]), (RESPONSE_KEYS, "1", "UNKNOWN_OP"))
                for op in REQUIRED_OPS:
                    self.assertNotEqual((self.rpp(product, op).get("error") or {}).get("code"), "UNKNOWN_OP", f"{product} lacks {op}")
                for op in RETIRED_OPS:
                    self.assertEqual(self.rpp(product, op)["error"]["code"], "UNKNOWN_OP", f"{product} still has {op}: installing is the installer's")

    def test_manifest_reports_product_install_state_and_roots(self):
        for product in PRODUCTS:
            with self.subTest(product=product):
                result = self.rpp(product, "manifest")["result"]
                self.assertEqual((result["product_id"], result["rpp"]), (f"respected-{product}", "1"))
                self.assertTrue(result["product_version"])
                self.assertEqual((result["installed"], result["install_state"]), (False, "absent"))
                self.assertEqual(set(result["roots"]), {"app", "data", "vault"})
                self.assertEqual(result["capabilities"], sorted(result["capabilities"]))
                self.assertTrue(set(REQUIRED_OPS) <= set(result["capabilities"]))

    def test_every_repo_carries_the_same_contract(self):
        copies = {name: (ROOT / name / "docs/PRODUCT_CONTRACT.md").read_bytes() for name in ("respected-hub", "respected-brain", "respected-orchestrator")}
        self.assertEqual(len(set(copies.values())), 1, "docs/PRODUCT_CONTRACT.md differs between the repos")

    def test_both_installers_share_the_same_switches_and_shared_pages(self):
        texts = {product: (ROOT / path).read_text(encoding="utf-8") for product, path in INSTALLERS.items()}
        shared = {product: text[text.index("{ ---- shared with"):text.index("{ ----", text.index("{ ---- shared with") + 10)] for product, text in texts.items()}
        normalise = lambda block: re.sub(r"\{ ---- shared with the \w+ installer ---- \}", "", block)
        self.assertEqual(normalise(shared["brain"]), normalise(shared["orchestrator"]), "the shared installer section differs")
        for product, text in texts.items():
            for token in ("/SET=", "'PURGE'", "'HUB'", "'HUBURL'", "Onar / Güncelle", "Ayarları değiştir", "Kaldır", "{code:UninstallArgs}",
                          "'uninstall --purge'", "WizardStyle=modern", "PrivilegesRequired=lowest", "DisableDirPage=yes"):
                self.assertIn(token, text, f"{product} installer lacks {token}")
            self.assertNotRegex(text, r"--package|--allow-unattested|--keep-data|update --", f"{product} installer still uses a retired command")


if __name__ == "__main__":
    unittest.main()
