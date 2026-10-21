"""Build, sign and publish a Respected release on GitHub Releases (the Hub downloads products from there).

    python publish_release.py build --key VAULT [--only hub,secondbrain,orchestrator,runtime] [--skip-hub-build]
        -> <ecosystem>/dist/release/: signed product archives, the three Hub installers, entries.json
    python publish_release.py catalog --key VAULT [--channel beta|stable] [--base FILE|URL] [--min-hub VERSION]
        -> dist/release/catalog.json + catalog.json.sig, merged into the index that is published now
    python publish_release.py upload [--yes]
        -> gh release create/upload per repository: products first, the catalog last. Without --yes it only prints.
    python publish_release.py local-catalog --bundle DIR [--bundle DIR ...] --out DIR    (tests: file:// URLs, development key)

Layout on GitHub (the Hub reads only the catalog URL; everything else is named by the signed catalog):
    respected0/respected-hub          tag v<hub version>   RespectedHub/-SecondBrain/-Orchestrator-Windows-Setup.exe, runtime-<python>.zip
    respected0/respected-secondbrain  tag v<version>       secondbrain-<version>.zip
    respected0/respected-orchestrator tag v<version>       orchestrator-<version>.zip
    respected0/respected-hub          tag catalog          catalog.json + catalog.json.sig (replaced on every release)
An archive is one signed bundle folder (bundle.json + payload/) zipped at its root. The index carries each archive's size and
SHA-256 and is signed with the release key over b"respected-catalog-v1\\n" + the exact file bytes ("<key_id> <base64>" line).
"""
import argparse, base64, hashlib, json, re, shutil, subprocess, sys, time, urllib.request, zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import bundle_signing as bs
from ecosystem_root import ECO

TOOLS = Path(__file__).resolve().parent
HUB = ECO / "respected-hub"
OUT = ECO / "dist" / "release"
STAGE = ECO / ".build-cache" / "release-bundles"
OWNER = "respected0"
INDEX_URL = f"https://github.com/{OWNER}/respected-hub/releases/download/catalog/catalog.json"
DOMAIN = b"respected-catalog-v1\n"
REPOS = {"hub": "respected-hub", "secondbrain": "respected-secondbrain", "orchestrator": "respected-orchestrator", "runtime": "respected-hub"}
INSTALLERS = ("RespectedHub", "RespectedSecondBrain", "RespectedOrchestrator")


def run(command, **kwargs):
    done = subprocess.run(command, **kwargs)
    if done.returncode:
        sys.exit(f"failed ({done.returncode}): {' '.join(map(str, command))}")
    return done


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


def source_version(product: str) -> str:
    """The human version written in the repository (SemVer, e.g. 0.1.0-beta.1): it names the release tag."""
    if product in ("hub", "runtime"):
        return json.loads((HUB / "src-tauri" / "tauri.conf.json").read_text("utf-8-sig"))["version"]
    text = (ECO / REPOS[product] / "pyproject.toml").read_text("utf-8")
    return re.search(r'(?m)^version\s*=\s*"([^"]+)"', text).group(1)


def zip_bundle(bundle: Path, archive: Path) -> None:
    archive.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(archive, "w", zipfile.ZIP_DEFLATED, compresslevel=9) as handle:
        for path in sorted(bundle.rglob("*")):
            if path.is_file():
                handle.write(path, path.relative_to(bundle).as_posix())


def bundle_version(bundle: Path) -> str:
    return json.loads((bundle / "bundle.json").read_text("utf-8"))["version"]


def entry(file: Path, version: str, repo: str, tag: str) -> dict:
    return {"version": version, "file": file.name, "sha256": sha256(file), "size": file.stat().st_size, "repo": repo, "tag": tag}


def build(args) -> None:
    only = set(args.only.split(",")) if args.only else set(REPOS)
    unknown = only - set(REPOS)
    if unknown:
        sys.exit(f"unknown components: {sorted(unknown)}")
    py = [sys.executable, "-B"]
    key = ["--key", str(args.key)]
    if STAGE.exists():
        shutil.rmtree(STAGE)
    STAGE.mkdir(parents=True)
    OUT.mkdir(parents=True, exist_ok=True)
    entries = json.loads((OUT / "entries.json").read_text("utf-8")) if (OUT / "entries.json").is_file() else {}
    hub_tag = "v" + source_version("hub")
    if "hub" in only:
        if not args.skip_hub_build:
            run(py + [str(TOOLS / "build_hub_installer.py"), "--release-key", str(args.key)])
        for name in INSTALLERS:
            shutil.copyfile(ECO / "dist" / f"{name}-Windows-Setup.exe", OUT / f"{name}-Windows-Setup.exe")
        entries["hub"] = entry(OUT / "RespectedHub-Windows-Setup.exe", source_version("hub"), REPOS["hub"], hub_tag)
    if "orchestrator" in only:
        run(py + [str(TOOLS / "build_bundle.py"), "orchestrator", "--out", str(STAGE), *key])
        bundle = next(STAGE.glob("orchestrator-*"))
        archive = OUT / f"orchestrator-{bundle_version(bundle)}.zip"
        zip_bundle(bundle, archive)
        entries["orchestrator"] = entry(archive, bundle_version(bundle), REPOS["orchestrator"], "v" + source_version("orchestrator"))
    if "secondbrain" in only:
        done = subprocess.run(py + [str(TOOLS / "build_brain_dist.py")], capture_output=True, text=True)
        if done.returncode:
            sys.exit(done.stderr)
        run(py + [str(TOOLS / "build_bundle.py"), "secondbrain-dist", "--dist", done.stdout.strip().splitlines()[-1], "--out", str(STAGE), *key])
        bundle = next(STAGE.glob("secondbrain-dist-*"))
        archive = OUT / f"secondbrain-{bundle_version(bundle)}.zip"
        zip_bundle(bundle, archive)
        entries["secondbrain"] = entry(archive, bundle_version(bundle), REPOS["secondbrain"], "v" + source_version("secondbrain"))
    if "runtime" in only:
        import os
        run(py + [str(TOOLS / "build_runtime.py"), "--out", str(STAGE)], env=dict(os.environ, RESPECTED_SIGNING_KEY=str(args.key)))
        bundle = next(STAGE.glob("runtime-*"))
        archive = OUT / f"runtime-{bundle_version(bundle)}.zip"
        zip_bundle(bundle, archive)
        entries["runtime"] = entry(archive, bundle_version(bundle), REPOS["runtime"], hub_tag)
    (OUT / "entries.json").write_text(json.dumps(entries, indent=1), encoding="utf-8")
    for name, row in entries.items():
        print(f"{name:13} {row['version']:16} {row['file']}  ({row['size']} bytes)")


def url_of(row: dict) -> str:
    return row.get("url") or f"https://github.com/{OWNER}/{row['repo']}/releases/download/{row['tag']}/{row['file']}"


def load_base(base: str | None) -> dict:
    if base and Path(base).is_file():
        return json.loads(Path(base).read_text("utf-8"))
    try:
        with urllib.request.urlopen(base or INDEX_URL, timeout=30) as response:
            return json.loads(response.read())
    except Exception as error:  # first release: nothing is published yet
        print(f"no published catalog used as base ({error})")
        return {"schema": 1, "sequence": 0, "hub": {}, "products": {}}


def merged_catalog(base: dict, entries: dict, channel: str, min_hub: str | None) -> dict:
    index = {"schema": 1, "sequence": max(int(time.strftime("%Y%m%d%H%M%S", time.gmtime())), int(base.get("sequence", 0)) + 1),
             "published": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "hub": dict(base.get("hub") or {}),
             "products": {name: dict(node) for name, node in (base.get("products") or {}).items()}}
    for name, row in entries.items():
        release = {"version": row["version"], "url": url_of(row), "sha256": row["sha256"], "size": row["size"]}
        if min_hub and name != "hub":
            release["min_hub"] = min_hub
        target = index["hub"] if name == "hub" else index["products"].setdefault(name, {})
        target[channel] = release
    return index


def sign_index(index: dict, key: Path, out: Path) -> None:
    body = json.dumps(index, indent=1, ensure_ascii=False).encode("utf-8") + b"\n"
    private = bs.load_private(key) if key else bs.ensure_dev_key()
    line = f"{bs.key_id(bs.public_bytes(private))} {base64.b64encode(private.sign(DOMAIN + body)).decode('ascii')}\n"
    (out / "catalog.json").write_bytes(body)
    (out / "catalog.json.sig").write_text(line, encoding="ascii", newline="\n")


def catalog(args) -> None:
    entries = json.loads((OUT / "entries.json").read_text("utf-8"))
    index = merged_catalog(load_base(args.base), entries, args.channel, args.min_hub)
    sign_index(index, args.key, OUT)
    print(json.dumps(index, indent=1))


def gh(*command, run_it):
    print("gh " + " ".join(command))
    if run_it:
        return subprocess.run(["gh", *command]).returncode
    return 0


def upload(args) -> None:
    entries = json.loads((OUT / "entries.json").read_text("utf-8"))
    if not (OUT / "catalog.json").is_file():
        sys.exit("run `catalog` first")
    notes = TOOLS.parent / "docs" / "RELEASE_NOTES_BETA.md"
    releases: dict[tuple[str, str], list[Path]] = {}
    for name, row in entries.items():
        files = [OUT / f"{n}-Windows-Setup.exe" for n in INSTALLERS] if name == "hub" else [OUT / row["file"]]
        releases.setdefault((row["repo"], row["tag"]), []).extend(files)
    for (repo, tag), files in releases.items():  # products first: the catalog must never name a missing file
        exists = subprocess.run(["gh", "release", "view", tag, "-R", f"{OWNER}/{repo}"], capture_output=True).returncode == 0
        if not exists:
            gh("release", "create", tag, "-R", f"{OWNER}/{repo}", "--title", f"Respected {tag[1:]}", "--notes-file", str(notes),
               *(["--prerelease"] if "-" in tag else []), "--target", "main", run_it=args.yes)
        gh("release", "upload", tag, "-R", f"{OWNER}/{repo}", "--clobber", *map(str, files), run_it=args.yes)
    exists = subprocess.run(["gh", "release", "view", "catalog", "-R", f"{OWNER}/respected-hub"], capture_output=True).returncode == 0
    if not exists:
        gh("release", "create", "catalog", "-R", f"{OWNER}/respected-hub", "--title", "Respected katalog",
           "--notes", "Respected Hub'ın okuduğu imzalı ürün kataloğu. Elle indirmeniz gerekmez.", "--prerelease", "--target", "main", run_it=args.yes)
    gh("release", "upload", "catalog", "-R", f"{OWNER}/respected-hub", "--clobber", str(OUT / "catalog.json"), str(OUT / "catalog.json.sig"), run_it=args.yes)
    if not args.yes:
        print("\nnothing was uploaded; run again with --yes to publish")


def local_catalog(args) -> None:
    """A complete, development-signed catalog with file:// URLs (end-to-end tests of the Hub's download path)."""
    out = args.out
    out.mkdir(parents=True, exist_ok=True)
    entries = {}
    for bundle in args.bundle:
        manifest = json.loads((bundle / "bundle.json").read_text("utf-8"))
        archive = out / f"{manifest['product']}-{manifest['version']}.zip"
        zip_bundle(bundle, archive)
        entries[manifest["product"]] = dict(entry(archive, manifest["version"], "", ""), url=archive.resolve().as_uri())
    sign_index(merged_catalog({"sequence": 0}, entries, args.channel, None), None, out)
    print(out / "catalog.json")


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="action", required=True)
    b = sub.add_parser("build"); b.add_argument("--key", type=Path, required=True); b.add_argument("--only"); b.add_argument("--skip-hub-build", action="store_true")
    c = sub.add_parser("catalog"); c.add_argument("--key", type=Path, required=True); c.add_argument("--channel", choices=("beta", "stable"), default="beta")
    c.add_argument("--base"); c.add_argument("--min-hub")
    u = sub.add_parser("upload"); u.add_argument("--yes", action="store_true")
    l = sub.add_parser("local-catalog"); l.add_argument("--bundle", type=Path, action="append", required=True); l.add_argument("--out", type=Path, required=True)
    l.add_argument("--channel", choices=("beta", "stable"), default="beta")
    args = parser.parse_args()
    {"build": build, "catalog": catalog, "upload": upload, "local-catalog": local_catalog}[args.action](args)
