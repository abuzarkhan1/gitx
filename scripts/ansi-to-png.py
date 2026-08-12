#!/usr/bin/env python3
"""Render an ANSI terminal capture (as saved by `tmux capture-pane -pe`) to
a PNG, used to regenerate the README dashboard screenshot.

Usage: scripts/ansi-to-png.py < capture.txt > docs/assets/gitx-dashboard.png

Handles SGR 38/48 with both 256-color (38;5;N) and truecolor (38;2;r;g;b)
sequences, the standard 256-color palette, and bold/intensity. Requires
Pillow.
"""

import re
import sys
from PIL import Image, ImageDraw, ImageFont

# Standard xterm 256-color palette.
PALETTE = [
    0x000000, 0x800000, 0x008000, 0x808000, 0x000080, 0x800080, 0x008080,
    0xC0C0C0, 0x808080, 0xFF0000, 0x00FF00, 0xFFFF00, 0x0000FF, 0xFF00FF,
    0x00FFFF, 0xFFFFFF,
]
PALETTE += [0x000000, 0x00005F, 0x000087, 0x0000AF, 0x0000D7, 0x0000FF,
            0x005F00, 0x005F5F, 0x005F87, 0x005FAF, 0x005FD7, 0x005FFF,
            0x008700, 0x00875F, 0x008787, 0x0087AF, 0x0087D7, 0x0087FF,
            0x00AF00, 0x00AF5F, 0x00AF87, 0x00AFAF, 0x00AFD7, 0x00AFFF,
            0x00D700, 0x00D75F, 0x00D787, 0x00D7AF, 0x00D7D7, 0x00D7FF,
            0x00FF00, 0x00FF5F, 0x00FF87, 0x00FFAF, 0x00FFD7, 0x00FFFF,
            0x5F0000, 0x5F005F, 0x5F0087, 0x5F00AF, 0x5F00D7, 0x5F00FF,
            0x5F5F00, 0x5F5F5F, 0x5F5F87, 0x5F5FAF, 0x5F5FD7, 0x5F5FFF,
            0x5F8700, 0x5F875F, 0x5F8787, 0x5F87AF, 0x5F87D7, 0x5F87FF,
            0x5FAF00, 0x5FAF5F, 0x5FAF87, 0x5FAFAF, 0x5FAFD7, 0x5FAFFF,
            0x5FD700, 0x5FD75F, 0x5FD787, 0x5FD7AF, 0x5FD7D7, 0x5FD7FF,
            0x5FFF00, 0x5FFF5F, 0x5FFF87, 0x5FFFAF, 0x5FFFD7, 0x5FFFFF,
            0x870000, 0x87005F, 0x870087, 0x8700AF, 0x8700D7, 0x8700FF,
            0x875F00, 0x875F5F, 0x875F87, 0x875FAF, 0x875FD7, 0x875FFF,
            0x878700, 0x87875F, 0x878787, 0x8787AF, 0x8787D7, 0x8787FF,
            0x87AF00, 0x87AF5F, 0x87AF87, 0x87AFAF, 0x87AFD7, 0x87AFFF,
            0x87D700, 0x87D75F, 0x87D787, 0x87D7AF, 0x87D7D7, 0x87D7FF,
            0x87FF00, 0x87FF5F, 0x87FF87, 0x87FFAF, 0x87FFD7, 0x87FFFF,
            0xAF0000, 0xAF005F, 0xAF0087, 0xAF00AF, 0xAF00D7, 0xAF00FF,
            0xAF5F00, 0xAF5F5F, 0xAF5F87, 0xAF5FAF, 0xAF5FD7, 0xAF5FFF,
            0xAF8700, 0xAF875F, 0xAF8787, 0xAF87AF, 0xAF87D7, 0xAF87FF,
            0xAFAF00, 0xAFAF5F, 0xAFAF87, 0xAFAFAF, 0xAFAFD7, 0xAFAFFF,
            0xAFD700, 0xAFD75F, 0xAFD787, 0xAFD7AF, 0xAFD7D7, 0xAFD7FF,
            0xAFFF00, 0xAFFF5F, 0xAFFF87, 0xAFFFAF, 0xAFFFD7, 0xAFFFFF,
            0xD70000, 0xD7005F, 0xD70087, 0xD700AF, 0xD700D7, 0xD700FF,
            0xD75F00, 0xD75F5F, 0xD75F87, 0xD75FAF, 0xD75FD7, 0xD75FFF,
            0xD78700, 0xD7875F, 0xD78787, 0xD787AF, 0xD787D7, 0xD787FF,
            0xD7AF00, 0xD7AF5F, 0xD7AF87, 0xD7AFAF, 0xD7AFD7, 0xD7AFFF,
            0xD7D700, 0xD7D75F, 0xD7D787, 0xD7D7AF, 0xD7D7D7, 0xD7D7FF,
            0xD7FF00, 0xD7FF5F, 0xD7FF87, 0xD7FFAF, 0xD7FFD7, 0xD7FFFF,
            0xFF0000, 0xFF005F, 0xFF0087, 0xFF00AF, 0xFF00D7, 0xFF00FF,
            0xFF5F00, 0xFF5F5F, 0xFF5F87, 0xFF5FAF, 0xFF5FD7, 0xFF5FFF,
            0xFF8700, 0xFF875F, 0xFF8787, 0xFF87AF, 0xFF87D7, 0xFF87FF,
            0xFFAF00, 0xFFAF5F, 0xFFAF87, 0xFFAFAF, 0xFFAFD7, 0xFFAFFF,
            0xFFD700, 0xFFD75F, 0xFFD787, 0xFFD7AF, 0xFFD7D7, 0xFFD7FF,
            0xFFFF00, 0xFFFF5F, 0xFFFF87, 0xFFFFAF, 0xFFFFD7, 0xFFFFF]
PALETTE += [0x080808, 0x121212, 0x1C1C1C, 0x262626, 0x303030, 0x3A3A3A,
            0x444444, 0x4E4E4E, 0x585858, 0x626262, 0x6C6C6C, 0x767676,
            0x808080, 0x8A8A8A, 0x949494, 0x9E9E9E, 0xA8A8A8, 0xB2B2B2,
            0xBCBCBC, 0xC6C6C6, 0xD0D0D0, 0xDADADA, 0xE4E4E4, 0xEEEEEE]

SGR = re.compile(rb"\x1b\[([0-9;]*)m")
OTHER_CSI = re.compile(rb"\x1b\[[0-9;?]*[ -/]*[@-~]")


def color_of(code):
    return PALETTE[code]


def bright(hexv):
    r = min(255, ((hexv >> 16) & 0xFF) + 40)
    g = min(255, ((hexv >> 8) & 0xFF) + 40)
    b = min(255, (hexv & 0xFF) + 40)
    return (r << 16) | (g << 8) | b


def parse_sgr(params):
    """Return (fg, bg) after applying an SGR parameter list."""
    fg = 0xFFFFFF
    bg = 0x000000
    i = 0
    while i < len(params):
        p = params[i]
        if p == 0:
            fg, bg = 0xFFFFFF, 0x000000
        elif p == 1:
            fg = bright(fg)
        elif p == 7:
            fg, bg = bg, fg
        elif 30 <= p <= 37:
            fg = PALETTE[p - 30]
        elif p == 38 and i + 2 < len(params) and params[i + 1] == 5:
            fg = color_of(params[i + 2])
            i += 2
        elif p == 38 and i + 4 < len(params) and params[i + 1] == 2:
            fg = (params[i + 2] << 16) | (params[i + 3] << 8) | params[i + 4]
            i += 4
        elif 40 <= p <= 47:
            bg = PALETTE[p - 40]
        elif p == 48 and i + 2 < len(params) and params[i + 1] == 5:
            bg = color_of(params[i + 2])
            i += 2
        elif p == 48 and i + 4 < len(params) and params[i + 1] == 2:
            bg = (params[i + 2] << 16) | (params[i + 3] << 8) | params[i + 4]
            i += 4
        elif 90 <= p <= 97:
            fg = bright(PALETTE[p - 90])
        elif 100 <= p <= 107:
            bg = bright(PALETTE[p - 100])
        i += 1
    return fg, bg


def render_line(draw, line, y, font, cell_w, cell_h):
    line = re.sub(OTHER_CSI, b"", line)  # drop cursor-move/clear escapes
    x = 10
    pos = 0
    fg, bg = 0xFFFFFF, 0x000000
    for m in SGR.finditer(line):
        segment = line[pos:m.start()]
        x = draw_segment(draw, segment, x, y, fg, bg, font, cell_w, cell_h)
        fg, bg = parse_sgr([int(p) for p in m.group(1).split(b";") if p])
        pos = m.end()
    draw_segment(draw, line[pos:], x, y, fg, bg, font, cell_w, cell_h)


def draw_segment(draw, segment, x, y, fg, bg, font, cell_w, cell_h):
    for ch in segment.decode("utf-8", "replace"):
        if bg != 0x000000:
            draw.rectangle((x, y, x + cell_w, y + cell_h), fill=_rgb(bg))
        if ch != " " and font is not None:
            draw.text((x, y), ch, font=font, fill=_rgb(fg))
        x += cell_w
    return x


def _rgb(hexv):
    return ((hexv >> 16) & 0xFF, (hexv >> 8) & 0xFF, hexv & 0xFF)


def main():
    data = sys.stdin.buffer.read()
    lines = data.split(b"\n")
    if lines and lines[-1] == b"":
        lines.pop()

    cell_w, cell_h = 10, 20
    font = None
    for candidate in [
        "/System/Library/Fonts/Menlo.ttc",
        "/System/Library/Fonts/SFNSMono.ttf",
        "/Library/Fonts/Monaco.ttf",
    ]:
        try:
            font = ImageFont.truetype(candidate, 15)
            break
        except OSError:
            continue

    rows = len(lines) if lines else 1
    cols = 1
    for line in lines:
        plain = re.sub(SGR, b"", re.sub(OTHER_CSI, b"", line)).rstrip(b"\x00")
        cols = max(cols, len(plain.decode("utf-8", "replace")))
    img = Image.new("RGB", (cols * cell_w + 20, rows * cell_h + 20), (12, 12, 12))
    draw = ImageDraw.Draw(img)
    y = 10
    for line in lines:
        render_line(draw, line, y, font, cell_w, cell_h)
        y += cell_h
    img.save(sys.stdout.buffer, "PNG")


if __name__ == "__main__":
    main()
