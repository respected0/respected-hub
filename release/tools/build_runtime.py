"""Build the Hub-managed Python runtime bundle (catalog entry `runtime-<python version>`).

    python tools/build_runtime.py [--out DIR]

Copies the interpreter this script runs under (a regular python.org install; nothing is
downloaded), prunes what the products never need, and adds the pure dependency closure of
agent-framework-core (the only third-party package the Orchestrator imports). The result is
an unsigned, hash-listed bundle like the product bundles, with every file under `runtime/`.
"""
import argparse, hashlib, importlib.metadata as md, json, re, shutil, sys
import sys as _sys
_sys.path.insert(0, str(__import__("pathlib").Path(__file__).resolve().parent))
from bundle_signing import sign_manifest
from pathlib import Path

from ecosystem_root import ECO as ROOT
BASE = Path(sys.base_prefix)
ROOTS = ["agent-framework-core"]
KEEP_TOP = {"python.exe", "pythonw.exe", "python3.dll", "LICENSE.txt"}
DROP_LIB = {"test", "tests", "idlelib", "turtledemo", "ensurepip", "tkinter", "lib2to3", "site-packages", "__pycache__", "venv"}
DROP_DLL = ("_tkinter", "tcl", "tk")  # tkinter and its Tcl/Tk are unused


def norm(name: str) -> str:
    return re.sub(r"[-_.]+", "-", name).lower()


def closure() -> list[md.Distribution]:
    """Distributions reachable from ROOTS without extras, honouring environment markers."""
    seen, queue, found = set(), list(ROOTS), []
    while queue:
        name = norm(queue.pop())
        if name in seen:
            continue
        seen.add(name)
        dist = md.distribution(name)
        found.append(dist)
        for requirement in dist.requires or []:
            req_name, _, marker = requirement.partition(";")
            if "extra ==" in marker:
                continue
            if marker.strip():
                from packaging.markers import Marker
                if not Marker(marker.strip()).evaluate():
                    continue
            queue.append(re.split(r"[ <>=!~\[(]", req_name.strip(), maxsplit=1)[0])
    return found


def copy_tree(source: Path, target: Path, skip) -> None:
    for item in sorted(source.rglob("*")):
        relative = item.relative_to(source)
        if skip(relative) or not item.is_file() or item.is_symlink() or item.suffix in {".pyc", ".pyo"} or "__pycache__" in relative.parts:
            continue
        destination = target / relative
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(item, destination)


def sha256_file(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def build(out: Path) -> Path:
    version = "%d.%d.%d" % sys.version_info[:3]
    target = out / f"runtime-{version}"
    if target.exists():
        shutil.rmtree(target)
    payload = target / "payload" / "runtime"
    payload.mkdir(parents=True)
    for name in BASE.iterdir():
        if name.is_file() and (name.name in KEEP_TOP or re.fullmatch(r"(python\d+|vcruntime\w+|msvcp\w+|concrt\w*)\.dll", name.name, re.I)):
            shutil.copyfile(name, payload / name.name)
    copy_tree(BASE / "DLLs", payload / "DLLs", lambda r: r.name.lower().startswith(DROP_DLL) or r.suffix.lower() == ".lib")
    copy_tree(BASE / "Lib", payload / "Lib", lambda r: bool(DROP_LIB & set(r.parts)))
    site = payload / "Lib" / "site-packages"
    for dist in closure():
        root = Path(dist.locate_file(""))
        for file in dist.files or []:
            source = root / file
            if source.is_file() and not source.is_symlink() and "__pycache__" not in file.parts and file.suffix not in {".pyc", ".pyo"}:
                destination = site / Path(*file.parts)
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, destination)
    files = []
    for path in sorted((target / "payload").rglob("*")):
        if path.is_file():
            files.append({"path": path.relative_to(target / "payload").as_posix(), "size": path.stat().st_size, "sha256": sha256_file(path)})
    digest = hashlib.sha256(json.dumps(files, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")).hexdigest()
    manifest = {"schema": 1, "product": "runtime", "version": version, "files": files, "bundle_digest": digest, "signature": None}
    (target / "bundle.json").write_text(json.dumps(sign_manifest(manifest), indent=1), encoding="utf-8")  # dev key unless RESPECTED_SIGNING_KEY is set
    return target


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", type=Path, default=ROOT / "catalog")
    built = build(parser.parse_args().out)
    size = sum(f.stat().st_size for f in (built / "payload").rglob("*") if f.is_file())
    print(built, f"{size / 1048576:.1f} MB")
