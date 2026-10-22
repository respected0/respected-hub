"""Builds the Respected brand files: SVG (full and small), 1024 px PNG and a multi-size ICO for Hub, Brain and Orchestrator.

The geometry below is the single source of the three marks. Rendering uses Edge in headless mode (transparent background) and a
small area-averaging resampler, so nothing needs installing. Run from anywhere:  python dev/brand/make_brand.py
Output: assets/brand/<name>.svg, <name>-small.svg, <name>-1024.png, <name>.ico  (the app and the other two repos copy from there).
"""
from __future__ import annotations

import os
import struct
import subprocess
import sys
import tempfile
import zlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "assets" / "brand"
NAVY, LINE, WHITE, GREY = "#11161d", "#243243", "#f2f4f8", "#8f98a0"
VIOLET, VIOLET_DARK, BLUE, BLUE_DARK = "#9d7dff", "#6c52c4", "#4c9aff", "#2d6bb8"

TILE = f'<rect x="1.5" y="1.5" width="253" height="253" rx="55" fill="{NAVY}" stroke="{LINE}" stroke-width="3"/>'


def letter(main: str, leg: str, inside: str = "") -> str:
    """A bold R in two flat tones whose counter is a doorway; `inside` is drawn in the doorway."""
    return (f'<path fill="{leg}" d="M122 166 H176 L222 214 H168 Z"/>'
            f'<path fill="{main}" fill-rule="evenodd" d="M62 40 H150 A66 66 0 0 1 150 172 H108 V214 H62 Z M108 140 V102 A29 29 0 0 1 166 102 V140 Z"/>'
            f'{inside}')


def neurons() -> str:
    cx, cy = 137, 114
    ends = [(121, 96), (153, 96), (137, 90), (122, 128), (152, 128)]
    rays = "".join(f'<line x1="{cx}" y1="{cy}" x2="{x}" y2="{y}" stroke="{WHITE}" stroke-width="4.5" stroke-linecap="round"/>'
                   f'<circle cx="{x}" cy="{y}" r="4.6" fill="{WHITE}"/>' for x, y in ends)
    return rays + f'<circle cx="{cx}" cy="{cy}" r="8.5" fill="{WHITE}"/>'


FORK = (f'<g fill="none" stroke="{WHITE}" stroke-width="5" stroke-linecap="round" stroke-linejoin="round">'
        '<path d="M137 138 V126 C137 116 120 116 120 106 V100 C120 90 137 92 137 82 M137 126 C137 116 154 116 154 106 V100 C154 90 137 92 137 82 M137 82 V76"/></g>')

# At 32 px and below the fine detail turns to noise: the doorway carries one bold shape instead.
MARKS = {
    "hub": (letter(WHITE, GREY), letter(WHITE, GREY)),
    "brain": (letter(VIOLET, VIOLET_DARK, neurons()), letter(VIOLET, VIOLET_DARK, f'<circle cx="137" cy="118" r="15" fill="{WHITE}"/>')),
    "orchestrator": (letter(BLUE, BLUE_DARK, FORK), letter(BLUE, BLUE_DARK, f'<rect x="119" y="96" width="12" height="40" rx="6" fill="{WHITE}"/><rect x="143" y="96" width="12" height="40" rx="6" fill="{WHITE}"/>')),
}


TILE_PLAIN = f'<rect width="256" height="256" rx="56" fill="{NAVY}"/>'   # small sizes: no border line (it would be a 1 px smudge)


def svg(inner: str, small: bool = False) -> str:
    return f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 256 256" width="256" height="256">{TILE_PLAIN if small else TILE}{inner}</svg>\n'


# ---------------------------------------------------------------- PNG (de)coding without libraries
def read_png(path: Path):
    data = path.read_bytes()
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    pos, idat, width = 8, b"", 0
    while pos < len(data):
        length, kind = struct.unpack(">I4s", data[pos:pos + 8])
        chunk = data[pos + 8:pos + 8 + length]
        if kind == b"IHDR":
            width, height, depth, ctype, _, _, interlace = struct.unpack(">IIBBBBB", chunk)
            assert depth == 8 and ctype in (2, 6) and interlace == 0, (depth, ctype, interlace)
        elif kind == b"IDAT":
            idat += chunk
        pos += 12 + length
    channels = 4 if ctype == 6 else 3
    raw = zlib.decompress(idat)
    stride, rows, prev = width * channels, [], bytearray(width * channels)
    for y in range(height):
        row = bytearray(raw[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        f = raw[y * (stride + 1)]
        for i in range(stride):
            a = row[i - channels] if i >= channels else 0
            b = prev[i]
            c = prev[i - channels] if i >= channels else 0
            if f == 1: row[i] = (row[i] + a) & 255
            elif f == 2: row[i] = (row[i] + b) & 255
            elif f == 3: row[i] = (row[i] + ((a + b) >> 1)) & 255
            elif f == 4:
                p = a + b - c
                pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                row[i] = (row[i] + (a if pa <= pb and pa <= pc else b if pb <= pc else c)) & 255
        rows.append(bytes(row)); prev = row
    if channels == 3:
        rows = [bytes(v for px in range(width) for v in (*r[px * 3:px * 3 + 3], 255)) for r in rows]
    return width, height, rows


def write_png(path: Path, width: int, height: int, rows) -> bytes:
    def chunk(kind: bytes, body: bytes) -> bytes:
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", zlib.crc32(kind + body) & 0xFFFFFFFF)
    raw = b"".join(b"\x00" + bytes(r) for r in rows)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", struct.pack(">IIBBBBB", width, height, 8, 6, 0, 0, 0)) + chunk(b"IDAT", zlib.compress(raw, 9)) + chunk(b"IEND", b"")
    path.write_bytes(png)
    return png


def resample(width: int, height: int, rows, size: int):
    """Area averaging in premultiplied alpha (no dark fringes at the transparent corners)."""
    scale = width / size
    out = []
    for dy in range(size):
        y0, y1 = dy * scale, (dy + 1) * scale
        line = bytearray()
        for dx in range(size):
            x0, x1 = dx * scale, (dx + 1) * scale
            r = g = b = a = wsum = 0.0
            for sy in range(int(y0), min(int(y1) + 1, height)):
                wy = min(sy + 1, y1) - max(sy, y0)
                if wy <= 0: continue
                row = rows[sy]
                for sx in range(int(x0), min(int(x1) + 1, width)):
                    wx = min(sx + 1, x1) - max(sx, x0)
                    if wx <= 0: continue
                    w = wx * wy
                    pr, pg, pb, pa = row[sx * 4:sx * 4 + 4]
                    r += pr * pa * w; g += pg * pa * w; b += pb * pa * w; a += pa * w; wsum += w
            if a > 0:
                line += bytes((round(r / a), round(g / a), round(b / a), round(a / wsum)))
            else:
                line += b"\x00\x00\x00\x00"
        out.append(bytes(line))
    return out


def render(svg_path: Path, png_path: Path) -> None:
    edge = next((p for p in (r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe", r"C:\Program Files\Microsoft\Edge\Application\msedge.exe") if os.path.exists(p)), None)
    if not edge:
        sys.exit("Microsoft Edge not found (used only to rasterise the SVG)")
    with tempfile.TemporaryDirectory() as temp:
        page = Path(temp) / "p.html"
        page.write_text(f'<html><body style="margin:0;background:transparent"><img src="{svg_path.as_uri()}" width="1024" height="1024" style="display:block"></body></html>', encoding="utf-8")
        subprocess.run([edge, "--headless=new", "--disable-gpu", "--hide-scrollbars", "--default-background-color=00000000", "--window-size=1024,1024",
                        f"--screenshot={png_path}", page.as_uri()], check=True, capture_output=True, timeout=120)


def ico(path: Path, images: list[tuple[int, bytes]]) -> None:
    head = struct.pack("<HHH", 0, 1, len(images))
    offset = 6 + 16 * len(images)
    entries, blobs = b"", b""
    for size, png in images:
        entries += struct.pack("<BBBBHHII", size % 256, size % 256, 0, 0, 1, 32, len(png), offset + len(blobs))
        blobs += png
    path.write_bytes(head + entries + blobs)


def main() -> None:
    OUT.mkdir(parents=True, exist_ok=True)
    for name, (full, small) in MARKS.items():
        (OUT / f"{name}.svg").write_text(svg(full), encoding="utf-8")
        (OUT / f"{name}-small.svg").write_text(svg(small, small=True), encoding="utf-8")
        big, little = OUT / f"{name}-1024.png", Path(tempfile.gettempdir()) / f"{name}-small-1024.png"
        render(OUT / f"{name}.svg", big)
        render(OUT / f"{name}-small.svg", little)
        full_px, small_px = read_png(big), read_png(little)
        images = []
        for size in (16, 24, 32, 48, 64, 128, 256):
            source = small_px if size <= 32 else full_px
            png = write_png(Path(tempfile.gettempdir()) / f"{name}-{size}.png", size, size, resample(*source, size))
            images.append((size, png))
            if size == 256:
                (OUT / f"{name}-256.png").write_bytes(png)
        ico(OUT / f"{name}.ico", images)
        print(name, "ok")


if __name__ == "__main__":
    main()
