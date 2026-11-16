#!/usr/bin/env python3
"""Render the Homebrew formula for a kaya release.

Usage: update-homebrew.py FORMULA_PATH VERSION TAG CHECKSUMS_PATH

Reads the release `checksums.txt`, requires a digest for every macOS/Linux
target the formula installs, and only then writes the formula — a partial
release aborts without touching the tap (DDL-9e7: re-rendering an
already-updated formula is a no-op, so a retry never republishes anything).
"""
import os
import sys

PLATFORMS = ("darwin_arm64", "darwin_amd64", "linux_arm64", "linux_amd64")
TOOL = "kaya"

formula_path, version, tag, checksums_path = sys.argv[1:5]

if tag != f"v{version}":
    print(
        f"ERROR: tag {tag!r} does not match version {version!r} (expected v{version})",
        file=sys.stderr,
    )
    sys.exit(1)

shas = {}
with open(checksums_path) as f:
    for line in f:
        parts = line.split()
        if len(parts) != 2:
            continue
        sha, name = parts
        name = name.rsplit("/", 1)[-1]
        for platform in PLATFORMS:
            if name == f"{TOOL}_{version}_{platform}.tar.gz":
                shas[platform] = sha

missing = [p for p in PLATFORMS if p not in shas]
if missing:
    print(
        f"ERROR: {checksums_path} has no {TOOL}_{version}_<platform>.tar.gz digest "
        f"for: {', '.join(missing)}",
        file=sys.stderr,
    )
    sys.exit(1)

base = f"https://github.com/charly-vibes/{TOOL}/releases/download/{tag}"

formula = f"""\
# typed: false
# frozen_string_literal: true

class Kaya < Formula
  desc "Research record compiler for AI-driven development"
  homepage "https://github.com/charly-vibes/kaya"
  version "{version}"
  license "Apache-2.0"

  on_macos do
    on_arm do
      url "{base}/{TOOL}_{version}_darwin_arm64.tar.gz"
      sha256 "{shas['darwin_arm64']}"
    end
    on_intel do
      url "{base}/{TOOL}_{version}_darwin_amd64.tar.gz"
      sha256 "{shas['darwin_amd64']}"
    end
  end

  on_linux do
    on_arm do
      if Hardware::CPU.is_64_bit?
        url "{base}/{TOOL}_{version}_linux_arm64.tar.gz"
        sha256 "{shas['linux_arm64']}"
      end
    end
    on_intel do
      url "{base}/{TOOL}_{version}_linux_amd64.tar.gz"
      sha256 "{shas['linux_amd64']}"
    end
  end

  def install
    bin.install "{TOOL}"
  end

  test do
    system "#{{bin}}/{TOOL}", "--version"
  end
end
"""

os.makedirs(os.path.dirname(formula_path), exist_ok=True)
with open(formula_path, "w") as f:
    f.write(formula)

print(f"Wrote {formula_path} (version {version})")
for platform, sha in shas.items():
    print(f"  {platform}: {sha[:16]}...")