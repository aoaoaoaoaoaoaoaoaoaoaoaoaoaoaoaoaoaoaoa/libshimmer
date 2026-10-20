#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.12"
# dependencies = ["markdown>=3.7", "pillow>=11"]
# ///
"""Render DESIGN.md as board.html: one dark page whose Mermaid views draw in the browser.

Every view is parsed first; parse errors name the view and line and fail the run, though
the board is still written. The page inlines the Mermaid bundle that codex-1337's
scripts/mermaid-render caches, so it opens offline. --snapshot also photographs the page.
"""

from __future__ import annotations

import argparse
import html
import json
import os
import re
import subprocess
import sys
import tempfile
import urllib.request
from pathlib import Path
from string import Template

import markdown
from PIL import Image, ImageChops

HERE = Path(__file__).resolve().parent
BUNDLE = Path.home() / ".cache/mermaid/mermaid.min.js"
SOURCE = "https://cdn.jsdelivr.net/npm/mermaid@11/dist/mermaid.min.js"
CANVAS = "#0d1117"
VIEW = re.compile(r"^```mermaid\n(.*?)^```$", re.DOTALL | re.MULTILINE)
CHROMIUM = [
    "chromium",
    "--headless",
    "--no-sandbox",
    "--disable-gpu",
    "--hide-scrollbars",
    "--virtual-time-budget=15000",
]

THEME = {
    "darkMode": True,
    "background": CANVAS,
    "fontFamily": "Adwaita Sans, Inter, DejaVu Sans, sans-serif",
    "fontSize": "15px",
    "primaryColor": "#1c2330",
    "primaryTextColor": "#e6edf3",
    "primaryBorderColor": "#6e7681",
    "secondaryColor": "#161b22",
    "tertiaryColor": "#11161d",
    "lineColor": "#8b949e",
    "textColor": "#c9d1d9",
    "edgeLabelBackground": CANVAS,
    "clusterBkg": "#11161d",
    "clusterBorder": "#30363d",
    "noteBkgColor": "#2b2410",
    "noteTextColor": "#f0e2b6",
    "noteBorderColor": "#9e7a1f",
    "actorBkg": "#1c2330",
    "actorBorder": "#58a6ff",
    "actorTextColor": "#e6edf3",
    "actorLineColor": "#484f58",
    "signalColor": "#c9d1d9",
    "signalTextColor": "#e6edf3",
    "labelBoxBkgColor": "#161b22",
    "labelBoxBorderColor": "#bc8cff",
    "labelTextColor": "#e6edf3",
    "loopTextColor": "#d2a8ff",
    "activationBkgColor": "#30363d",
    "activationBorderColor": "#8b949e",
    "sequenceNumberColor": CANVAS,
}

PAGE = Template("""<!doctype html>
<html lang="en">
<meta charset="utf-8">
<title>$title</title>
<style>
:root { color-scheme: dark; }
body { margin: 0; background: $canvas; color: #c9d1d9;
       font: 16px/1.55 "Adwaita Sans", Inter, "DejaVu Sans", sans-serif; }
main { max-width: 1180px; margin: 0 auto; padding: 40px 32px 96px; }
h1 { color: #f0f6fc; font-size: 30px; margin: 0 0 6px; }
h2 { color: #f0f6fc; font-size: 22px; margin: 56px 0 14px; padding-top: 18px;
     border-top: 1px solid #30363d; }
strong { color: #f0f6fc; }
code { font: 0.88em "JetBrains Mono", monospace; background: #161b22; padding: 1px 5px;
       border-radius: 4px; color: #e6edf3; }
pre { background: #161b22; border: 1px solid #30363d; border-radius: 8px; padding: 16px;
      overflow-x: auto; }
pre code { padding: 0; background: none; font-size: 13.5px; line-height: 1.5; }
pre.mermaid { background: $canvas; border-color: #21262d; text-align: center; padding: 24px 8px; }
table { border-collapse: collapse; width: 100%; margin: 14px 0; font-size: 14.5px; }
th, td { border: 1px solid #30363d; padding: 6px 10px; text-align: left; vertical-align: top; }
th { background: #161b22; color: #f0f6fc; }
tr:nth-child(even) td { background: #10151c; }
a { color: #58a6ff; }
</style>
<main>
$body
</main>
<script>$bundle</script>
<script>
mermaid.initialize({ startOnLoad: true, theme: "base", securityLevel: "loose",
  themeVariables: $theme,
  flowchart: { htmlLabels: true, curve: "basis", padding: 14 },
  sequence: { mirrorActors: false } });
</script>
</html>
""")

CHECK = Template("""<!doctype html>
<meta charset="utf-8">
<script>$bundle</script>
<pre id="verdict"></pre>
<script>
(async () => {
  mermaid.initialize({ startOnLoad: false });
  const faults = [];
  for (const [index, view] of $views.entries()) {
    try { await mermaid.parse(view); }
    catch (error) { faults.push(`view $${index + 1}: $${String(error.message || error)}`); }
  }
  document.getElementById("verdict").textContent = faults.join("\\n\\n") || "ok";
})();
</script>
""")


def bundle() -> str:
    if not BUNDLE.exists():
        BUNDLE.parent.mkdir(parents=True, exist_ok=True)
        with urllib.request.urlopen(SOURCE, timeout=120) as response:
            BUNDLE.write_bytes(response.read())
    return BUNDLE.read_text()


def chromium(page: Path, *args: str) -> str:
    """Run headless Chromium on `page` with a throwaway profile and no display."""
    env = {
        key: value
        for key, value in os.environ.items()
        if key not in {"DISPLAY", "WAYLAND_DISPLAY"}
    }
    with tempfile.TemporaryDirectory() as profile:
        return subprocess.run(
            [*CHROMIUM, f"--user-data-dir={profile}", *args, page.as_uri()],
            check=True,
            capture_output=True,
            env=env,
            text=True,
        ).stdout


def faults(views: list[str]) -> list[str]:
    """Mermaid's parse error for each view that has one."""
    with tempfile.TemporaryDirectory() as scratch:
        page = Path(scratch) / "check.html"
        page.write_text(CHECK.substitute(bundle=bundle(), views=json.dumps(views)))
        dom = chromium(page, "--dump-dom")
    verdict = re.search(r'<pre id="verdict">(.*?)</pre>', dom, re.DOTALL)
    found = html.unescape(verdict.group(1)) if verdict else "no verdict from Chromium"
    return [] if found == "ok" else found.split("\n\n")


def render(dossier: str, views: list[str]) -> str:
    def hold(match: re.Match[str]) -> str:
        views.append(match.group(1))
        return f"\n\nVIEW{len(views) - 1}\n\n"

    body = markdown.markdown(
        VIEW.sub(hold, dossier), extensions=["tables", "fenced_code", "sane_lists"]
    )
    for index, source in enumerate(views):
        body = body.replace(
            f"<p>VIEW{index}</p>", f'<pre class="mermaid">{html.escape(source)}</pre>'
        )
    return PAGE.substitute(
        title=html.escape(dossier.splitlines()[0].lstrip("# ")),
        canvas=CANVAS,
        body=body,
        bundle=bundle(),
        theme=json.dumps(THEME),
    )


def snapshot(page: Path, out: Path, width: int) -> None:
    """Photograph the page down to its last drawn row."""
    chromium(page, f"--window-size={width},16000", f"--screenshot={out}")
    image = Image.open(out).convert("RGB")
    drawn = ImageChops.difference(image, Image.new("RGB", image.size, CANVAS)).getbbox()
    if drawn is not None:
        image.crop((0, 0, image.width, min(image.height, drawn[3] + 40))).save(out)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--snapshot", type=Path, help="also photograph the page to this PNG"
    )
    parser.add_argument("--width", type=int, default=1400)
    args = parser.parse_args()
    views: list[str] = []
    page = HERE / "board.html"
    page.write_text(render((HERE / "DESIGN.md").read_text(), views))
    print(page)
    broken = faults(views)
    for fault in broken:
        print(fault, file=sys.stderr)
    if args.snapshot:
        snapshot(page, args.snapshot.resolve(), args.width)
        print(args.snapshot)
    return 1 if broken else 0


if __name__ == "__main__":
    raise SystemExit(main())
