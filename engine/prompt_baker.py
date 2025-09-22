#!/usr/bin/env python3
"""Prompt baking utilities for Meta² engine.

Provides commands under ``meta2-engine prompts`` to help author, validate,
preview, and persist small sys-prompt artifacts that live alongside the engine
codebase. Prompts are stored as YAML files in ``prompts/`` by default.
"""

from __future__ import annotations

import argparse
import os
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Iterable, List, Optional


ROOT = Path(__file__).resolve().parent
PROMPTS_DIR = ROOT / "prompts"


@dataclass
class PromptRequest:
    prompt_id: str
    title: str
    sys_prompt: str
    author: str
    spec_ref: Optional[str]
    tags: List[str]
    notes: Optional[str]
    tests: List[str]
    dry_run: bool
    force: bool


def resolve_spec(explicit: Optional[str]) -> Optional[str]:
    if explicit:
        path = Path(explicit)
        if path.exists():
            return str(path)
        return explicit  # allow remote URLs or refs

    env_spec = os.getenv("META2_SPEC_PATH")
    if env_spec:
        path = Path(env_spec)
        if path.exists():
            return str(path)
        return env_spec

    candidates = [
        ROOT.parent / "Meta-orchestrator-API---Snapshots" / "latest.json",
        ROOT.parent.parent / "Meta-orchestrator-API---Snapshots" / "latest.json",
        ROOT / "latest.json",
    ]
    for candidate in candidates:
        if candidate.exists():
            return str(candidate)
    return "latest.json"


def ensure_prompt_dir() -> None:
    PROMPTS_DIR.mkdir(parents=True, exist_ok=True)


def load_prompt_text(inline: Optional[str], path: Optional[str]) -> str:
    if inline and path:
        raise ValueError("Provide either --prompt or --prompt-file, not both")
    if inline:
        return inline
    if path:
        file_path = Path(path)
        return file_path.read_text(encoding="utf-8")
    raise ValueError("A prompt string or prompt file is required")


def check_forbidden(prompt: str) -> None:
    forbidden_tokens = ["sk-", "-----BEGIN", "ssh-rsa", "PRIVATE KEY"]
    for token in forbidden_tokens:
        if token in prompt:
            raise ValueError(f"Prompt appears to contain a secret token pattern: {token}")


def build_yaml(request: PromptRequest) -> str:
    lines: List[str] = []
    lines.append(f"id: {request.prompt_id}")
    lines.append(f"title: {request.title}")
    lines.append("sys_prompt: |-")
    for line in request.sys_prompt.rstrip().splitlines():
        lines.append(f"  {line}")
    if not request.sys_prompt.endswith("\n"):
        lines.append("  ")  # ensure terminating newline block indicator

    if request.spec_ref:
        lines.append(f"spec_ref: {request.spec_ref}")
    lines.append(f"author: {request.author}")
    lines.append(
        "created_at: "
        + datetime.now(timezone.utc).replace(microsecond=0).isoformat()
    )
    lines.append("provenance:")
    lines.append("  source: meta2-engine")
    if request.notes:
        lines.append(f"  notes: {request.notes}")

    if request.tags:
        lines.append("tags:")
        for tag in request.tags:
            lines.append(f"  - {tag}")

    lines.append("tests:")
    if request.tests:
        for entry in request.tests:
            lines.append(f"  - {entry}")
    else:
        lines.append("  - echo healthz -> expect 200")

    return "\n".join(lines) + "\n"


def write_prompt_file(request: PromptRequest) -> Path:
    ensure_prompt_dir()
    output_path = PROMPTS_DIR / f"{request.prompt_id}.yaml"
    if output_path.exists() and not request.force:
        raise FileExistsError(
            f"Prompt file {output_path} already exists. Use --force to overwrite."
        )
    yaml_body = build_yaml(request)
    if request.dry_run:
        print(yaml_body)
        return output_path
    output_path.write_text(yaml_body, encoding="utf-8")
    return output_path


def parse_tests(raw: Iterable[str]) -> List[str]:
    out: List[str] = []
    for item in raw:
        cleaned = item.strip()
        if cleaned:
            out.append(cleaned)
    return out


def handle_bake(args: argparse.Namespace) -> None:
    sys_prompt = load_prompt_text(args.prompt, args.prompt_file)
    check_forbidden(sys_prompt)

    spec_ref = resolve_spec(args.spec)
    if not spec_ref and not args.allow_missing_spec:
        raise FileNotFoundError(
            "Spec reference could not be resolved. Provide --spec or set META2_SPEC_PATH,"
            " or pass --allow-missing-spec to skip."
        )

    request = PromptRequest(
        prompt_id=args.id,
        title=args.title,
        sys_prompt=sys_prompt,
        author=args.author,
        spec_ref=spec_ref,
        tags=[tag.strip() for tag in args.tags.split(",") if tag.strip()] if args.tags else [],
        notes=args.notes,
        tests=parse_tests(args.test),
        dry_run=args.dry_run,
        force=args.force,
    )

    output = write_prompt_file(request)

    if args.dry_run:
        print("--- dry run complete (no file written) ---")
    else:
        print(f"Prompt written to {output}")
        print("Next steps:")
        print(f"  - git add {output.relative_to(ROOT)}")
        print("  - git commit -m 'Add prompt' && git push")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Meta² prompt baking utilities")
    sub = parser.add_subparsers(dest="command", required=True)

    bake = sub.add_parser("bake", help="Create or update a prompt YAML artifact")
    bake.add_argument("--id", required=True, help="Prompt identifier (used as filename)")
    bake.add_argument("--title", required=True, help="Human-readable title")
    bake.add_argument("--prompt", help="Inline sys prompt text")
    bake.add_argument("--prompt-file", help="File containing sys prompt text")
    bake.add_argument("--author", default=os.getenv("USER", "unknown"))
    bake.add_argument(
        "--spec",
        help="Path/URL to OpenAPI spec (defaults to first available latest.json)",
    )
    bake.add_argument("--tags", help="Comma-separated list of tags")
    bake.add_argument(
        "--notes",
        help="Optional provenance notes stored alongside the prompt",
    )
    bake.add_argument(
        "--test",
        action="append",
        default=[],
        help="Add test description entries (can be repeated)",
    )
    bake.add_argument("--dry-run", action="store_true", help="Print YAML without writing")
    bake.add_argument("--force", action="store_true", help="Overwrite existing file")
    bake.add_argument(
        "--allow-missing-spec",
        action="store_true",
        help="Do not error if spec cannot be resolved",
    )
    bake.set_defaults(func=handle_bake)

    return parser


def main(argv: Optional[List[str]] = None) -> None:
    parser = build_parser()
    args = parser.parse_args(argv)
    args.func(args)


if __name__ == "__main__":
    main()
