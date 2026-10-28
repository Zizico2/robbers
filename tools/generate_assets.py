#!/usr/bin/env python3
"""Draw Robbers' original tile/pawn art with Pillow. No model or downloaded art.

Run from any directory: python3 tools/generate_assets.py
Fixed geometry and seeds make outputs reproducible. Pillow is the only dependency.
"""
from pathlib import Path
import math
import random
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "assets"
SIZE = 320
INK = "#263c42"
PAPER = "#f1e8ce"
KINDS = ["residential", "market", "park", "bank", "airport", "police", "hospital", "school"]
COLORS = {"residential": "#c9d4bc", "market": "#ddc7a4", "park": "#9dbba1", "bank": "#d8c994",
          "airport": "#b5c9ce", "police": "#acc4cd", "hospital": "#d4c7b9", "school": "#d0b89d"}


def font(size):
    # Pillow ships its own default font: no system-font dependency.
    return ImageFont.load_default(size=size)


def hexagon(radius=153, cy=160):
    return [(160 + radius * math.cos(math.radians(a)), cy + radius * math.sin(math.radians(a)))
            for a in [-90, -30, 30, 90, 150, 210]]


def tree(draw, x, y, radius=12):
    draw.ellipse((x-radius+3, y-radius+5, x+radius+3, y+radius+5), fill="#75907d")
    draw.ellipse((x-radius, y-radius, x+radius, y+radius), fill="#507d68", outline=INK, width=2)
    draw.arc((x-radius+4, y-radius+3, x+radius-3, y+radius-4), 150, 270, fill="#9cbb8e", width=3)


def building(draw, box, color="#b17354", roof="#dbad83"):
    x, y, w, h = box
    draw.rounded_rectangle((x+5, y+7, x+w+7, y+h+7), radius=3, fill="#819080")
    draw.rectangle((x, y, x+w, y+h), fill=color, outline=INK, width=3)
    draw.rectangle((x-2, y-5, x+w+2, y+12), fill=roof, outline=INK, width=3)
    for wx in range(x+9, x+w-4, 17):
        for wy in range(y+22, y+h-3, 20):
            draw.rectangle((wx, wy, wx+7, wy+9), fill="#e6dcb5", outline=INK, width=1)
    draw.rectangle((x+w//2-5, y+h-13, x+w//2+5, y+h), fill=INK)


def tile(kind, variant=0):
    rng = random.Random(50 + variant)
    im = Image.new("RGBA", (SIZE, SIZE))
    d = ImageDraw.Draw(im)
    d.polygon(hexagon(), fill=INK)
    d.polygon(hexagon(148), fill=PAPER)
    d.polygon(hexagon(140), fill=COLORS[kind])
    # Fine printed flecks, kept subtle and clipped by the hex mask below.
    for _ in range(340):
        x, y = rng.randrange(25, 295), rng.randrange(20, 295)
        d.point((x, y), fill="#a5b49c")
    # Decorative pedestrian paths: never imply restricted edge connections.
    d.line([(55, 224), (100, 203), (215, 203), (264, 225)], fill="#e9dec0", width=16)
    d.line([(160, 65), (160, 207)], fill="#e9dec0", width=13)
    if kind == "residential":
        building(d, (80, 100, 55, 59), "#b27053", "#dba77d")
        building(d, (171, 133, 62, 54), "#66878a", "#b0c1b3")
        building(d, (174, 76, 39, 39), "#c29f62", "#e2c790")
        tree(d, 77, 180); tree(d, 238, 120); tree(d, 130, 190, 9)
    elif kind == "park":
        d.ellipse((114, 126, 207, 191), fill="#668e99", outline=INK, width=3)
        d.ellipse((133, 138, 187, 175), fill="#a1c1be")
        for x, y in [(91, 108), (197, 91), (230, 157), (80, 174), (147, 83)]: tree(d, x, y, 15)
        d.rectangle((102, 197, 132, 202), fill="#ae7d56", outline=INK)
    elif kind == "airport":
        d.polygon([(91, 83), (122, 72), (207, 207), (178, 220)], fill="#53696e", outline=INK)
        d.line([(109, 88), (188, 207)], fill="#dddec6", width=3)
        building(d, (172, 91, 63, 60), "#879c9d", "#d3dfd5")
        d.polygon([(131, 116), (140, 133), (168, 138), (169, 145), (144, 142), (146, 157),
                   (154, 164), (150, 169), (135, 160), (130, 139), (111, 149), (105, 144),
                   (128, 130), (125, 116)], fill=PAPER, outline=INK, width=2)
        tree(d, 229, 191)
    elif kind == "bank":
        building(d, (96, 106, 128, 80), "#b7a471", "#eadab1")
        d.polygon([(87, 110), (160, 72), (233, 110)], fill=PAPER, outline=INK, width=3)
        d.text((160, 99), "$", font=font(23), anchor="mm", fill=INK)
        for x in [104, 133, 177, 205]: d.rectangle((x, 118, x+10, 179), fill=PAPER, outline=INK, width=2)
        d.rectangle((89, 185, 231, 194), fill=PAPER, outline=INK, width=2)
        tree(d, 73, 169, 11); tree(d, 249, 168, 11)
    elif kind == "hospital":
        building(d, (93, 98, 133, 93), "#c6c8b8", "#f2eddb")
        d.rectangle((151, 116, 169, 162), fill="#bb5c50")
        d.rectangle((137, 130, 183, 147), fill="#bb5c50")
        tree(d, 75, 175); tree(d, 247, 157)
    elif kind == "police":
        building(d, (104, 96, 112, 96), "#56747e", "#a2b7b8")
        d.polygon([(144, 124), (160, 119), (177, 124), (173, 148), (160, 159), (147, 148)],
                  fill="#dfc78c", outline=INK, width=2)
        d.text((160, 137), "*", font=font(25), anchor="mm", fill=INK)
        d.rectangle((77, 201, 106, 213), fill=PAPER, outline=INK, width=2)
        d.rectangle((87, 198, 98, 202), fill="#af5549")
        tree(d, 238, 164)
    elif kind == "school":
        building(d, (85, 111, 143, 77), "#bd8163", "#d8b995")
        building(d, (141, 92, 38, 96), "#bb9271", "#eee0ba")
        d.ellipse((148, 114, 171, 137), fill=PAPER, outline=INK, width=2)
        d.line([(160, 119), (160, 126), (166, 128)], fill=INK, width=2)
        d.line([(198, 95), (198, 66)], fill=INK, width=3)
        d.polygon([(199, 67), (222, 73), (199, 81)], fill="#ba5e4d")
        tree(d, 65, 170); tree(d, 246, 177)
    elif kind == "market":
        for x, y, color in [(80, 111, "#b76f56"), (172, 92, "#c7a66c"), (161, 169, "#6d9391")]:
            d.rectangle((x, y, x+64, y+39), fill=PAPER, outline=INK, width=2)
            for stripe in range(4):
                d.rectangle((x+stripe*16, y, x+stripe*16+8, y+17), fill=color)
            d.rectangle((x+12, y+25, x+52, y+35), fill=color, outline=INK)
        tree(d, 87, 187, 11)
    label = "HOMES" if kind == "residential" else kind.upper()
    d.rounded_rectangle((83, 236, 237, 263), radius=5, fill=PAPER, outline=INK, width=1)
    d.text((160, 249), label, font=font(17), anchor="mm", fill=INK)
    # Clip the full painting to the regular hex, leaving real transparent corners.
    mask = Image.new("L", im.size)
    ImageDraw.Draw(mask).polygon(hexagon(), fill=255)
    im.putalpha(mask)
    return im


def pawn(color):
    im = Image.new("RGBA", (96, 120))
    d = ImageDraw.Draw(im)
    d.ellipse((17, 91, 83, 111), fill="#172e3670")
    d.rounded_rectangle((22, 52, 74, 103), radius=17, fill=INK)
    d.rounded_rectangle((26, 51, 70, 97), radius=15, fill=color)
    d.ellipse((22, 12, 74, 64), fill=INK)
    d.ellipse((26, 15, 70, 59), fill=color)
    d.arc((31, 20, 65, 53), 180, 260, fill="#fff0cd", width=4)
    d.line((35, 63, 32, 86), fill="#fff0cd", width=3)
    return im


def main():
    (OUT / "tiles").mkdir(parents=True, exist_ok=True)
    (OUT / "pawns").mkdir(parents=True, exist_ok=True)
    preview = Image.new("RGB", (SIZE*4, SIZE*2), "#182c33")
    for index, kind in enumerate(KINDS):
        image = tile(kind)
        image.save(OUT / "tiles" / f"{kind}.png")
        preview.paste(image, ((index % 4)*SIZE, (index // 4)*SIZE), image)
    for index, color in enumerate(["#df4d4a", "#4da3d6", "#efb247", "#50b892"]):
        pawn(color).save(OUT / "pawns" / f"{index}.png")
    preview.save(OUT / "tile_catalog.png")
    print("Generated 8 city tiles, 4 pawns, and tile_catalog.png")


if __name__ == "__main__":
    main()
