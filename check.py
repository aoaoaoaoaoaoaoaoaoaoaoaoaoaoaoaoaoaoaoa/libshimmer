#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.14"
# dependencies = []
# ///
"""Run the Rust gate declared in `[workspace.metadata.rust-starter]`."""

import argparse
import subprocess
import tomllib
from dataclasses import dataclass
from pathlib import Path, PurePosixPath
from typing import Never

ROOT = Path(__file__).resolve().parent
METADATA = "workspace.metadata.rust-starter"
IGNORED_DIRS = frozenset({".direnv", ".git", ".hg", ".jj", ".svn", "__pycache__", "node_modules", "target", "vendor"})

type Command = tuple[str, ...]


@dataclass(frozen=True, slots=True)
class SourceFiles:
    max_lines: int = 3000
    include: tuple[str, ...] = ("*.rs", "**/*.rs")
    exclude: tuple[str, ...] = ()


@dataclass(frozen=True, slots=True)
class Gate:
    canonicalize: tuple[Command, ...]
    format: Command
    clippy: Command
    test: Command
    doc: Command | None
    source_files: SourceFiles


def invalid(key: str, expected: str) -> Never:
    raise SystemExit(f"[check] invalid {METADATA}.{key}: expected {expected}")


def strings(raw: object, key: str, *, allow_empty: bool = False) -> tuple[str, ...]:
    if isinstance(raw, list):
        items = tuple(item for item in raw if isinstance(item, str) and item)
        if len(items) == len(raw) and (items or allow_empty):
            return items
    invalid(key, "a string list" if allow_empty else "a non-empty string list")


def commands(raw: object, key: str) -> tuple[Command, ...]:
    if isinstance(raw, list) and raw:
        return tuple(strings(command, f"{key}[{index}]") for index, command in enumerate(raw, start=1))
    invalid(key, "a non-empty list of commands")


def source_files(raw: object) -> SourceFiles:
    default = SourceFiles()
    if raw is None:
        return default
    if not isinstance(raw, dict):
        invalid("source_files", "a table")
    max_lines = raw.get("max_lines", default.max_lines)
    if isinstance(max_lines, bool) or not isinstance(max_lines, int) or max_lines <= 0:
        invalid("source_files.max_lines", "a positive integer")
    include = raw.get("include")
    exclude = raw.get("exclude")
    return SourceFiles(
        max_lines,
        default.include if include is None else strings(include, "source_files.include"),
        default.exclude if exclude is None else strings(exclude, "source_files.exclude", allow_empty=True),
    )


def load_gate() -> Gate:
    manifest = tomllib.loads((ROOT / "Cargo.toml").read_text(encoding="utf-8"))
    metadata = manifest["workspace"]["metadata"]["rust-starter"]
    doc = metadata.get("doc_command")
    return Gate(
        commands(metadata.get("canonicalize_commands"), "canonicalize_commands"),
        strings(metadata.get("format_command"), "format_command"),
        strings(metadata.get("clippy_command"), "clippy_command"),
        strings(metadata.get("test_command"), "test_command"),
        None if doc is None else strings(doc, "doc_command"),
        source_files(metadata.get("source_files")),
    )


def run(name: str, command: Command) -> None:
    print(f"[check] {name}: {' '.join(command)}", flush=True)
    if code := subprocess.run(command, cwd=ROOT, check=False).returncode:
        raise SystemExit(code)


def canonicalize(gate: Gate) -> None:
    for index, command in enumerate(gate.canonicalize, start=1):
        run(f"canonicalize.{index}", command)


def matches(path: PurePosixPath, pattern: str) -> bool:
    # `PurePath.match` treats `**` as one segment, so `**/x` misses root-level files; retry without the prefix.
    return path.match(pattern) or (pattern.startswith("**/") and path.match(pattern.removeprefix("**/")))


def enforce_line_cap(policy: SourceFiles) -> None:
    print(f"[check] source-files: max {policy.max_lines} lines", flush=True)
    violations: list[tuple[str, int]] = []
    for directory, dirnames, filenames in ROOT.walk():
        dirnames[:] = sorted(name for name in dirnames if name not in IGNORED_DIRS)
        for filename in sorted(filenames):
            path = directory / filename
            relative = PurePosixPath(path.relative_to(ROOT).as_posix())
            if not any(matches(relative, pattern) for pattern in policy.include):
                continue
            if any(matches(relative, pattern) for pattern in policy.exclude):
                continue
            if (lines := len(path.read_text(encoding="utf-8").splitlines())) > policy.max_lines:
                violations.append((relative.as_posix(), lines))
    if violations:
        print(f"[check] source-files: {len(violations)} file(s) exceed the limit", flush=True)
        for relative, lines in violations:
            print(f"[check] source-files: {relative}: {lines} lines", flush=True)
        print("[check] source-files: split by responsibility into modules (mod / #[path]); do not use include! except for build-script output", flush=True)
        raise SystemExit(1)


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the manifest-owned Rust gate.")
    parser.add_argument(
        "mode",
        nargs="?",
        choices=("check", "verify", "deep", "fix"),
        default="check",
        help="check: canonicalize, then verify; verify: never write; deep: check plus docs; fix: canonicalize only",
    )
    mode: str = parser.parse_args().mode
    gate = load_gate()
    if mode == "fix":
        canonicalize(gate)
        return
    enforce_line_cap(gate.source_files)
    if mode != "verify":
        canonicalize(gate)
    run("fmt", gate.format)
    run("clippy", gate.clippy)
    run("test", gate.test)
    if mode == "deep" and gate.doc is not None:
        run("doc", gate.doc)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        raise SystemExit(130) from None
