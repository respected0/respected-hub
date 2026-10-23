"""Build, sign and publish a Respected release on GitHub Releases (the Hub downloads products from there).

    python publish_release.py build --key VAULT [--only hub,brain,orchestrator] [--skip-hub-build]
        -> <ecosystem>/dist/release/: signed product archives (each holds the product's own installer), the installers, entries.json
    python publish_release.py catalog --key VAULT [--channel beta|stable] [--base FILE|URL] [--min-hub VERSION]
        -> dist/release/catalog.json + catalog.json.sig, merged into the index that is published now
    python publish_release.py upload [--yes]
        -> gh release create/upload per repository: products first, the catalog last. Without --yes it only prints.
    python publish_release.py local-catalog --bundle DIR [--bundle DIR ...] --out DIR    (tests: file:// URLs, development key)

Layout on GitHub (the Hub reads only the catalog URL; everything else is named by the signed catalog):
    respected0/respected-hub          tag v<hub version>   Respected-Hub-<version>-Windows-x64-Setup.exe
    respected0/respected-brain  tag v<version>       Respected-Brain-<version>-Windows-x64-Package.zip, Respected-Brain-<version>-Windows-x64-Setup.exe
    respected0/respected-orchestrator tag v<version>       Respected-Orchestrator-<version>-Windows-x64-Package.zip, Respected-Orchestrator-<version>-Windows-x64-Setup.exe
    respected0/respected-hub          tag catalog          catalog.json + catalog.json.sig (replaced on every release)
An archive is one signed bundle folder (bundle.json + payload/<Product>-Windows-Setup.exe) zipped at its root; the same installer is
also attached on its own for people who install without the Hub. The index carries each archive's size and
SHA-256 and is signed with the release key over b"respected-catalog-v1\\n" + the exact file bytes ("<key_id> <base64>" line).
"""
import argparse, base64, hashlib, json, re, shutil, subprocess, sys, time, urllib.request, zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))
from signing import bundle_signing as bs
from common.ecosystem_root import ECO

TOOLS = Path(__file__).resolve().parents[1]
HUB = ECO / "respected-hub"
OUT = ECO / "dist" / "release"
STAGE = ECO / ".build-cache" / "release-bundles"
OWNER = "respected0"
INDEX_URL = f"https://github.com/{OWNER}/respected-hub/releases/download/catalog/catalog.json"
DOMAIN = b"respected-catalog-v1\n"
REPOS = {"hub": "respected-hub", "brain": "respected-brain", "orchestrator": "respected-orchestrator"}
PRODUCT_INSTALLERS = {"hub": "RespectedHub", "brain": "RespectedBrain", "orchestrator": "RespectedOrchestrator"}
DISPLAY = {"hub": "Respected-Hub", "brain": "Respected-Brain", "orchestrator": "Respected-Orchestrator"}


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


def installer_path(name: str) -> Path:
    """Where `name`'s Windows installer was built: the Hub's in <ecosystem>/dist, a product's in its own repository's dist."""
    repo = {"RespectedHub": ECO / "dist", "RespectedBrain": ECO / REPOS["brain"] / "dist", "RespectedOrchestrator": ECO / REPOS["orchestrator"] / "dist"}[name]
    return repo / f"{name}-Windows-Setup.exe"


def installer_name(product: str, version: str) -> str:
    """The file people download: Respected-Hub-0.1.0-beta.1-Windows-x64-Setup.exe (inside catalog bundles the name stays fixed)."""
    return f"{DISPLAY[product]}-{version}-Windows-x64-Setup.exe"


def package_name(product: str, version: str) -> str:
    """The signed catalog package (what the Hub downloads): Respected-Brain-0.1.0-beta.1-Windows-x64-Package.zip."""
    return f"{DISPLAY[product]}-{version}-Windows-x64-Package.zip"


def source_version(product: str) -> str:
    """The human version written in the repository (SemVer, e.g. 0.1.0-beta.1): it names the release tag."""
    if product == "hub":
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


def drop_old_installers(product: str) -> None:
    # installers, catalog packages and the unversioned archives of earlier builds
    for pattern in (f"{DISPLAY[product]}-*-Windows-x64-Setup.exe", f"{DISPLAY[product]}-*-Windows-x64-Package.zip", f"{product}-*.zip"):
        for old in OUT.glob(pattern):
            old.unlink()


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
            run(py + [str(TOOLS / "build" / "build_hub_installer.py"), "--release-key", str(args.key)])
        drop_old_installers("hub")
        named = OUT / installer_name("hub", source_version("hub"))
        shutil.copyfile(installer_path("RespectedHub"), named)
        entries["hub"] = entry(named, source_version("hub"), REPOS["hub"], hub_tag)
    for product in ("orchestrator", "brain"):
        if product not in only:
            continue
        # a release is always built from the current source (--force): never a cached build of older code
        run(py + [str(TOOLS / "build" / "build_bundle.py"), product, "--force", "--out", str(STAGE), *key])
        name = {"orchestrator": "RespectedOrchestrator", "brain": "RespectedBrain"}[product]
        drop_old_installers(product)
        shutil.copyfile(installer_path(name), OUT / installer_name(product, source_version(product)))
        bundle = next(STAGE.glob(f"{product}-*"))
        archive = OUT / package_name(product, source_version(product))
        zip_bundle(bundle, archive)
        entries[product] = entry(archive, bundle_version(bundle), REPOS[product], "v" + source_version(product))
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


def merged_catalog(base: dict, entries: dict, channel: str, min_hub: str | None, notes: str | None = None) -> dict:
    index = {"schema": 1, "sequence": max(int(time.strftime("%Y%m%d%H%M%S", time.gmtime())), int(base.get("sequence", 0)) + 1),
             "published": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()), "hub": dict(base.get("hub") or {}),
             "products": {name: dict(node) for name, node in (base.get("products") or {}).items()}}
    for name, row in entries.items():
        release = {"version": row["version"], "url": url_of(row), "sha256": row["sha256"], "size": row["size"]}
        if min_hub and name != "hub":
            release["min_hub"] = min_hub
        if notes:  # short "what changed" text the Hub shows before the person updates; covered by the catalog signature
            release["notes"] = notes
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
    index = merged_catalog(load_base(args.base), entries, args.channel, args.min_hub, args.notes)
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
        installer = OUT / installer_name(name, row["tag"][1:])
        files = [installer] if name == "hub" else [installer, OUT / row["file"]]
        releases.setdefault((row["repo"], row["tag"]), []).extend(files)
    for (repo, tag), files in releases.items():  # products first: the catalog must never name a missing file
        exists = subprocess.run(["gh", "release", "view", tag, "-R", f"{OWNER}/{repo}"], capture_output=True).returncode == 0
        if not exists:
            gh("release", "create", tag, "-R", f"{OWNER}/{repo}", "--title", f"Respected {tag[1:]}", "--notes-file", str(notes),
               "--target", "main", run_it=args.yes)  # never --prerelease: the installers download the Hub from releases/latest
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
    c.add_argument("--base"); c.add_argument("--min-hub"); c.add_argument("--notes")
    u = sub.add_parser("upload"); u.add_argument("--yes", action="store_true")
    l = sub.add_parser("local-catalog"); l.add_argument("--bundle", type=Path, action="append", required=True); l.add_argument("--out", type=Path, required=True)
    l.add_argument("--channel", choices=("beta", "stable"), default="beta")
    args = parser.parse_args()
    {"build": build, "catalog": catalog, "upload": upload, "local-catalog": local_catalog}[args.action](args)
