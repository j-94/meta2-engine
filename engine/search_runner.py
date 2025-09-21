#!/usr/bin/env python3
"""Search utilities for Meta² engine CLI.

Attempts to query the orchestrator `/search` endpoint; if unavailable, falls
back to a simple local text scan so operators still get signal (and CI can
exercise the command without network access).
"""

from __future__ import annotations

import argparse
import json
import os
import sys
import urllib.error
import urllib.parse
import urllib.request
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, Iterable, List, Optional


DEFAULT_BASE_URL = "http://127.0.0.1:8080"
ROOT = Path(__file__).resolve().parent
FALLBACK_SCAN_ROOTS = [ROOT, ROOT.parent, ROOT.parent / "Meta-orchestrator-API---Snapshots"]


@dataclass
class SearchArgs:
    query: str
    limit: int
    base_url: str
    api_key: Optional[str]
    timeout: float
    out_path: Optional[Path]
    fallback: bool


def call_remote(args: SearchArgs) -> List[Dict[str, object]]:
    params = urllib.parse.urlencode({"q": args.query, "limit": args.limit})
    url = f"{args.base_url.rstrip('/')}/search?{params}"
    req = urllib.request.Request(url)
    if args.api_key:
        req.add_header("X-API-Key", args.api_key)
    try:
        with urllib.request.urlopen(req, timeout=args.timeout) as response:
            body = response.read()
            try:
                data = json.loads(body)
            except json.JSONDecodeError:
                raise RuntimeError("Search API returned non-JSON payload") from None
    except urllib.error.HTTPError as exc:  # pragma: no cover - networking
        if exc.code == 404:
            raise FileNotFoundError("/search endpoint not found on orchestrator") from exc
        raise RuntimeError(f"Search API HTTP error: {exc.code}") from exc
    except urllib.error.URLError as exc:  # pragma: no cover - networking
        raise ConnectionError(f"Failed to reach orchestrator search endpoint: {exc}") from exc

    if isinstance(data, dict) and "results" in data:
        items = data["results"]
    else:
        items = data
    if not isinstance(items, list):
        raise RuntimeError("Unexpected search response format")
    return items[: args.limit]


def local_scan(args: SearchArgs) -> List[Dict[str, object]]:
    roots = [root for root in FALLBACK_SCAN_ROOTS if root.exists()]
    matches: List[Dict[str, object]] = []
    query_lower = args.query.lower()
    for root in roots:
        for path in root.rglob("*.md"):
            try:
                text = path.read_text(encoding="utf-8")
            except Exception:
                continue
            if query_lower in text.lower():
                snippet = extract_snippet(text, query_lower)
                matches.append(
                    {
                        "path": str(path.relative_to(ROOT.parent)),
                        "snippet": snippet,
                    }
                )
                if len(matches) >= args.limit:
                    return matches
    return matches


def extract_snippet(text: str, needle: str, radius: int = 120) -> str:
    lower = text.lower()
    idx = lower.find(needle)
    if idx == -1:
        return text[: radius] + ("…" if len(text) > radius else "")
    start = max(0, idx - radius)
    end = min(len(text), idx + len(needle) + radius)
    snippet = text[start:end].replace("\n", " ")
    if start > 0:
        snippet = "…" + snippet
    if end < len(text):
        snippet = snippet + "…"
    return snippet


def write_jsonl(path: Path, rows: Iterable[Dict[str, object]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as fh:
        for row in rows:
            fh.write(json.dumps(row, ensure_ascii=False))
            fh.write("\n")


def main(argv: Optional[List[str]] = None) -> None:
    parser = argparse.ArgumentParser(description="Meta² search runner")
    parser.add_argument("--query", required=True, help="Search query string")
    parser.add_argument("--limit", type=int, default=5, help="Maximum results (default 5)")
    parser.add_argument("--base-url", default=DEFAULT_BASE_URL, help="Orchestrator base URL")
    parser.add_argument("--api-key", help="Optional API key header value")
    parser.add_argument("--timeout", type=float, default=8.0, help="HTTP timeout seconds")
    parser.add_argument(
        "--out",
        help="Write results to JSONL file (in addition to stdout)",
    )
    parser.add_argument(
        "--no-fallback",
        action="store_true",
        help="Fail instead of scanning local files when remote search is unavailable",
    )

    args_ns = parser.parse_args(argv)
    args = SearchArgs(
        query=args_ns.query,
        limit=max(1, args_ns.limit),
        base_url=args_ns.base_url,
        api_key=args_ns.api_key or os.getenv("ORCH_API_KEY"),
        timeout=args_ns.timeout,
        out_path=Path(args_ns.out).resolve() if args_ns.out else None,
        fallback=not args_ns.no_fallback,
    )

    try:
        results = call_remote(args)
        source = "remote"
    except (FileNotFoundError, ConnectionError, RuntimeError) as exc:
        if not args.fallback:
            raise
        print(f"[meta2-search] Remote search unavailable ({exc}); using local fallback.", file=sys.stderr)
        results = local_scan(args)
        source = "local"

    payload = {
        "query": args.query,
        "limit": args.limit,
        "source": source,
        "count": len(results),
        "results": results,
    }

    print(json.dumps(payload, ensure_ascii=False, indent=2))
    if args.out_path:
        json_rows = [{"query": args.query, **row, "source": source} for row in results]
        write_jsonl(args.out_path, json_rows)
        print(f"[meta2-search] wrote {len(results)} rows to {args.out_path}")


if __name__ == "__main__":
    main()
