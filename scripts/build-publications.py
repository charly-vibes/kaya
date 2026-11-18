#!/usr/bin/env python3
"""Render the publication tree into the book artifact.

Purpose: publish docs/public as HTML inside the mdBook output so the deployed
site contains the client-facing publication pages as well as the book.
Responsibilities: build a throwaway mdBook source from docs/public, render it
into book/public, and copy the structured export beside the pages.
Rationale: GitHub Pages serves one site per repository; a second upload would
replace the book. Rendering through mdBook keeps the Markdown sources and the
deployed pages identical instead of maintaining a second HTML copy.
"""

from __future__ import annotations

import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

PUBLIC = Path("docs/public")
BOOK_OUT = Path("book/public")
EXPORT = "publication.json"


def title_of(path: Path) -> str:
    for line in path.read_text(encoding="utf-8").splitlines():
        match = re.match(r"^#\s+(.+)$", line)
        if match:
            return match.group(1).strip()
    return path.stem


def main() -> int:
    if not PUBLIC.is_dir():
        print(f"build-publications: {PUBLIC} is missing", file=sys.stderr)
        return 1
    pages = sorted(path for path in PUBLIC.glob("*.md") if path.name != "SUMMARY.md")
    if not pages:
        print(f"build-publications: {PUBLIC} has no Markdown pages", file=sys.stderr)
        return 1

    with tempfile.TemporaryDirectory(prefix="kaya-publications-") as tmp:
        root = Path(tmp)
        src = root / "src"
        src.mkdir()
        for page in pages:
            shutil.copy2(page, src / page.name)
        summary = ["# Summary", ""]
        summary.extend(f"- [{title_of(page)}]({page.name})" for page in pages)
        (src / "SUMMARY.md").write_text("\n".join(summary) + "\n", encoding="utf-8")
        (root / "book.toml").write_text(
            '[book]\ntitle = "kaya publications"\nsrc = "src"\n',
            encoding="utf-8",
        )

        shutil.rmtree(BOOK_OUT, ignore_errors=True)
        BOOK_OUT.parent.mkdir(parents=True, exist_ok=True)
        process = subprocess.run(
            ["mdbook", "build", str(root), "-d", str(BOOK_OUT.resolve())],
            check=False,
        )
        if process.returncode != 0:
            print("build-publications: mdbook failed", file=sys.stderr)
            return process.returncode

    export = PUBLIC / EXPORT
    if export.is_file():
        shutil.copy2(export, BOOK_OUT / EXPORT)
    print(f"build-publications: rendered {len(pages)} page(s) into {BOOK_OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())