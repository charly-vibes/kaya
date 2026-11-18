#!/usr/bin/env python3
"""Check that release claims agree with the crate.

Purpose: catch a status page or README that contradicts the release the
repository actually ships.
Responsibilities: read the version from Cargo.toml, require docs/src/status.md
to name that release, and refuse stale claims such as "no release tagged" or a
deferred crates.io channel.
Rationale: the claim and the manifest are both in the repository; a gate is the
only way they stay equal without a network call.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

CARGO = Path("Cargo.toml")
STATUS = Path("docs/src/status.md")
README = Path("README.md")

# Phrases that were true once and must not survive a release.
FORBIDDEN = [
    "no release tagged",
    "crates.io publication is deferred",
    "kept current by",
]


def crate_version() -> str:
    match = re.search(
        r'^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"', CARGO.read_text(), re.MULTILINE
    )
    if match is None:
        raise SystemExit(f"check-release-claims: no version in {CARGO}")
    return match.group(1)


def main() -> int:
    version = crate_version()
    status = STATUS.read_text(encoding="utf-8")
    failures = 0

    if f"v{version}" not in status:
        print(
            f"check-release-claims: {STATUS} does not name v{version}",
            file=sys.stderr,
        )
        failures += 1

    release_row = next(
        (line for line in status.splitlines() if line.startswith("| Release |")),
        None,
    )
    if release_row is None:
        print(f"check-release-claims: {STATUS} has no Release row", file=sys.stderr)
        failures += 1
    elif f"v{version}" not in release_row.split("|")[2]:
        print(
            f"check-release-claims: the Release row status does not name v{version}",
            file=sys.stderr,
        )
        failures += 1

    for path in (STATUS, README):
        text = path.read_text(encoding="utf-8")
        lowered = text.lower()
        for phrase in FORBIDDEN:
            if phrase in lowered:
                print(
                    f"check-release-claims: {path} still claims \"{phrase}\"",
                    file=sys.stderr,
                )
                failures += 1

    if failures:
        return 1
    print(f"check-release-claims: claims agree with v{version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())