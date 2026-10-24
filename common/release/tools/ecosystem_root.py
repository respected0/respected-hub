"""Locate the ecosystem folder: the one that holds respected-hub, respected-brain and respected-orchestrator side by side.
RESPECTED_ECOSYSTEM_ROOT overrides the search."""
import os
from pathlib import Path


def find() -> Path:
    given = os.environ.get("RESPECTED_ECOSYSTEM_ROOT")
    if given:
        return Path(given).resolve()
    for folder in Path(__file__).resolve().parents:
        if (folder / "respected-hub").is_dir() and (folder / "respected-brain").is_dir():
            return folder
    raise SystemExit("ecosystem folder not found: put respected-hub next to respected-brain or set RESPECTED_ECOSYSTEM_ROOT")


ECO = find()
