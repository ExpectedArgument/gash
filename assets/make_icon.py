"""Rasterize assets/gash-icon.svg into assets/gash.ico, the icon build.rs embeds in gash.exe.

Zero dependencies: the icon is two filled polygons (paths made only of M/L/Z), so this
scan-converts them directly, with 16 sub-scanlines per pixel row and exact horizontal
coverage, then composites them in order. (The bolt's 2-unit stroke is ignored: it is a
quarter of a pixel even at 256 px.) Sizes below 256 are stored as 32-bit BMPs, which is
what Windows expects at small sizes; 256 is stored as PNG.

Usage: python assets/make_icon.py
"""

import re
import struct
import xml.etree.ElementTree as ET
import zlib
from pathlib import Path

HERE = Path(__file__).parent
SRC = HERE / "gash-icon.svg"
OUT = HERE / "gash.ico"
SIZES = (16, 20, 24, 32, 40, 48, 64, 256)
SUB = 16  # sub-scanlines per pixel row


def load_shapes():
    ns = "{http://www.w3.org/2000/svg}"
    root = ET.parse(SRC).getroot()
    x, y, w, h = (float(n) for n in root.get("viewBox").split())
    if (x, y) != (0, 0) or w != h:
        raise SystemExit("expected a square viewBox at 0 0")
    shapes = []
    for p in root.iter(f"{ns}path"):
        d = p.get("d")
        if not re.fullmatch(r"\s*M[\d.\s-]+(?:L[\d.\s-]+)*Z\s*", d):
            raise SystemExit(f"unsupported path data (only M/L/Z): {d[:40]}...")
        nums = [float(n) for n in re.findall(r"-?\d+(?:\.\d+)?", d)]
        fill = p.get("fill")
        shapes.append((list(zip(nums[0::2], nums[1::2])), tuple(int(fill[i:i + 2], 16) for i in (1, 3, 5))))
    return w, shapes


def coverage(pts, scale, n):
    """Fraction of each pixel of an n x n grid covered by the polygon (non-zero rule)."""
    edges = []
    for (x0, y0), (x1, y1) in zip(pts, pts[1:] + pts[:1]):
        if y0 != y1:
            edges.append((x0 * scale, y0 * scale, x1 * scale, y1 * scale, 1 if y1 > y0 else -1))
    cov = [0.0] * (n * n)
    wt = 1 / SUB
    for row in range(n):
        base = row * n
        for s in range(SUB):
            y = row + (s + 0.5) / SUB
            xs = sorted(
                (x0 + (y - y0) * (x1 - x0) / (y1 - y0), w)
                for x0, y0, x1, y1, w in edges
                if y0 <= y < y1 or y1 <= y < y0
            )
            wind = 0
            start = 0.0
            for x, w in xs:
                before = wind
                wind += w
                if before == 0 and wind != 0:
                    start = x
                elif before != 0 and wind == 0:
                    a, b = max(start, 0.0), min(x, float(n))
                    if b <= a:
                        continue
                    ia, ib = int(a), int(b)
                    if ia == ib:
                        cov[base + ia] += (b - a) * wt
                        continue
                    cov[base + ia] += (ia + 1 - a) * wt
                    for i in range(ia + 1, ib):
                        cov[base + i] += wt
                    if ib < n:
                        cov[base + ib] += (b - ib) * wt
    return cov


def render(n, units, shapes):
    """Straight-alpha RGBA rows, top to bottom."""
    scale = n / units
    r = [0.0] * (n * n)
    g = [0.0] * (n * n)
    b = [0.0] * (n * n)
    a = [0.0] * (n * n)
    for pts, (cr, cg, cb) in shapes:  # premultiplied "over", in document order
        for i, c in enumerate(coverage(pts, scale, n)):
            if c > 0:
                c = min(c, 1.0)
                k = 1 - c
                r[i] = cr * c + r[i] * k
                g[i] = cg * c + g[i] * k
                b[i] = cb * c + b[i] * k
                a[i] = c + a[i] * k
    rows = []
    for y in range(n):
        row = bytearray()
        for i in range(y * n, (y + 1) * n):
            if a[i] <= 0:
                row += b"\0\0\0\0"
            else:
                row += bytes((round(r[i] / a[i]), round(g[i] / a[i]), round(b[i] / a[i]), round(a[i] * 255)))
        rows.append(bytes(row))
    return rows


def png(n, rows):
    def chunk(tag, data):
        return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", zlib.crc32(tag + data))

    raw = b"".join(b"\0" + row for row in rows)
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", struct.pack(">IIBBBBB", n, n, 8, 6, 0, 0, 0))
        + chunk(b"IDAT", zlib.compress(raw, 9))
        + chunk(b"IEND", b"")
    )


def bmp(n, rows):
    """ICO-style DIB: BITMAPINFOHEADER (double height), bottom-up BGRA, then the AND mask."""
    stride = (n + 31) // 32 * 4
    xor = bytearray()
    mask = bytearray()
    for row in reversed(rows):
        bits = bytearray(stride)
        for x in range(n):
            pr, pg, pb, pa = row[4 * x:4 * x + 4]
            xor += bytes((pb, pg, pr, pa))
            if pa == 0:
                bits[x // 8] |= 0x80 >> (x % 8)
        mask += bits
    header = struct.pack("<IiiHHIIiiII", 40, n, 2 * n, 1, 32, 0, len(xor) + len(mask), 0, 0, 0, 0)
    return header + bytes(xor) + bytes(mask)


def main():
    units, shapes = load_shapes()
    images = []
    for n in SIZES:
        rows = render(n, units, shapes)
        images.append((n, png(n, rows) if n >= 256 else bmp(n, rows)))
    out = bytearray(struct.pack("<HHH", 0, 1, len(images)))
    offset = 6 + 16 * len(images)
    for n, data in images:
        dim = 0 if n >= 256 else n
        out += struct.pack("<BBBBHHII", dim, dim, 0, 0, 1, 32, len(data), offset)
        offset += len(data)
    for _, data in images:
        out += data
    OUT.write_bytes(bytes(out))
    print(f"wrote {OUT.name} ({len(out)} bytes): {', '.join(map(str, SIZES))} px")


if __name__ == "__main__":
    main()
