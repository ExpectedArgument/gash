"""Generate assets/speed-light.svg and assets/speed-dark.svg (16:9, 1920x1080).

A real-time comparison of gash and Git Bash: each ball crosses its track once per run of
the workload, so one crossing takes exactly the measured wall-clock time. Inspired by
Ben Dicken's "language balls" visualization.

The same SVG is the source for the social video: assets/video/render.html loads it,
freezes the CSS animation and moves each ball (data-dur = seconds per crossing) frame by
frame, then encodes an MP4.

The gash lanes use the official icon, assets/gash-icon.svg. Git logo by Jason Long,
CC BY 3.0 (https://git-scm.com/downloads/logos).

Usage: python assets/make_speed_svg.py
"""

import xml.etree.ElementTree as ET
from pathlib import Path
from xml.sax.saxutils import quoteattr

# Wall-clock milliseconds on Windows 11, Git for Windows 2.56 (bash 5.3.15) vs gash.
TIMINGS = [
    # (title, detail, gash_ms, git_bash_ms)
    ("Start the shell", "bash -c true  × 100", 894, 19187),
    ("Claude Code Bash tool", "10 tool calls", 684, 15277),
    ("Run external tools", "sed -n 1p  × 50", 1494, 8943),
    ("A real script", "read loop, recursion, arrays", 260, 290087),
]

# Video length: one full back-and-forth of Git Bash's "Start the shell" ball, so that lane
# loops seamlessly. Stored on the SVG as data-loop for assets/video/render.html.
LOOP_SECONDS = 2 * TIMINGS[0][3] / 1000

THEMES = {
    "light": {
        "bg": "#ffffff", "text": "#1f2328", "muted": "#59636e", "faint": "#818b98", "track": "#e3e7eb",
        "rule": "#eef1f4", "gash": "#1fa45a", "gash_text": "#157f46", "git": "#f05133", "git_text": "#c4381c",
    },
    "dark": {
        "bg": "#0d1117", "text": "#e6edf3", "muted": "#9198a1", "faint": "#6e7681", "track": "#262c34",
        "rule": "#1c2128", "gash": "#1fa45a", "gash_text": "#3cc477", "git": "#f05133", "git_text": "#f47b5e",
    },
}
# "gash" green is the icon's own fill (#1FA45A); the text shades keep its hue at AA contrast.

SANS = "-apple-system, BlinkMacSystemFont, 'Segoe UI', 'Noto Sans', Helvetica, Arial, sans-serif"
MONO = "ui-monospace, SFMono-Regular, 'SF Mono', Menlo, Consolas, 'Liberation Mono', monospace"

W, H = 1920, 1080
PAD = 110
LOGO_X, LOGO = 620, 52
TRACK_X0, TRACK_X1 = 744, 1372      # ball centre travels between these
TIME_X = 1414
TOP = 268                           # first divider
BLOCK_H = 172
LANES = (50, 122)                   # lane centres within a block
BALL_R = 28

GIT_PATH = (
    "M90.156 41.965 50.036 1.848a5.918 5.918 0 0 0-8.372 0l-8.33 8.33 10.566 10.566a7.03 7.03 0 0 1 7.23 1.684 "
    "7.034 7.034 0 0 1 1.669 7.277l10.187 10.184a7.028 7.028 0 0 1 7.278 1.672 7.04 7.04 0 0 1 0 9.957 7.05 7.05 "
    "0 0 1-9.965 0 7.044 7.044 0 0 1-1.528-7.66l-9.5-9.497V59.36a7.04 7.04 0 0 1 1.86 11.29 7.04 7.04 0 0 1-9.957 "
    "0 7.04 7.04 0 0 1 0-9.958 7.06 7.06 0 0 1 2.304-1.539V33.926a7.049 7.049 0 0 1-3.82-9.234L29.242 14.272 "
    "1.73 41.777a5.925 5.925 0 0 0 0 8.371L41.852 90.27a5.925 5.925 0 0 0 8.37 0l39.934-39.934a5.925 5.925 0 0 0 "
    "0-8.371"
)


def fmt(ms: float) -> str:
    s = ms / 1000
    if s < 10:
        return f"{s:.2f} s"
    if s < 100:
        return f"{s:.1f} s"
    return f"{s:,.0f} s"


ICON = Path(__file__).with_name("gash-icon.svg")
ICON_BOX = (100, 924)               # the squircle's extent inside the icon's 1024-unit viewBox


def icon_symbol() -> str:
    """The official icon as a <symbol>, so each lane only needs a <use>."""
    ns = "{http://www.w3.org/2000/svg}"
    root = ET.parse(ICON).getroot()
    paths = "".join(
        "<path " + " ".join(f"{k}={quoteattr(v)}" for k, v in p.attrib.items()) + "/>"
        for p in root.iter(f"{ns}path")
    )
    return f'<symbol id="gash-icon" viewBox="{root.get("viewBox")}">{paths}</symbol>'


def gash_logo(x: float, cy: float) -> str:
    # Scale so the visible squircle (not the padded viewBox) is LOGO px wide, left edge at x.
    size = LOGO * 1024 / (ICON_BOX[1] - ICON_BOX[0])
    off = ICON_BOX[0] * size / 1024
    return (
        f'<use href="#gash-icon" x="{x - off:.2f}" y="{cy - size / 2:.2f}" '
        f'width="{size:.2f}" height="{size:.2f}"/>'
    )


def git_logo(x: float, cy: float, color: str) -> str:
    y = cy - LOGO / 2
    return f'<path transform="translate({x} {y}) scale({LOGO / 92})" fill="{color}" d="{GIT_PATH}"/>'


def build(t: dict) -> str:
    dist = TRACK_X1 - TRACK_X0
    o = [
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" viewBox="0 0 {W} {H}" role="img" '
        f'data-dist="{dist}" data-loop="{LOOP_SECONDS:.3f}" aria-label="gash vs Git Bash in real time: each ball crosses its track once per run">',
        "<style>",
        "  .ball { animation: run linear infinite alternate; }",
        f"  @keyframes run {{ from {{ transform: translateX(0); }} to {{ transform: translateX({dist}px); }} }}",
        "  @media (prefers-reduced-motion: reduce) { .ball { animation: none; } }",
        "</style>",
        f"<defs>{icon_symbol()}</defs>",
        f'<rect width="{W}" height="{H}" fill="{t["bg"]}"/>',
        f'<text x="{PAD}" y="160" font-family="{SANS}" font-size="72" font-weight="600" fill="{t["text"]}" '
        f'letter-spacing="-1">gash vs Git Bash</text>',
        f'<text x="{PAD}" y="214" font-family="{SANS}" font-size="30" fill="{t["muted"]}">'
        "Each ball crosses its track once per run, in real time.</text>",
        f'<text id="clock" x="{W - PAD}" y="160" font-family="{MONO}" font-size="40" fill="{t["faint"]}" '
        'text-anchor="end"></text>',
    ]
    y = TOP
    for title, detail, g, b in TIMINGS:
        o.append(f'<line x1="{PAD}" y1="{y}" x2="{W - PAD}" y2="{y}" stroke="{t["rule"]}" stroke-width="2"/>')
        mid = y + (LANES[0] + LANES[1]) / 2
        o.append(
            f'<text x="{PAD}" y="{mid - 6}" font-family="{SANS}" font-size="38" font-weight="600" '
            f'fill="{t["text"]}">{title}</text>'
        )
        o.append(
            f'<text x="{PAD}" y="{mid + 34}" font-family="{MONO}" font-size="26" fill="{t["faint"]}">{detail}</text>'
        )
        lanes = (
            (g, t["gash"], t["gash_text"], gash_logo(LOGO_X, y + LANES[0])),
            (b, t["git"], t["git_text"], git_logo(LOGO_X, y + LANES[1], t["git"])),
        )
        for (ms, ball, txt, logo), dy in zip(lanes, LANES):
            cy = y + dy
            dur = ms / 1000
            o.append(logo)
            o.append(
                f'<line x1="{TRACK_X0}" y1="{cy}" x2="{TRACK_X1}" y2="{cy}" stroke="{t["track"]}" '
                'stroke-width="4" stroke-linecap="round"/>'
            )
            o.append(
                f'<circle class="ball" data-dur="{dur:.3f}" style="animation-duration:{dur:.3f}s" '
                f'cx="{TRACK_X0}" cy="{cy}" r="{BALL_R}" fill="{ball}"/>'
            )
            o.append(
                f'<text x="{TIME_X}" y="{cy + 11}" font-family="{MONO}" font-size="31" fill="{txt}">{fmt(ms)}</text>'
            )
        o.append(
            f'<text x="{W - PAD}" y="{mid + 8}" font-family="{SANS}" font-size="62" font-weight="700" '
            f'fill="{t["gash_text"]}" text-anchor="end" letter-spacing="-1">{b / g:,.0f}×</text>'
        )
        o.append(
            f'<text x="{W - PAD}" y="{mid + 44}" font-family="{SANS}" font-size="26" fill="{t["faint"]}" '
            'text-anchor="end">faster</text>'
        )
        y += BLOCK_H
    o.append(f'<line x1="{PAD}" y1="{y}" x2="{W - PAD}" y2="{y}" stroke="{t["rule"]}" stroke-width="2"/>')
    o.append(
        f'<text x="{PAD}" y="{y + 66}" font-family="{SANS}" font-size="25" fill="{t["faint"]}">'
        "Wall-clock time on Windows 11 · Git for Windows 2.56 (bash 5.3.15) vs gash</text>"
    )
    o.append(
        f'<text x="{W - PAD}" y="{y + 66}" font-family="{MONO}" font-size="25" fill="{t["muted"]}" '
        'text-anchor="end">github.com/hex2f/gash</text>'
    )
    o.append("</svg>")
    return "\n".join(o) + "\n"


if __name__ == "__main__":
    here = Path(__file__).parent
    for name, theme in THEMES.items():
        p = here / f"speed-{name}.svg"
        p.write_text(build(theme), encoding="utf-8")
        print(f"wrote {p.name} ({p.stat().st_size} bytes)")
    print(f"video loop: {LOOP_SECONDS:.3f} s")
