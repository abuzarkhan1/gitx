#!/usr/bin/env python3
"""Render a .excalidraw file to a PNG.

Excalidraw files are JSON and need the Excalidraw app (excalidraw.com, VS Code
extension, Obsidian plugin) to view. This script renders the subset of the
format this repo's diagrams use — rectangles, arrows, and text — to a PNG so
the diagram can be opened in any image viewer or embedded in markdown.

Usage:
    python3 scripts/render-excalidraw-to-png.py <input.excalidraw> <output.png>
    python3 scripts/render-excalidraw-to-png.py <input.excalidraw> <output.png> --scale 3

Draw order follows the file's element order (which matches its z-index).
Text bound to a container is wrapped to the container width and centered;
free-standing text is drawn at its coordinates.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

try:
    from PIL import Image, ImageDraw, ImageFont
except ImportError:
    sys.exit("error: Pillow is required (pip install Pillow)")


FONT_CANDIDATES = [
    "/System/Library/Fonts/Supplemental/Arial.ttf",
    "/System/Library/Fonts/Helvetica.ttc",
    "/Library/Fonts/Arial.ttf",
    "C:/Windows/Fonts/arial.ttf",
    "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
]


def load_font(size: int) -> ImageFont.FreeTypeFont:
    for path in FONT_CANDIDATES:
        if Path(path).exists():
            try:
                return ImageFont.truetype(path, size)
            except OSError:
                continue
    return ImageFont.load_default()


def hex_color(value: str) -> tuple[int, int, int, int]:
    value = (value or "#000000").strip().lstrip("#")
    if len(value) == 3:
        value = "".join(c * 2 for c in value)
    if len(value) == 4:  # #RGBA
        value = "".join(c * 2 for c in value)
    r = int(value[0:2], 16)
    g = int(value[2:4], 16)
    b = int(value[4:6], 16)
    a = int(value[6:8], 16) if len(value) == 8 else 255
    return (r, g, b, a)


def wrap_text(text: str, font: ImageFont.FreeTypeFont, max_width: int) -> list[str]:
    lines: list[str] = []
    for raw in text.split("\n"):
        words = raw.split(" ")
        if not words:
            lines.append("")
            continue
        current = words[0]
        for word in words[1:]:
            candidate = current + " " + word
            if font.getlength(candidate) <= max_width:
                current = candidate
            else:
                lines.append(current)
                current = word
        lines.append(current)
    return lines


def draw_dashed_line(
    draw: ImageDraw.ImageDraw,
    start: tuple[float, float],
    end: tuple[float, float],
    fill: tuple[int, int, int, int],
    width: int,
    dash: int = 8,
    gap: int = 6,
) -> None:
    x1, y1 = start
    x2, y2 = end
    dx, dy = x2 - x1, y2 - y1
    length = (dx * dx + dy * dy) ** 0.5
    if length == 0:
        return
    ux, uy = dx / length, dy / length
    pos = 0.0
    on = True
    while pos < length:
        end_pos = min(pos + (dash if on else gap), length)
        if on:
            draw.line(
                [(x1 + ux * pos, y1 + uy * pos), (x1 + ux * end_pos, y1 + uy * end_pos)],
                fill=fill,
                width=width,
            )
        on = not on
        pos = end_pos


def render(input_path: str, output_path: str, scale: int) -> None:
    data = json.loads(Path(input_path).read_text(encoding="utf-8"))
    elements = data.get("elements", [])

    # Bounding box of the whole drawing.
    min_x = min_y = float("inf")
    max_x = max_y = float("-inf")
    for el in elements:
        min_x = min(min_x, el.get("x", 0))
        min_y = min(min_y, el.get("y", 0))
        max_x = max(max_x, el.get("x", 0) + el.get("width", 0))
        max_y = max(max_y, el.get("y", 0) + el.get("height", 0))

    bg = hex_color(data.get("appState", {}).get("viewBackgroundColor", "#ffffff"))
    padding = 40
    canvas_w = int(max_x - min_x + 2 * padding)
    canvas_h = int(max_y - min_y + 2 * padding)
    img = Image.new("RGBA", (canvas_w * scale, canvas_h * scale), bg)
    draw = ImageDraw.Draw(img)
    offset = (padding - min_x, padding - min_y)

    def tx(x: float) -> float:
        return (x + offset[0]) * scale

    def ty(y: float) -> float:
        return (y + offset[1]) * scale

    containers = {el["id"]: el for el in elements if el["type"] == "rectangle"}

    for el in elements:
        if el.get("isDeleted"):
            continue
        etype = el["type"]
        x, y = el.get("x", 0), el.get("y", 0)
        w, h = el.get("width", 0), el.get("height", 0)
        stroke = hex_color(el.get("strokeColor", "#1e1e1e"))
        stroke_w = max(1, int(el.get("strokeWidth", 2) * scale))
        dashed = el.get("strokeStyle") == "dashed"

        if etype == "rectangle":
            fill = hex_color(el.get("backgroundColor", "transparent"))
            radius = 12 * scale
            draw.rounded_rectangle(
                [tx(x), ty(y), tx(x + w), ty(y + h)],
                radius=radius,
                fill=fill,
                outline=None,
            )
            if dashed:
                r = radius
                # dashed outline via rounded-rect segments is complex; draw a
                # plain dashed rectangle inside the fill instead.
                inset = stroke_w / 2
                draw_dashed_line(
                    draw,
                    (tx(x) + inset, ty(y) + inset),
                    (tx(x + w) - inset, ty(y) + inset),
                    stroke,
                    stroke_w,
                )
                draw_dashed_line(
                    draw,
                    (tx(x + w) - inset, ty(y) + inset),
                    (tx(x + w) - inset, ty(y + h) - inset),
                    stroke,
                    stroke_w,
                )
                draw_dashed_line(
                    draw,
                    (tx(x + w) - inset, ty(y + h) - inset),
                    (tx(x) + inset, ty(y + h) - inset),
                    stroke,
                    stroke_w,
                )
                draw_dashed_line(
                    draw,
                    (tx(x) + inset, ty(y + h) - inset),
                    (tx(x) + inset, ty(y) + inset),
                    stroke,
                    stroke_w,
                )
            else:
                draw.rounded_rectangle(
                    [tx(x), ty(y), tx(x + w), ty(y + h)],
                    radius=radius,
                    outline=stroke,
                    width=stroke_w,
                )

        elif etype == "arrow":
            points = el.get("points") or [[0, 0]]
            sx = x + points[0][0]
            sy = y + points[0][1]
            ex = x + points[-1][0]
            ey = y + points[-1][1]
            if dashed:
                draw_dashed_line(
                    draw, (tx(sx), ty(sy)), (tx(ex), ty(ey)), stroke, stroke_w
                )
            else:
                draw.line(
                    [(tx(sx), ty(sy)), (tx(ex), ty(ey))],
                    fill=stroke,
                    width=stroke_w,
                )
            # arrowhead at the end
            dx, dy = ex - sx, ey - sy
            length = (dx * dx + dy * dy) ** 0.5
            if length > 0:
                ux, uy = dx / length, dy / length
                head = 14 * scale
                base = (tx(ex) - ux * head, ty(ey) - uy * head)
                px, py = -uy, ux
                draw.polygon(
                    [
                        (tx(ex), ty(ey)),
                        (base[0] + px * head * 0.4, base[1] + py * head * 0.4),
                        (base[0] - px * head * 0.4, base[1] - py * head * 0.4),
                    ],
                    fill=stroke,
                )

        elif etype == "text":
            size = max(6, int(el.get("fontSize", 16)))
            font = load_font(size * scale)
            align = el.get("textAlign", "left")
            valign = el.get("verticalAlign", "top")
            container = containers.get(el.get("containerId") or "")

            if container is not None:
                box_x, box_y = container["x"], container["y"]
                box_w, box_h = container["width"], container["height"]
                max_w = max(20, (box_w - 24)) * scale
                lines = wrap_text(el.get("text", ""), font, max_w)
                line_h = int(font.size * 1.25)
                block_h = line_h * len(lines)
                text_w = max(font.getlength(l) for l in lines) if lines else 0
                text_x = tx(box_x) + (box_w * scale - text_w) / 2
                if valign == "middle":
                    text_y = ty(box_y) + (box_h * scale - block_h) / 2
                else:
                    text_y = ty(box_y) + 8 * scale
            else:
                max_w = max(20, el.get("width", 200)) * scale
                lines = wrap_text(el.get("text", ""), font, max_w)
                line_h = int(font.size * 1.25)
                if align == "center":
                    text_x = tx(x) + (w * scale - max(font.getlength(l) for l in lines)) / 2
                else:
                    text_x = tx(x)
                text_y = ty(y)

            for i, line in enumerate(lines):
                if align == "center" and container is None:
                    lx = tx(x) + (w * scale - font.getlength(line)) / 2
                elif align == "center":
                    lx = tx(box_x) + (box_w * scale - font.getlength(line)) / 2
                else:
                    lx = text_x
                draw.text((lx, text_y + i * line_h), line, font=font, fill=stroke)

    # downscale for antialiasing
    img = img.resize((canvas_w, canvas_h), Image.LANCZOS)
    img.convert("RGB").save(output_path, "PNG")
    print(
        f"rendered {len(elements)} elements -> {output_path} "
        f"({canvas_w}x{canvas_h})"
    )


def main() -> None:
    parser = argparse.ArgumentParser(description="Render a .excalidraw file to PNG")
    parser.add_argument("input", help="path to the .excalidraw file")
    parser.add_argument("output", help="path for the output PNG")
    parser.add_argument("--scale", type=int, default=2, help="supersampling scale")
    args = parser.parse_args()
    render(args.input, args.output, args.scale)


if __name__ == "__main__":
    main()
