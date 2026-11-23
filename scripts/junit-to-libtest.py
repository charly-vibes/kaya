#!/usr/bin/env python3
"""Convert a nextest JUnit XML report into the libtest JSONL kaya imports.

Purpose: give a repository on a stable Rust toolchain a way to feed
`kaya import test` the run report nextest produces.
Responsibilities: read one JUnit XML report, name every test and the outcome
its element states, and write the libtest JSONL records the test-runner
adapter consumes; refuse a report it cannot read without writing anything.
Rationale: stable libtest cannot emit JSON at all — `--format json` is
nightly-only and stable libtest rejects `-Z unstable-options` outright
(rust-lang/rust#109044) — so the stable path runs nextest and converts the
JUnit XML nextest does emit.
"""

from __future__ import annotations

import argparse
import json
import sys
import xml.etree.ElementTree as ET
from pathlib import Path

# The JUnit elements that name a test's outcome. A case with none of them
# passed; `<failure>` and `<error>` both fail.
FAILED = {"failure", "error"}
SKIPPED = {"skipped"}


def local_name(tag: str) -> str:
    """The element name without its XML namespace."""
    return tag.rpartition("}")[2]


def outcome(case: ET.Element) -> str:
    """The libtest event naming this test case's outcome.

    Only the outcome elements count: nextest also writes `<flakyFailure>` and
    `<rerunFailure>` for a test's earlier attempts, and those record history
    rather than the result the report states.
    """
    children = {local_name(child.tag) for child in case}
    if children & FAILED:
        return "failed"
    if children & SKIPPED:
        return "ignored"
    return "ok"


def record(case: ET.Element) -> dict[str, str]:
    """One libtest JSONL record for one test case.

    The name carries the class name nextest reports beside the test name, so
    two binaries that run a test of the same name stay distinguishable — the
    same reason libtest qualifies its own names.
    """
    name = case.get("name", "")
    classname = case.get("classname")
    if classname:
        name = f"{classname}::{name}"
    return {"type": "test", "event": outcome(case), "name": name}


def convert(report: Path) -> list[dict[str, str]]:
    """Every test record the report states, in report order.

    Raises ET.ParseError when the report is not well-formed XML, and
    ValueError when it states no test case at all.
    """
    tree = ET.parse(report)
    cases = (node for node in tree.getroot().iter() if local_name(node.tag) == "testcase")
    records = [record(case) for case in cases]
    if not records:
        raise ValueError("the report states no test case")
    return records


def write_atomically(output: Path, body: str) -> None:
    """Write `body` to `output` so a reader never sees a partial report.

    The report is written to a sibling temporary file and renamed into place:
    a rename is atomic, so the output is either the previous report or the
    whole new one. Writing in place would leave a truncated file whose
    surviving records still parse - a report that silently under-counts the
    run.
    """
    temporary = output.with_name(output.name + ".partial")
    try:
        temporary.write_text(body, encoding="utf-8")
        temporary.replace(output)
    except OSError:
        temporary.unlink(missing_ok=True)
        raise


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("report", help="JUnit XML report to read")
    parser.add_argument("output", help="libtest JSONL report to write")
    args = parser.parse_args()

    try:
        records = convert(Path(args.report))
    except ET.ParseError as err:
        print(f"junit-to-libtest: {args.report} is not valid XML: {err}", file=sys.stderr)
        return 1
    except OSError as err:
        print(f"junit-to-libtest: cannot read {args.report}: {err}", file=sys.stderr)
        return 1
    except ValueError as err:
        print(f"junit-to-libtest: {args.report} is not a test report: {err}", file=sys.stderr)
        return 1

    body = "".join(json.dumps(entry) + "\n" for entry in records)
    try:
        write_atomically(Path(args.output), body)
    except OSError as err:
        print(f"junit-to-libtest: cannot write {args.output}: {err}", file=sys.stderr)
        return 1

    print(f"junit-to-libtest: wrote {len(records)} test record(s) to {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
