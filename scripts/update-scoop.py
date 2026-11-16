#!/usr/bin/env python3
"""Render the Scoop manifest for a kaya release.

Usage: update-scoop.py MANIFEST_PATH VERSION TAG CHECKSUMS_PATH

Reads the release `checksums.txt`, requires the Windows archive digest, and
only then writes the manifest — a partial release aborts without touching the
bucket (DDL-9e7: re-rendering an already-updated manifest is a no-op, so a
retry never republishes anything).
"""
import json
import os
import sys

TOOL = "kaya"

manifest_path, version, tag, checksums_path = sys.argv[1:5]

if tag != f"v{version}":
    print(
        f"ERROR: tag {tag!r} does not match version {version!r} (expected v{version})",
        file=sys.stderr,
    )
    sys.exit(1)

sha_win = None
with open(checksums_path) as f:
    for line in f:
        parts = line.split()
        if len(parts) == 2 and parts[1].rsplit("/", 1)[-1] == f"{TOOL}_{version}_windows_amd64.zip":
            sha_win = parts[0]
            break

if sha_win is None:
    print(
        f"ERROR: {checksums_path} has no {TOOL}_{version}_windows_amd64.zip digest",
        file=sys.stderr,
    )
    sys.exit(1)

url = f"https://github.com/charly-vibes/{TOOL}/releases/download/{tag}/{TOOL}_{version}_windows_amd64.zip"

manifest = {
    "version": version,
    "description": "Research record compiler for AI-driven development",
    "homepage": "https://github.com/charly-vibes/kaya",
    "license": "Apache-2.0",
    "url": url,
    "hash": f"sha256:{sha_win}",
    "bin": f"{TOOL}.exe",
    "checkver": {
        "github": f"https://github.com/charly-vibes/{TOOL}"
    },
    "autoupdate": {
        "url": f"https://github.com/charly-vibes/{TOOL}/releases/download/v$version/{TOOL}_$version_windows_amd64.zip"
    },
}

os.makedirs(os.path.dirname(manifest_path), exist_ok=True)
with open(manifest_path, "w") as f:
    json.dump(manifest, f, indent=4)
    f.write("\n")

print(f"Wrote {manifest_path} (version {version})")
print(f"  url: {url}")
print(f"  sha256: {sha_win[:16]}...")