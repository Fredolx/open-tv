"""Draws the Lily TV golden retriever logo and writes icon.png / icon.ico / a preview."""
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

S = 2048  # drawn at 2x of 1024 and downsampled for smooth edges
out_dir = Path(sys.argv[1])
out_dir.mkdir(parents=True, exist_ok=True)


def p(v):
    return int(v * S / 1024)


def box(cx, cy, w, h):
    return [p(cx - w / 2), p(cy - h / 2), p(cx + w / 2), p(cy + h / 2)]


BG_TOP = (36, 52, 92)
BG_BOTTOM = (22, 30, 56)
EAR = (176, 104, 38)
EAR_SHADE = (150, 86, 30)
FUR = (226, 162, 72)
FUR_LIGHT = (240, 190, 110)
CREAM = (248, 222, 172)
DARK = (40, 26, 20)
TONGUE = (232, 112, 128)
TONGUE_LINE = (196, 80, 98)

img = Image.new("RGBA", (S, S), (0, 0, 0, 0))

# rounded-square tile with a soft vertical gradient
grad = Image.new("RGBA", (S, S))
gd = ImageDraw.Draw(grad)
for y in range(S):
    t = y / (S - 1)
    gd.line([(0, y), (S, y)], fill=tuple(int(a + (b - a) * t) for a, b in zip(BG_TOP, BG_BOTTOM)) + (255,))
tile = Image.new("L", (S, S), 0)
ImageDraw.Draw(tile).rounded_rectangle([p(24), p(24), p(1000), p(1000)], radius=p(230), fill=255)
img.paste(grad, (0, 0), tile)

# everything else is clipped to the tile
art = Image.new("RGBA", (S, S), (0, 0, 0, 0))
d = ImageDraw.Draw(art)

# chest: a broad body with a scalloped, feathery ruff
d.ellipse(box(512, 1080, 820, 560), fill=FUR)
d.ellipse(box(512, 1010, 380, 260), fill=FUR_LIGHT)
# soft tufts along the top of the lighter bib
for x, y in ((400, 900), (456, 884), (512, 878), (568, 884), (624, 900)):
    d.ellipse(box(x, y, 70, 70), fill=FUR_LIGHT)

# neck joining head to chest
d.rounded_rectangle(box(512, 740, 380, 260), radius=p(120), fill=FUR)


def ear(cx, cy, angle, mirror):
    """A soft, drooping ear with a slightly wavy lower edge."""
    layer = Image.new("RGBA", (S, S), (0, 0, 0, 0))
    ld = ImageDraw.Draw(layer)
    side = -1 if mirror else 1
    ld.ellipse(box(cx, cy, 200, 420), fill=EAR)
    # a gentle wave at the tip rather than curls
    ld.ellipse(box(cx - 30 * side, cy + 160, 130, 130), fill=EAR)
    ld.ellipse(box(cx + 30 * side, cy + 175, 110, 110), fill=EAR)
    ld.ellipse(box(cx + 4 * side, cy + 40, 110, 300), fill=EAR_SHADE)
    return layer.rotate(angle, center=(p(cx), p(cy - 170)), resample=Image.BICUBIC)


art.alpha_composite(ear(292, 500, -18, False))
art.alpha_composite(ear(732, 500, 18, True))
d = ImageDraw.Draw(art)

# head with fluffy cheeks and a lighter forehead blaze
d.ellipse(box(512, 470, 500, 540), fill=FUR)
for cx in (318, 706):
    d.ellipse(box(cx, 600, 150, 150), fill=FUR)
for cx in (356, 668):
    d.ellipse(box(cx, 660, 130, 120), fill=FUR)
d.ellipse(box(512, 372, 230, 250), fill=FUR_LIGHT)
d.ellipse(box(512, 290, 150, 110), fill=FUR_LIGHT)

# muzzle
d.ellipse(box(512, 632, 330, 250), fill=CREAM)

# tongue (behind the mouth line)
d.rounded_rectangle(box(512, 724, 92, 110), radius=p(46), fill=TONGUE)
d.line([p(512), p(690), p(512), p(764)], fill=TONGUE_LINE, width=p(8))

# mouth: from under the nose, curving out each side
w = p(12)
d.line([p(512), p(600), p(512), p(660)], fill=DARK, width=w)
d.arc(box(462, 640, 100, 80), start=10, end=150, fill=DARK, width=w)
d.arc(box(562, 640, 100, 80), start=30, end=170, fill=DARK, width=w)

# nose
d.rounded_rectangle(box(512, 572, 132, 92), radius=p(44), fill=DARK)
d.ellipse(box(486, 552, 38, 22), fill=(110, 90, 84))

# eyes, brows
for ex in (412, 612):
    d.ellipse(box(ex, 468, 70, 76), fill=DARK)
    d.ellipse(box(ex + 12, 454, 22, 22), fill=(255, 255, 255))
d.arc(box(404, 430, 110, 70), start=200, end=320, fill=EAR, width=p(14))
d.arc(box(620, 430, 110, 70), start=220, end=340, fill=EAR, width=p(14))

clipped = Image.new("RGBA", (S, S), (0, 0, 0, 0))
clipped.paste(art, (0, 0), Image.composite(art.getchannel("A"), Image.new("L", (S, S), 0), tile))
img.alpha_composite(clipped)

big = img.resize((1024, 1024), Image.LANCZOS)
big.save(out_dir / "icon-1024.png")
big.resize((128, 128), Image.LANCZOS).save(out_dir / "icon.png")
big.save(out_dir / "icon.ico", sizes=[(16, 16), (24, 24), (32, 32), (48, 48), (64, 64), (128, 128), (256, 256)])

# preview sheet: big + small sizes on light and dark backgrounds
sheet = Image.new("RGBA", (1400, 560), (240, 240, 240, 255))
ImageDraw.Draw(sheet).rectangle([700, 0, 1400, 560], fill=(30, 30, 30, 255))
for ox in (0, 700):
    sheet.alpha_composite(big.resize((256, 256), Image.LANCZOS), (ox + 40, 40))
    x = ox + 340
    for size in (128, 64, 32, 16):
        sheet.alpha_composite(big.resize((size, size), Image.LANCZOS), (x, 40))
        x += size + 20
sheet.save(out_dir / "preview.png")
print("ok")
