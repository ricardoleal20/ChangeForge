#!/usr/bin/env python3
"""
Static docs builder and dev server for ChangeForge.

Usage:
  python docs/docs_build.py build     # Build site into ./docs
  python docs/docs_build.py serve     # Build and serve ./docs at http://localhost:8000
  python docs/docs_build.py serve -p 5173
"""

from __future__ import annotations

import http.server
import os
import re
import shutil
import socketserver
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Tuple

REPO_ROOT = Path(__file__).resolve().parents[1]
SRC_DIR = REPO_ROOT / "docs" / "webpage"
TEMPLATES_DIR = SRC_DIR / "templates"
CONTENT_DIR = SRC_DIR / "content"
DEST_DIR = REPO_ROOT / "docs"
IMG_DIR = REPO_ROOT / "docs" / "img"


def read_version_from_pyproject(pyproject_path: Path) -> str:
    """
    Read version="x.y.z" from pyproject.toml without third-party deps.
    Returns a normalized string like "v1.2.3" (with leading 'v').
    """
    if not pyproject_path.exists():
        return "v0.0.0"
    content = pyproject_path.read_text(encoding="utf-8")
    # Try common patterns under [project] or tool sections
    m = re.search(r'(?mi)^\s*version\s*=\s*"(.*?)"\s*$', content)
    if not m:
        return "v0.0.0"
    ver = m.group(1).strip()
    if not ver.startswith("v"):
        ver = f"v{ver}"
    return ver


def ensure_dirs():
    (DEST_DIR / "commands").mkdir(parents=True, exist_ok=True)
    (DEST_DIR / "img").mkdir(parents=True, exist_ok=True)


def copy_images():
    # Images are already under docs/img → ensure they remain available
    # If user adds new images, just keep the folder as-is.
    # We copy only if missing, to avoid unnecessary churn.
    src_img = REPO_ROOT / "docs" / "img"
    if src_img.exists():
        # Mirror copy while preserving existing files
        for p in src_img.rglob("*"):
            if p.is_file():
                rel = p.relative_to(src_img)
                dest = IMG_DIR / rel
                dest.parent.mkdir(parents=True, exist_ok=True)
                if not dest.exists():
                    shutil.copy2(p, dest)


@dataclass
class TransformContext:
    version: str
    is_commands_page: bool
    is_root_page: bool


def transform_html(
    html: str, ctx: TransformContext, base_url: str | None = None
) -> str:
    """Apply small transforms: set version badge, fix asset paths."""
    # 1) Replace version placeholders
    html = html.replace("{{VERSION}}", ctx.version)
    html = html.replace("{{REL}}", ".." if ctx.is_commands_page else ".")
    # Also support replacing existing badge text
    # <span class="badge">v0.1.4</span> → <span class="badge">vX.Y.Z</span>
    html = re.sub(
        r'(<span\s+class="badge"\s*>\s*)v?[0-9]+\.[0-9]+\.[0-9]+(\s*</span>)',
        rf"\1{ctx.version}\2",
        html,
        flags=re.IGNORECASE,
    )

    # 2) Fix logo/image paths relative to destination
    # Root pages end up at docs/*.html → images live at docs/img/*
    # Command pages end up at docs/commands/*.html → images live at ../img/*
    if ctx.is_root_page:
        # from ../img/* → ./img/*
        html = html.replace('src="../img/', 'src="./img/')
        # sometimes authors may use ../../img in root src, normalize too
        html = html.replace('src="../../img/', 'src="./img/')
    elif ctx.is_commands_page:
        # from ../../img/* → ../img/*
        html = html.replace('src="../../img/', 'src="../img/')

    # 3) If a base_url (from CNAME) is provided, convert asset paths to absolute URLs
    if base_url:
        # Normalize trailing slash
        base = base_url.rstrip("/")
        # Stylesheet link
        html = re.sub(
            r'href="(?:\./|\.\./)styles\.css"',
            f'href="{base}/styles.css"',
            html,
            flags=re.IGNORECASE,
        )
        # Image sources (logo, screenshots, etc.)
        html = re.sub(
            r'src="(?:\./|\.\./)img/([^"]+)"',
            rf'src="{base}/img/\1"',
            html,
            flags=re.IGNORECASE,
        )

    return html


def markdown_to_html(md: str) -> str:
    """
    Minimal markdown to HTML converter (headings, paragraphs, lists, code blocks, inline code).
    Not exhaustive, just enough for our docs.
    """
    lines = md.splitlines()
    html_lines = []
    in_code_block = False
    in_list = False
    code_buffer: list[str] = []

    def flush_list():
        nonlocal in_list
        if in_list:
            html_lines.append("</ul>")
            in_list = False

    def flush_code():
        nonlocal in_code_block, code_buffer
        if in_code_block:
            code_html = "\n".join(code_buffer)
            html_lines.append(
                '<div class="card code"><pre><code>' + code_html + "</code></pre></div>"
            )
            code_buffer = []
            in_code_block = False

    for raw in lines:
        line = raw.rstrip("\n")
        if line.strip().startswith("```"):
            if in_code_block:
                flush_code()
            else:
                flush_list()
                in_code_block = True
                code_buffer = []
            continue
        if in_code_block:
            code_buffer.append(line)
            continue
        if line.startswith("### "):
            flush_list()
            html_lines.append(f"<h3>{line[4:].strip()}</h3>")
            continue
        if line.startswith("## "):
            flush_list()
            html_lines.append(f"<h2>{line[3:].strip()}</h2>")
            continue
        if line.startswith("# "):
            flush_list()
            html_lines.append(f"<h1>{line[2:].strip()}</h1>")
            continue
        if line.startswith("- "):
            if not in_list:
                in_list = True
                html_lines.append("<ul>")
            html_lines.append(f"<li>{line[2:].strip()}</li>")
            continue
        if not line.strip():
            flush_list()
            html_lines.append("")
            continue
        # inline code
        text = re.sub(r"`([^`]+)`", r"<code>\1</code>", line)
        html_lines.append(f"<p>{text}</p>")

    flush_code()
    flush_list()
    # Join paragraphs; remove duplicate blank lines within HTML
    return "\n".join(html_lines).replace("\n\n\n", "\n\n")


def render_with_base(
    content_html: str,
    page_title: str,
    is_commands_page: bool,
    version: str,
    base_url: str | None,
) -> str:
    base = (TEMPLATES_DIR / "base.html").read_text(encoding="utf-8")
    rel = ".." if is_commands_page else "."
    out = (
        base.replace("{{TITLE}}", page_title)
        .replace("{{CONTENT}}", content_html)
        .replace("{{REL}}", rel)
        .replace("{{VERSION}}", version)
    )
    # Final pass through transform to normalize any remaining bits
    ctx = TransformContext(
        version=version,
        is_commands_page=is_commands_page,
        is_root_page=not is_commands_page,
    )
    return transform_html(out, ctx, base_url)


def read_cname_base_url(cname_path: Path) -> str | None:
    """Return https://<domain> if CNAME exists and has a hostname."""
    if not cname_path.exists():
        return None
    domain = cname_path.read_text(encoding="utf-8").strip().splitlines()[0].strip()
    if not domain:
        return None
    if domain.startswith("http://") or domain.startswith("https://"):
        return domain
    return f"https://{domain}"


def build() -> Tuple[int, int]:
    """Build pages from docs/webpage → docs with path adjustments."""
    ensure_dirs()
    copy_images()
    version = read_version_from_pyproject(REPO_ROOT / "pyproject.toml")
    base_url = read_cname_base_url(DEST_DIR / "CNAME")

    copied = 0
    transformed = 0

    # 0) Render content-driven pages (Markdown → HTML via template)
    content_outputs: list[Tuple[Path, Path]] = []
    if CONTENT_DIR.exists():
        for src in CONTENT_DIR.rglob("*.md"):
            rel = src.relative_to(CONTENT_DIR)
            dest = DEST_DIR / rel.with_suffix(".html")
            # Detect if under commands
            is_commands = "commands" in rel.parts
            md = src.read_text(encoding="utf-8")
            html_body = markdown_to_html(md)
            # Use the first heading as page title if present
            mtitle = re.search(r"^#\s+(.+)$", md, flags=re.MULTILINE)
            title = mtitle.group(1).strip() if mtitle else "ChangeForge Docs"
            out = render_with_base(html_body, title, is_commands, version, base_url)
            dest.parent.mkdir(parents=True, exist_ok=True)
            dest.write_text(out, encoding="utf-8")
            transformed += 1
            content_outputs.append((src, dest))

    # 1) Map static source → destination (skip those already generated)
    mappings = [
        (SRC_DIR / "index.html", DEST_DIR / "index.html"),
        (SRC_DIR / "styles.css", DEST_DIR / "styles.css"),
        (SRC_DIR / "getting-started.html", DEST_DIR / "getting-started.html"),
        (SRC_DIR / "configuration.html", DEST_DIR / "configuration.html"),
        (SRC_DIR / "commands" / "init.html", DEST_DIR / "commands" / "init.html"),
        (SRC_DIR / "commands" / "create.html", DEST_DIR / "commands" / "create.html"),
        (SRC_DIR / "commands" / "list.html", DEST_DIR / "commands" / "list.html"),
        (SRC_DIR / "commands" / "bump.html", DEST_DIR / "commands" / "bump.html"),
    ]

    for src, dest in mappings:
        if not src.exists():
            continue
        # skip if this file was already generated from content
        if dest.exists() and any(d == dest for _, d in content_outputs):
            continue
        raw = src.read_text(encoding="utf-8")
        ctx = TransformContext(
            version=version,
            is_commands_page=("commands" in src.parts),
            is_root_page=("commands" not in src.parts),
        )
        out = transform_html(raw, ctx, base_url)
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_text(out, encoding="utf-8")
        copied += 1
        transformed += 1

    return copied, transformed


def serve(port: int = 8000):
    copied, transformed = build()
    os.chdir(DEST_DIR)
    handler = http.server.SimpleHTTPRequestHandler
    with socketserver.TCPServer(("", port), handler) as httpd:
        print(f"[docs] Built {copied} files (transformed {transformed}).")
        print(f"[docs] Serving docs directory at http://localhost:{port}")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n[docs] Server stopped.")


def main(argv: list[str]) -> int:
    if len(argv) < 2 or argv[1] in {"-h", "--help", "help"}:
        print(__doc__)
        return 0
    cmd = argv[1]
    if cmd == "build":
        copied, transformed = build()
        print(
            f"[docs] Built {copied} files (transformed {transformed}). Output: {DEST_DIR}"
        )
        return 0
    if cmd == "serve":
        port = 8000
        # parse optional -p/--port
        if "-p" in argv:
            try:
                port = int(argv[argv.index("-p") + 1])
            except Exception:
                pass
        if "--port" in argv:
            try:
                port = int(argv[argv.index("--port") + 1])
            except Exception:
                pass
        serve(port)
        return 0
    print(f"Unknown command: {cmd}\n")
    print(__doc__)
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
