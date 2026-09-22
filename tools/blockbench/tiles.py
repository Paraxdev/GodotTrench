"""Pixel art tiles for the GodotTrench low poly nature pack.

Every tile is drawn from a short hue shifted ramp (dark cool shadows, warm light), so the
result reads as hand placed pixels rather than noise.
"""
import math
import random

from PIL import Image


def hexrgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def ramp(*hexes):
    return [hexrgb(h) for h in hexes]


# Dark to light.
PINE = ramp("#132b22", "#1b4430", "#255b38", "#347441", "#4a8a48", "#6aa452")
OAK = ramp("#1b3316", "#27491c", "#355f22", "#487828", "#62912f", "#86ab3c")
BIRCH_LEAF = ramp("#2d4a17", "#3f621c", "#557d22", "#6f9628", "#8fb031", "#b4c94a")
BUSH = ramp("#162f1c", "#214326", "#2e5a2e", "#3e7235", "#548a3b", "#72a346")
BARK = ramp("#2a1b14", "#3b271b", "#4f3522", "#65462b", "#7d5a36", "#957045")
BARK_DARK = ramp("#1f1612", "#2e211a", "#3e2d22", "#503b2b", "#624a36", "#765b42")
BIRCH_BARK = ramp("#2b2724", "#6b6660", "#a9a49a", "#cbc6b9", "#e2ddd0", "#f2eee4")
WOOD = ramp("#6b4a2c", "#86603a", "#a07749", "#b88e5a", "#cda56e", "#dcbb84")
STONE = ramp("#3a3d45", "#4c5058", "#60646b", "#767a7e", "#8d9092", "#a8aaa8")
STONE_WARM = ramp("#433e3b", "#56504b", "#6b645d", "#827a70", "#9a9185", "#b3aa9b")
MOSS = ramp("#22361a", "#2e4a1f", "#3c5f24", "#4f742a", "#658a33", "#7fa03e")
GRASS = ramp("#1f3c17", "#2c521c", "#3d6b22", "#52852a", "#6c9e33", "#8fb845")
DRY = ramp("#6a5a2e", "#86743a", "#a08c48", "#b9a45a", "#cfbb6f", "#e2d08a")


def pick(pal, k):
    """k in 0..1 picks a ramp entry, clamped."""
    i = int(round(k * (len(pal) - 1)))
    return pal[max(0, min(len(pal) - 1, i))]


def new(w, h, fill=(0, 0, 0, 0)):
    return Image.new("RGBA", (w, h), fill)


def put(img, x, y, c, a=255):
    w, h = img.size
    if 0 <= x < w and 0 <= y < h:
        img.putpixel((x, y), (*c[:3], a))


def leaf_clusters(w, h, pal, seed, light=0.0, density=1.0, fringe=0, wrap=True, clump_r=(2, 4)):
    """Overlapping leaf clumps with a lit top left, a dark rim below and a dark gap colour behind them.
    `light` shifts the whole tile brighter (up faces) or darker (down faces), `fringe` cuts a ragged
    transparent edge into the bottom rows so side faces droop."""
    rnd = random.Random(seed)
    img = new(w, h, (*pal[0], 255))
    n = int(w * h / 9 * density)
    clumps = [(rnd.uniform(0, w), rnd.uniform(-2, h), rnd.uniform(*clump_r)) for _ in range(n)]
    clumps.sort(key=lambda c: c[1])
    for cx, cy, r in clumps:
        for dy in range(-int(r) - 1, int(r) + 2):
            for dx in range(-int(r) - 1, int(r) + 2):
                d = math.hypot(dx / r, dy / (r * 0.8))
                if d > 1.0:
                    continue
                x, y = int(cx + dx), int(cy + dy)
                if wrap:
                    x %= w
                if not 0 <= y < h:
                    continue
                # Clumps are lit from the top left, the rim facing down is shaded.
                shade = 0.62 - (dx + dy * 1.4) / (r * 3.2) - d * 0.28
                vertical = 0.12 - y / h * 0.22
                k = shade + vertical + light + rnd.uniform(-0.04, 0.04)
                if d > 0.82 and dy > 0:
                    k -= 0.25
                put(img, x, y, pick(pal, k))
    # Sparse sparkle leaves catch the sun.
    for _ in range(int(w * h / 60)):
        x, y = rnd.randrange(w), rnd.randrange(h)
        if y < h * 0.7:
            put(img, x, y, pal[-1] if light >= 0 else pal[-2])
    if fringe:
        for x in range(w):
            depth = rnd.choice([0, 0, 1, 1, 2, fringe]) if fringe else 0
            for y in range(h - depth, h):
                put(img, x, y, (0, 0, 0), 0)
    return img


def bark(w, h, pal, seed):
    """Vertical ridged bark: wandering dark furrows, lit ridge edges on their left."""
    rnd = random.Random(seed)
    img = new(w, h, (*pal[2], 255))
    furrows = []
    x = rnd.uniform(0, 3)
    while x < w:
        furrows.append(x)
        x += rnd.uniform(3, 5)
    for y in range(h):
        for x in range(w):
            k = 0.5 + rnd.uniform(-0.05, 0.05)
            for f in furrows:
                fx = (f + math.sin(y * 0.45 + f) * 0.9) % w
                d = (x - fx) % w
                if d < 1:
                    k = 0.08
                elif d < 2:
                    k = min(k, 0.3)
                elif d < 3:
                    k = max(k, 0.78)
            put(img, x, y, pick(pal, k))
    # Horizontal breaks across the ridges.
    for _ in range(h // 3):
        x, y = rnd.randrange(w), rnd.randrange(h)
        for dx in range(rnd.randint(1, 3)):
            put(img, (x + dx) % w, y, pal[1])
    return img


def birch_bark(w, h, seed):
    rnd = random.Random(seed)
    pal = BIRCH_BARK
    img = new(w, h, (*pal[4], 255))
    for y in range(h):
        for x in range(w):
            k = 0.78 + rnd.uniform(-0.1, 0.1) + (0.1 if x < w // 3 else 0) - (0.12 if x > w * 0.8 else 0)
            put(img, x, y, pick(pal, k))
    # Lenticels: short dark dashes, a few wide black scars.
    for _ in range(h // 2):
        x, y = rnd.randrange(w), rnd.randrange(h)
        for dx in range(rnd.randint(2, 4)):
            put(img, (x + dx) % w, y, pal[1])
    for _ in range(h // 10):
        x, y = rnd.randrange(w), rnd.randrange(h - 2)
        for dx in range(rnd.randint(3, 6)):
            put(img, (x + dx) % w, y, pal[0])
            if dx % 2 == 0:
                put(img, (x + dx) % w, y + 1, pal[0])
    return img


def rings(s, wood, bark_pal):
    img = new(s, s)
    c = (s - 1) / 2
    for y in range(s):
        for x in range(s):
            d = math.hypot(x - c, y - c) / (s / 2)
            if d > 0.86:
                put(img, x, y, bark_pal[1] if d > 0.95 else bark_pal[3])
            else:
                ring = int(d * 7) % 2
                put(img, x, y, wood[2 + ring] if d > 0.12 else wood[1])
    # A radial crack.
    for t in range(int(s * 0.4)):
        put(img, int(c + t * 0.5), int(c - t), wood[0])
    return img


# Stone --------------------------------------------------------------------------------------------------------------

def stone(w, h, pal, seed, light=0.0, cells=7):
    """Faceted stone: plates with their own tone and a lit rim, a dark crack along a few of their seams, grit on top."""
    rnd = random.Random(seed)
    pts = [(rnd.uniform(0, w), rnd.uniform(0, h), rnd.uniform(-0.14, 0.14)) for _ in range(cells)]
    cracked = {(a, b) for a in range(cells) for b in range(cells) if a < b and rnd.random() < 0.3}
    img = new(w, h)
    for y in range(h):
        for x in range(w):
            ds = sorted((min(abs(x - px), w - abs(x - px)) ** 2 + (y - py) ** 2, i) for i, (px, py, _) in enumerate(pts))
            (d0, i0), (d1, i1) = ds[0], ds[1]
            edge = math.sqrt(d1) - math.sqrt(d0)
            k = 0.55 + pts[i0][2] + light - y / h * 0.12 + rnd.uniform(-0.05, 0.05)
            if edge < 0.9 and (min(i0, i1), max(i0, i1)) in cracked:
                k = 0.1 + light * 0.5
            elif edge < 0.9:
                k -= 0.1
            elif edge < 1.9 and pts[i0][1] < y:
                k += 0.14
            put(img, x, y, pick(pal, k))
    for _ in range(w * h // 25):
        x, y = rnd.randrange(w), rnd.randrange(h)
        put(img, x, y, pick(pal, 0.3 + light + rnd.choice([-0.15, 0.3])))
    return img


def moss_cover(img, pal, seed, depth=5, full=False):
    """Moss over the top rows of a stone tile, hanging down in tongues."""
    rnd = random.Random(seed)
    w, h = img.size
    out = img.copy()
    edge = [depth + int(2 * math.sin(x * 0.7 + seed) + rnd.randint(-1, 2)) for x in range(w)]
    for x in range(w):
        bottom = h if full else max(1, edge[x])
        for y in range(bottom):
            k = 0.6 + rnd.uniform(-0.18, 0.18) - (y / max(1, bottom)) * 0.2
            if y == bottom - 1 and not full:
                k = 0.2
            put(out, x, y, pick(pal, k))
    for _ in range(w * h // 30):
        x, y = rnd.randrange(w), rnd.randrange(h if full else depth)
        put(out, x, y, pal[-1])
    return out


# Cards --------------------------------------------------------------------------------------------------------------

def blades(w, h, pal, seed, count=16, min_h=0.45, heads=None, width=1):
    """Grass blades on transparent background, dark at the root and bright at the tip, bending sideways."""
    rnd = random.Random(seed)
    img = new(w, h)
    for i in range(count):
        x0 = (i + rnd.uniform(0, 1)) * w / count
        height = rnd.uniform(min_h, 1.0) * (h - 1)
        bend = rnd.uniform(-5, 5)
        for t in range(int(height)):
            f = t / height
            x = int(x0 + bend * f * f)
            y = h - 1 - t
            k = 0.12 + f * 0.9 + rnd.uniform(-0.05, 0.05)
            for dx in range(width if f < 0.7 else 1):
                put(img, (x + dx) % w, y, pick(pal, k))
        if heads and rnd.random() < 0.6:
            hx, hy = int(x0 + bend), h - 1 - int(height)
            for dy in range(3):
                put(img, hx % w, hy - dy, pick(heads, 0.8 - dy * 0.2))
                put(img, (hx + (1 if dy % 2 else -1)) % w, hy - dy, pick(heads, 0.5))
    return img


def flower_card(w, h, petals, center, seed, count=4):
    rnd = random.Random(seed)
    img = blades(w, h, GRASS, seed + 1, count=7, min_h=0.25)
    for i in range(count):
        x0 = int((i + 0.5) * w / count + rnd.uniform(-1.5, 1.5))
        top = int(rnd.uniform(0.18, 0.45) * h)
        for y in range(top, h):
            put(img, x0, y, pick(GRASS, 0.3 + (h - y) / h * 0.3))
        # A leaf halfway down the stem.
        ly = rnd.randint(top + 4, h - 3)
        side = rnd.choice([-1, 1])
        put(img, x0 + side, ly, GRASS[3])
        put(img, x0 + 2 * side, ly - 1, GRASS[4])
        pal = petals[i % len(petals)]
        for dx, dy in [(0, -2), (-2, 0), (2, 0), (0, 2), (-1, -1), (1, -1), (-1, 1), (1, 1), (0, -1), (-1, 0), (1, 0), (0, 1)]:
            k = 0.85 - (dy + 2) * 0.12 - abs(dx) * 0.05
            put(img, x0 + dx, top + dy, pick(pal, k))
        put(img, x0, top, hexrgb(center) if isinstance(center, str) else center)
    return img


def frond(w, h, pal, seed):
    """Fern frond pointing up: a rachis with alternating pinnae that shrink toward the tip."""
    rnd = random.Random(seed)
    img = new(w, h)
    cx = w // 2
    for y in range(h):
        put(img, cx, y, pick(pal, 0.25 + (h - y) / h * 0.3))
    for y in range(2, h - 1, 2):
        t = y / h
        reach = int((w / 2 - 1) * math.sin(t * math.pi * 0.95) + 0.5)
        for side in (-1, 1):
            yy = y + (1 if side > 0 else 0)
            for r in range(1, reach + 1):
                ly = yy - r // 3
                k = 0.95 - r / max(1, reach) * 0.45 - t * 0.15 + rnd.uniform(-0.05, 0.05)
                put(img, cx + side * r, ly, pick(pal, k))
                if r < reach - 1:
                    put(img, cx + side * r, ly + 1, pick(pal, k - 0.3))
    return img


def berries(s, seed):
    rnd = random.Random(seed)
    red = ramp("#4a0f14", "#7a1620", "#a8232b", "#cf3a37", "#ec6a5a", "#ffb3a0")
    img = new(s, s, (*red[2], 255))
    for y in range(s):
        for x in range(s):
            d = math.hypot(x - s * 0.35, y - s * 0.35) / s
            put(img, x, y, pick(red, 0.85 - d * 1.2 + rnd.uniform(-0.05, 0.05)))
    put(img, int(s * 0.3), int(s * 0.3), red[-1])
    return img


def cap(w, h, base, spots, seed, spot_count=6):
    rnd = random.Random(seed)
    img = new(w, h)
    for y in range(h):
        for x in range(w):
            put(img, x, y, pick(base, 0.75 - y / h * 0.45 + rnd.uniform(-0.05, 0.05)))
    if spots:
        for _ in range(spot_count):
            x, y = rnd.randrange(1, w - 2), rnd.randrange(1, h - 2)
            for dx, dy in [(0, 0), (1, 0), (0, 1), (1, 1)]:
                if rnd.random() < 0.85:
                    put(img, x + dx, y + dy, spots)
    return img


def solid(w, h, pal, seed, lo=0.3, hi=0.7):
    rnd = random.Random(seed)
    img = new(w, h)
    for y in range(h):
        for x in range(w):
            put(img, x, y, pick(pal, hi - (hi - lo) * y / max(1, h - 1) + rnd.uniform(-0.06, 0.06)))
    return img


class Atlas:
    """A texture sheet plus named UV regions [u0, v0, u1, v1] in pixels."""

    def __init__(self, w, h):
        self.img = new(w, h)
        self.regions = {}

    def add(self, name, tile, x, y):
        self.img.paste(tile, (x, y))
        self.regions[name] = [x, y, x + tile.size[0], y + tile.size[1]]
        return self

    def __getitem__(self, name):
        return self.regions[name]
