#!/usr/bin/env python3
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


SECTION_ORDER = [
    ("breaking", "⚠️ 重大变更"),
    ("feat", "✨ 新增功能"),
    ("fix", "🐞 问题修复"),
    ("perf", "⚡ 性能优化"),
    ("ux", "🎨 体验改进"),
    ("docs", "📚 文档与流程"),
    ("maintenance", "🧰 内部维护"),
]

TYPE_MAP = {
    "feat": "feat",
    "fix": "fix",
    "perf": "perf",
    "ui": "ux",
    "ux": "ux",
    "docs": "docs",
    "doc": "docs",
    "build": "maintenance",
    "ci": "maintenance",
    "chore": "maintenance",
    "refactor": "maintenance",
    "style": "maintenance",
    "test": "maintenance",
}

RELEASE_COMMIT_RE = re.compile(r"^chore(?:\([^)]*\))?:\s*release\s+v?\d", re.I)
CONVENTIONAL_RE = re.compile(r"^(?P<type>[a-zA-Z]+)(?:\([^)]+\))?(?P<breaking>!)?:\s*(?P<subject>.+)$")


def run_git(repo: Path, args: list[str], check: bool = True) -> str:
    result = subprocess.run(
        ["git", *args],
        cwd=repo,
        text=True,
        encoding="utf-8",
        errors="replace",
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if check and result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or f"git {' '.join(args)} failed")
    return result.stdout.strip()


def discover_previous_tag(repo: Path, to_ref: str) -> str | None:
    candidates = []
    if to_ref.startswith("v"):
        candidates.append(f"{to_ref}^")
    candidates.append(to_ref)

    for ref in candidates:
        out = run_git(repo, ["describe", "--tags", "--abbrev=0", ref], check=False)
        if out:
            if out != to_ref:
                return out
            parent_out = run_git(repo, ["describe", "--tags", "--abbrev=0", f"{to_ref}^"], check=False)
            if parent_out:
                return parent_out
    return None


def infer_next_version(from_tag: str | None) -> str:
    if not from_tag:
        return "0.1.0"

    raw = from_tag[1:] if from_tag.startswith("v") else from_tag
    match = re.match(r"^(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?P<suffix>.*)$", raw)
    if not match or match.group("suffix"):
        raise RuntimeError(f"Cannot infer next patch version from tag: {from_tag}")

    major = int(match.group("major"))
    minor = int(match.group("minor"))
    patch = int(match.group("patch")) + 1
    return f"{major}.{minor}.{patch}"


def parse_commit(line: str) -> dict:
    sha, subject, author, date = line.split("\x1f", 3)
    match = CONVENTIONAL_RE.match(subject)
    category = None
    clean_subject = subject
    breaking = False

    if match:
        commit_type = match.group("type").lower()
        category = TYPE_MAP.get(commit_type)
        clean_subject = match.group("subject").strip()
        breaking = bool(match.group("breaking"))
        if breaking:
            category = "breaking"

    return {
        "sha": sha,
        "short_sha": sha[:7],
        "subject": clean_subject,
        "raw_subject": subject,
        "author": author,
        "date": date,
        "category": category,
        "breaking": breaking,
    }


def collect_commits(repo: Path, from_tag: str | None, to_ref: str) -> list[dict]:
    rev_range = f"{from_tag}..{to_ref}" if from_tag else to_ref
    fmt = "%H%x1f%s%x1f%an%x1f%ad"
    out = run_git(repo, ["log", "--no-merges", "--date=short", f"--pretty=format:{fmt}", rev_range], check=False)
    commits = []
    for line in out.splitlines():
        if not line.strip():
            continue
        commit = parse_commit(line)
        if RELEASE_COMMIT_RE.match(commit["raw_subject"]):
            continue
        if not commit["category"]:
            continue
        commits.append(commit)
    return commits


def render_markdown(version: str, from_tag: str | None, to_ref: str, commits: list[dict]) -> str:
    title = f"## v{version}" if not version.startswith("v") else f"## {version}"
    lines = [title, ""]
    if from_tag:
        lines.extend([f"范围：`{from_tag}..{to_ref}`", ""])
    else:
        lines.extend([f"范围：`{to_ref}`", ""])

    if not commits:
        lines.append("- 本次版本没有检测到用户可见的提交。")
        return "\n".join(lines)

    grouped = {key: [] for key, _ in SECTION_ORDER}
    for commit in commits:
        grouped.setdefault(commit["category"], []).append(commit)

    for key, heading in SECTION_ORDER:
        items = grouped.get(key, [])
        if not items:
            continue
        lines.append(f"### {heading}")
        for commit in items:
            lines.append(f"- {commit['subject']} (`{commit['short_sha']}`)")
        lines.append("")

    return "\n".join(lines).rstrip()


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate release changelog from Git tags and commits.")
    parser.add_argument("--repo", default=".", help="Repository path. Defaults to current directory.")
    parser.add_argument("--version", help="Target version, with or without v prefix. Defaults to previous tag patch +1.")
    parser.add_argument("--from-tag", help="Previous release tag. Auto-discovered when omitted.")
    parser.add_argument("--to-ref", default="HEAD", help="Target ref or tag. Defaults to HEAD.")
    parser.add_argument("--json", action="store_true", help="Print structured JSON instead of Markdown.")
    args = parser.parse_args()

    repo = Path(args.repo).resolve()
    if not (repo / ".git").exists():
        print(f"Not a Git repository: {repo}", file=sys.stderr)
        return 2

    from_tag = args.from_tag or discover_previous_tag(repo, args.to_ref)
    version = args.version or infer_next_version(from_tag)
    commits = collect_commits(repo, from_tag, args.to_ref)

    markdown = render_markdown(version, from_tag, args.to_ref, commits)
    if args.json:
        print(json.dumps({
            "version": version,
            "from_tag": from_tag,
            "to_ref": args.to_ref,
            "commits": commits,
            "markdown": markdown,
        }, ensure_ascii=False, indent=2))
    else:
        print(markdown)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
