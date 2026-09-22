#!/usr/bin/env python3
"""Builds the demo project's textures in godot/demo/textures from CC0 photo scans.

  python tools/fetch_demo_textures.py                 # rebuild every texture in TEXTURES
  python tools/fetch_demo_textures.py planks cobble   # rebuild only these
  python tools/fetch_demo_textures.py --check         # verify the files on disk, no network

Needs Pillow and numpy. Every entry in TEXTURES names a face material ("showcase/planks"), where its
photo comes from and the world size one repeat covers in map units (32 units per meter). Poly Haven
publishes the real size of each scan, so for those the size is derived from the /info endpoint unless
the table overrides it. The size is written into the material as metadata/texture_size, which the
editor and the Godot addon both use for UVs instead of the image's pixel size.

Downloads are cached in target/demo_texture_cache. The script writes the albedo, the OpenGL normal
map, a roughness map where one is listed, the .tres material and CREDITS.md. A few surfaces have no
fitting photo (glass, stained glass, water) and are drawn here instead, some from CC0 photos.
A scan that ships an emission map (ambientCG facades) keeps it when its extra enables emission, and a size
can be a (width, height) pair for textures that are not square, such as signs.
"""
import argparse
import io
import json
import math
import pathlib
import sys
import urllib.request
import zipfile

import numpy as np
from PIL import Image

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "godot" / "demo" / "textures"
CACHE = ROOT / "target" / "demo_texture_cache"
RES = "res://demo/textures"
UNITS_PER_METER = 32.0
RESOLUTION = 1024
JPEG_QUALITY = 88

# Poly Haven's CDN rejects urllib's default user agent.
HEADERS = {"User-Agent": "Mozilla/5.0 (compatible; GodotTrenchTextureFetch/1.0)"}

PH = "polyhaven"
ACG = "ambientcg"
MADE = "made"

ALPHA_CUTOUT = "transparency = 2\nalpha_scissor_threshold = 0.5\ncull_mode = 2\n"


def tex(name, source, asset, size=None, normal=True, roughness=False, rotate=False, level=None, extra="", alpha=False, fmt=None, recipe=None, uses=None):
    """One texture. size is in map units, None takes the scan's real size from Poly Haven.
    rotate turns the photo a quarter turn so grain or rows run the other way. level is (mean brightness,
    saturation) to repaint a scan, for example grey paint into white. uses lists the other assets a drawn
    texture is made from, for the credits."""
    return {
        "name": name,
        "source": source,
        "asset": asset,
        "size": size,
        "normal": normal,
        "roughness": roughness,
        "rotate": rotate,
        "level": level,
        "extra": extra,
        "alpha": alpha,
        "fmt": fmt or ("png" if alpha else "jpg"),
        "recipe": recipe,
        "uses": uses or [],
    }


TEXTURES = [
    tex("showcase/stone_bricks", PH, "medieval_blocks_03"),
    tex("showcase/red_bricks", PH, "large_red_bricks"),
    tex("showcase/planks", PH, "wood_planks"),
    tex("showcase/planks_dark", PH, "dark_wooden_planks"),
    tex("showcase/beam", PH, "rough_wood", rotate=True),
    tex("showcase/logs", PH, "wood_trunk_wall", rotate=True),
    tex("showcase/bark", PH, "pine_bark"),
    tex("showcase/white_paint", ACG, "Planks010", size=64, level=(212, 0.3)),
    tex("showcase/red_paint", ACG, "PaintedWood003", size=48),
    tex("showcase/plaster", PH, "painted_plaster_wall"),
    tex("showcase/cobble", PH, "cobblestone_floor_07"),
    tex("showcase/church_tiles", PH, "floor_tiles_06"),
    tex("showcase/roof_red", PH, "clay_roof_tiles_02"),
    tex("showcase/roof_slate", PH, "grey_roof_tiles_02"),
    tex("showcase/carpet", ACG, "Fabric026", size=32),
    tex("showcase/metal", ACG, "Metal046B", size=32, roughness=True, extra="metallic = 0.85\n"),
    tex("showcase/gold", ACG, "Metal048C", size=32, level=(170, 1.5), roughness=True, extra="metallic = 1.0\n"),
    tex("showcase/grass", ACG, "Grass004", size=64),
    tex("showcase/grass_dark", ACG, "Grass002", size=64),
    tex("showcase/dirt", PH, "dirt_floor"),
    tex("showcase/sand", PH, "sand_01"),
    tex("showcase/snow", ACG, "Snow005", size=64, level=(230, 0.5)),
    tex("showcase/rock", PH, "marble_rock_01"),
    tex("showcase/cliff", ACG, "Rock028", size=96),
    tex("showcase/leaves", MADE, "leaves", size=32, normal=False, alpha=True, extra=ALPHA_CUTOUT, recipe="scatter_leaves", uses=[(ACG, "LeafSet024")]),
    tex("showcase/pine", MADE, "pine", size=32, normal=False, alpha=True, extra=ALPHA_CUTOUT, recipe="scatter_needles", uses=[(ACG, "PineNeedles001")]),
    tex("showcase/iron_bars", MADE, "iron_bars", size=64, normal=False, alpha=True, extra=ALPHA_CUTOUT + "metallic = 0.8\n", recipe="iron_bars", uses=[(ACG, "Metal046B")]),
    tex("showcase/chalkboard", MADE, "chalkboard", size=64, normal=False, recipe="chalkboard", uses=[(PH, "painted_plaster_wall")]),
    tex("showcase/lamp", MADE, "lamp", size=32, normal=False, recipe="lamp", uses=[(PH, "white_plaster_02")],
        extra="emission_enabled = true\nemission = Color(1, 0.9, 0.6, 1)\nemission_energy_multiplier = 3.0\n"),
    tex("showcase/glass", MADE, "glass", size=64, normal=False, alpha=True, recipe="glass",
        extra="transparency = 1\nalbedo_color = Color(1, 1, 1, 0.35)\nroughness = 0.1\nmetallic_specular = 0.9\n"),
    tex("showcase/stained_glass", MADE, "stained_glass", size=64, normal=False, recipe="stained_glass",
        extra="emission_enabled = true\nemission = Color(1, 1, 1, 1)\nemission_energy_multiplier = 0.6\nemission_operator = 1\nemission_texture = ExtResource(\"1_albedo\")\n"),
    tex("showcase/water", MADE, "water", size=128, recipe="water",
        extra="transparency = 1\nalbedo_color = Color(1, 1, 1, 0.82)\nroughness = 0.05\n"),
    # The base textures stay PNG, maps point decals straight at base/wall.png and base/metal.png.
    tex("base/floor", PH, "large_grey_tiles", fmt="png"),
    tex("base/wall", PH, "brick_wall_02", fmt="png"),
    tex("base/metal", PH, "metal_plate", fmt="png", roughness=True, extra="metallic = 0.8\n"),
    # Night district (examples/mcp/night_district.json). One repeat of a facade is six floors of six 3 m bays.
    tex("night/asphalt", PH, "road_damaged"),
    tex("night/sidewalk", PH, "concrete_pavement"),
    tex("night/cracked_concrete", PH, "cracked_concrete"),
    tex("night/dark_brick", PH, "dark_brick_wall"),
    tex("night/corrugated", PH, "corrugated_iron_02", roughness=True, extra="metallic = 0.6\n"),
    tex("night/roller_door", PH, "painted_metal_shutter", roughness=True, extra="metallic = 0.5\n"),
    tex("night/rusty_metal", PH, "green_metal_rust", roughness=True, extra="metallic = 0.4\n"),
    tex("night/apartment_lit", ACG, "Facade020B", size=576, extra="emission_enabled = true\nemission = Color(0, 0, 0, 1)\nemission_energy_multiplier = 2.2\n"),
    tex("night/apartment_dark", ACG, "Facade018A", size=576),
    tex("night/chainlink", MADE, "chainlink", size=64, alpha=True, extra="transparency = 3\ncull_mode = 2\nmetallic = 0.7\n", recipe="chainlink", uses=[(ACG, "Fence003")]),
    tex("night/window_warm", MADE, "window_warm", size=64, normal=False, recipe="window_warm", uses=[(ACG, "Facade020B")], extra="emission_enabled = true\nemission = Color(0, 0, 0, 1)\nemission_energy_multiplier = 2.5\n"),
    tex("night/window_dark", MADE, "window_dark", size=64, normal=False, recipe="window_dark", uses=[(ACG, "Facade018A")]),
    tex("night/shop_sign", MADE, "shop_sign", size=(256, 64), normal=False, recipe="shop_sign", uses=[(PH, "green_metal_rust")], extra="emission_enabled = true\nemission = Color(0, 0, 0, 1)\nemission_energy_multiplier = 3.0\n"),
    tex("night/neon_red", MADE, "neon_red", size=32, normal=False, recipe="neon_red", extra="emission_enabled = true\nemission = Color(0, 0, 0, 1)\nemission_energy_multiplier = 4.0\n"),
    # Withered city (examples/mcp/withered_city.json).
    tex("withered/concrete", PH, "concrete_wall_006"),
    tex("withered/concrete_panels", PH, "concrete_layers_02"),
    tex("withered/slab", PH, "cracked_concrete"),
    tex("withered/plaster", PH, "rough_plaster_broken"),
    tex("withered/mossy_plaster", PH, "worn_mossy_plasterwall"),
    tex("withered/brick", PH, "broken_brick_wall"),
    tex("withered/rust", PH, "rusty_metal_04", roughness=True, extra="metallic = 0.6\n"),
    tex("withered/shutter", PH, "rusty_metal_shutter", roughness=True, extra="metallic = 0.6\n"),
    tex("withered/asphalt", PH, "asphalt_02"),
    tex("withered/pavers", PH, "overgrown_concrete_pavers"),
    tex("withered/rubble", PH, "rubble"),
    tex("withered/gravel", PH, "rocky_gravel"),
    tex("withered/moss", PH, "concrete_moss"),
    tex("withered/tiles", PH, "large_floor_tiles_02"),
    tex("withered/letters", MADE, "letters", size=64, recipe="glow_paint", uses=[(PH, "damaged_concrete_floor_03")],
        extra="emission_enabled = true\nemission = Color(0, 0, 0, 1)\nemission_energy_multiplier = 4.0\n"),
    tex("withered/puddle", MADE, "puddle", size=96, recipe="puddle",
        extra="transparency = 1\nalbedo_color = Color(1, 1, 1, 0.78)\nroughness = 0.02\nmetallic_specular = 1.0\n"),
]


def get(url):
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read()


def cached(key, url):
    path = CACHE / key
    if not path.exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        print(f"  download {url}")
        path.write_bytes(get(url))
    return path.read_bytes()


def ph_info(asset):
    return json.loads(cached(f"ph/{asset}.info.json", f"https://api.polyhaven.com/info/{asset}"))


def ph_maps(asset):
    """albedo, normal (OpenGL), roughness images of a Poly Haven texture at 1k."""
    files = json.loads(cached(f"ph/{asset}.files.json", f"https://api.polyhaven.com/files/{asset}"))
    load = lambda key: Image.open(io.BytesIO(cached(f"ph/{asset}_{key}_1k.jpg", files[key]["1k"]["jpg"]["url"])))
    return {"color": load("Diffuse"), "normal": load("nor_gl"), "roughness": load("Rough")}


def acg_maps(asset):
    """Every map in an ambientCG 1K JPG download, keyed by its suffix in lower case (color, normalgl, opacity...)."""
    data = cached(f"acg/{asset}_1K-JPG.zip", f"https://ambientcg.com/get?file={asset}_1K-JPG.zip")
    maps = {}
    with zipfile.ZipFile(io.BytesIO(data)) as z:
        for n in z.namelist():
            stem = pathlib.PurePath(n).stem
            if not n.lower().endswith((".jpg", ".png")) or "_" not in stem:
                continue
            key = stem.rsplit("_", 1)[1].lower()
            maps[key] = Image.open(io.BytesIO(z.read(n)))
            maps[key].load()
    maps.setdefault("normal", maps.get("normalgl"))
    return maps


def source_maps(source, asset):
    return ph_maps(asset) if source == PH else acg_maps(asset)


def fit(img, mode="RGB", size=RESOLUTION):
    img = img.convert(mode)
    return img if img.size == (size, size) else img.resize((size, size), Image.LANCZOS)


def rotate_normal(img):
    """A quarter turn counter clockwise, with the tangent space vector turned along."""
    turned = np.asarray(img.convert("RGB").transpose(Image.Transpose.ROTATE_90)).astype(np.int16)
    out = turned.copy()
    out[..., 0] = 255 - turned[..., 1]
    out[..., 1] = turned[..., 0]
    return Image.fromarray(out.astype(np.uint8))


def relevel(img, mean, saturation):
    arr = np.asarray(img.convert("RGB")).astype(np.float32)
    lum = arr.mean(axis=-1, keepdims=True)
    arr = lum + (arr - lum) * saturation
    arr *= mean / arr.mean()
    return Image.fromarray(arr.clip(0, 255).astype(np.uint8))


def rng(seed):
    return np.random.default_rng(seed)


def tiling_noise(size, cells, seed, octaves=1):
    """Smooth value noise that wraps, in 0..1."""
    out = np.zeros((size, size))
    amp, total = 1.0, 0.0
    for o in range(octaves):
        c = cells * (2**o)
        grid = rng(seed + o).random((c, c))
        coords = np.arange(size) * c / size
        i0 = np.floor(coords).astype(int)
        t = coords - i0
        t = t * t * (3 - 2 * t)
        i1 = (i0 + 1) % c
        rows = grid[i0][:, i0] * (1 - t)[None, :] + grid[i0][:, i1] * t[None, :]
        rows1 = grid[i1][:, i0] * (1 - t)[None, :] + grid[i1][:, i1] * t[None, :]
        out += amp * (rows * (1 - t)[:, None] + rows1 * t[:, None])
        total += amp
        amp *= 0.5
    return out / total


def normal_from_height(h, strength):
    dx = (np.roll(h, -1, axis=1) - np.roll(h, 1, axis=1)) * strength
    # Rows grow downwards while the green channel points up.
    dy = (np.roll(h, 1, axis=0) - np.roll(h, -1, axis=0)) * strength
    n = np.stack([-dx, -dy, np.ones_like(h)], axis=-1)
    n /= np.linalg.norm(n, axis=-1, keepdims=True)
    return Image.fromarray(((n * 0.5 + 0.5) * 255).round().astype(np.uint8))


def sprites(color, opacity):
    """Cut an atlas into its separate pieces: column bands first, then row bands inside each column."""
    alpha = np.asarray(opacity.convert("L")) > 96
    rgba = np.dstack([np.asarray(color.convert("RGB")), np.asarray(opacity.convert("L"))])

    def bands(mask_1d):
        out, start = [], None
        for i, v in enumerate(list(mask_1d) + [False]):
            if v and start is None:
                start = i
            elif not v and start is not None:
                if i - start > 8:
                    out.append((start, i))
                start = None
        return out

    pieces = []
    for x0, x1 in bands(alpha.any(axis=0)):
        column = alpha[:, x0:x1]
        for y0, y1 in bands(column.any(axis=1)):
            xs = np.where(column[y0:y1].any(axis=0))[0]
            pieces.append(Image.fromarray(rgba[y0:y1, x0 + xs[0] : x0 + xs[-1] + 1].astype(np.uint8), "RGBA"))
    return pieces


def scatter(pieces, count, size_range, seed, shade=(0.55, 1.0), size=RESOLUTION):
    """Drops atlas pieces at random with random turns, wrapping at the edges so the result tiles.
    Pieces laid first end up underneath and are drawn darker, which reads as depth."""
    r = rng(seed)
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    for i in range(count):
        piece = pieces[r.integers(len(pieces))]
        target = r.uniform(*size_range) * size
        scale = target / max(piece.size)
        p = piece.resize((max(1, int(piece.width * scale)), max(1, int(piece.height * scale))), Image.LANCZOS)
        p = p.rotate(r.uniform(0, 360), resample=Image.BICUBIC, expand=True)
        k = shade[0] + (shade[1] - shade[0]) * (i / max(1, count - 1)) * r.uniform(0.85, 1.0)
        rgb = np.asarray(p).astype(np.float32)
        rgb[..., :3] *= k
        p = Image.fromarray(rgb.clip(0, 255).astype(np.uint8), "RGBA")
        x, y = int(r.integers(size)), int(r.integers(size))
        for ox in (-size, 0, size):
            for oy in (-size, 0, size):
                if x + ox + p.width > 0 and x + ox < size and y + oy + p.height > 0 and y + oy < size:
                    paste_clipped(canvas, p, x + ox, y + oy)
    return canvas


def paste_clipped(canvas, p, x, y):
    cx, cy = max(0, -x), max(0, -y)
    w, h = min(p.width, canvas.width - x), min(p.height, canvas.height - y)
    canvas.alpha_composite(p.crop((cx, cy, w, h)), (x + cx, y + cy))


def binarize_alpha(img, threshold=128):
    a = np.asarray(img.getchannel("A"))
    img.putalpha(Image.fromarray(np.where(a >= threshold, 255, 0).astype(np.uint8)))
    return img


def recipe_scatter_leaves(_):
    m = acg_maps("LeafSet024")
    leaves = sprites(m["color"], m["opacity"])
    img = scatter(leaves, 420, (0.07, 0.11), seed=7)
    return {"color": binarize_alpha(img)}


def recipe_scatter_needles(_):
    m = acg_maps("PineNeedles001")
    sprigs = sprites(m["color"], m["opacity"])
    img = scatter(sprigs, 2600, (0.2, 0.32), seed=11, shade=(0.45, 0.95))
    # The scanned needles are dry and reddish, living pine is a dark blue green.
    arr = np.asarray(img).astype(np.float32)
    lum = arr[..., :3].mean(axis=-1, keepdims=True) / 255.0
    arr[..., :3] = lum * np.array([70.0, 120.0, 80.0]) * 1.6
    return {"color": binarize_alpha(Image.fromarray(arr.clip(0, 255).astype(np.uint8), "RGBA"))}


def recipe_iron_bars(_):
    metal = fit(acg_maps("Metal046B")["color"], size=512)
    arr = np.asarray(metal).astype(np.float32)
    s = 512
    x = np.arange(s)
    # Four vertical bars, a rail along the top edge and one across the middle, rounded by a cosine shade.
    bar_w = s // 12
    vert = np.abs(((x + bar_w // 2) % (s // 4)) - bar_w // 2)
    vert_mask = vert < bar_w // 2
    vert_shade = np.cos(np.clip(vert / (bar_w / 2), 0, 1) * math.pi / 2)
    rail_w = s // 16
    horiz = np.minimum(np.abs(x - rail_w // 2), np.abs(x - s // 2))
    horiz_mask = horiz < rail_w // 2
    horiz_shade = np.cos(np.clip(horiz / (rail_w / 2), 0, 1) * math.pi / 2)
    mask = vert_mask[None, :] | horiz_mask[:, None]
    shade = np.maximum(vert_shade[None, :] * vert_mask[None, :], horiz_shade[:, None] * horiz_mask[:, None])
    arr *= (0.45 + 0.75 * shade)[..., None]
    rgba = np.dstack([arr.clip(0, 255), np.where(mask, 255, 0)]).astype(np.uint8)
    return {"color": Image.fromarray(rgba, "RGBA")}


def recipe_chalkboard(_):
    base = fit(ph_maps("painted_plaster_wall")["color"], "L")
    lum = np.asarray(base).astype(np.float32) / 255.0
    lum = (lum - lum.mean()) * 0.35
    s = RESOLUTION
    # Wiped chalk: soft cloudy haze, stronger in horizontal sweeps.
    haze = tiling_noise(s, 4, 3, octaves=4)
    sweeps = tiling_noise(s, 2, 5, octaves=2)
    sweeps = np.repeat(sweeps[:, :1], s, axis=1) * 0.6 + sweeps * 0.4
    chalk = np.clip((haze * 0.6 + sweeps * 0.4 - 0.45) * 1.6, 0, 1) * 0.22
    green = np.array([34.0, 58.0, 47.0])
    white = np.array([205.0, 212.0, 205.0])
    rgb = green * (1.0 + lum[..., None]) * (1 - chalk[..., None]) + white * chalk[..., None]
    return {"color": Image.fromarray(rgb.clip(0, 255).astype(np.uint8))}


def recipe_lamp(_):
    base = fit(ph_maps("white_plaster_02")["color"], "L")
    lum = np.asarray(base).astype(np.float32) / 255.0
    lum = 1.0 + (lum - lum.mean()) * 0.5
    warm = np.array([255.0, 236.0, 176.0])
    return {"color": Image.fromarray((warm * lum[..., None]).clip(0, 255).astype(np.uint8))}


def recipe_glass(_):
    s = 512
    smudge = tiling_noise(s, 3, 21, octaves=5)
    y, x = np.mgrid[0:s, 0:s]
    # Two soft diagonal glints that wrap with the tile.
    streak = np.maximum(0, np.cos((x + y) * 2 * math.pi / s) - 0.9) / 0.1
    streak += np.maximum(0, np.cos((x + y) * 4 * math.pi / s + 1.3) - 0.95) / 0.05 * 0.5
    base = np.array([168.0, 200.0, 212.0])
    rgb = base * (0.94 + smudge[..., None] * 0.1) + np.array([22.0, 20.0, 18.0]) * streak[..., None]
    alpha = 200 + smudge * 55
    return {"color": Image.fromarray(np.dstack([rgb.clip(0, 255), alpha.clip(0, 255)]).astype(np.uint8), "RGBA")}


def recipe_stained_glass(_):
    s = 512
    r = rng(24)
    points = r.random((34, 2)) * s
    palette = np.array([[176, 36, 46], [36, 74, 176], [214, 164, 38], [44, 132, 66], [112, 46, 140], [196, 96, 34]], dtype=np.float32)
    colors = palette[r.integers(len(palette), size=len(points))]
    y, x = np.mgrid[0:s, 0:s].astype(np.float32)
    d1 = np.full((s, s), np.inf)
    d2 = np.full((s, s), np.inf)
    owner = np.zeros((s, s), dtype=int)
    for i, (px, py) in enumerate(points):
        dx = np.abs(x - px)
        dy = np.abs(y - py)
        d = np.sqrt(np.minimum(dx, s - dx) ** 2 + np.minimum(dy, s - dy) ** 2)
        closer = d < d1
        d2 = np.where(closer, d1, np.minimum(d2, d))
        d1 = np.where(closer, d, d1)
        owner = np.where(closer, i, owner)
    glass = tiling_noise(s, 8, 9, octaves=3)
    rgb = colors[owner] * (0.8 + glass[..., None] * 0.4)
    # Lead cames between the panes, with a slight highlight along their middle.
    edge = d2 - d1
    came = edge < 7
    rgb = np.where(came[..., None], np.array([38.0, 38.0, 42.0]) + (7 - edge[..., None]) * 3, rgb)
    return {"color": Image.fromarray(rgb.clip(0, 255).astype(np.uint8))}


def recipe_water(_):
    s = 512
    y, x = np.mgrid[0:s, 0:s].astype(np.float64) / s
    r = rng(28)
    h = np.zeros((s, s))
    # Whole wave numbers keep every wave periodic across the tile.
    for _ in range(24):
        kx, ky = r.integers(-6, 7, size=2)
        if kx == 0 and ky == 0:
            continue
        amp = 1.0 / math.hypot(kx, ky)
        h += amp * np.sin(2 * math.pi * (kx * x + ky * y) + r.uniform(0, 2 * math.pi))
    h += tiling_noise(s, 16, 29, octaves=3) * 1.5
    h = (h - h.min()) / (h.max() - h.min())
    deep = np.array([24.0, 70.0, 112.0])
    light = np.array([78.0, 138.0, 170.0])
    rgb = deep + (light - deep) * (h[..., None] ** 1.5)
    return {"color": Image.fromarray(rgb.clip(0, 255).astype(np.uint8)), "normal": normal_from_height(h, 6.0)}


def recipe_glow_paint(_):
    """Glowing paint rolled over cracked concrete: the paint skips the cracks and has worn thin in patches."""
    src = ph_maps("damaged_concrete_floor_03")
    base = np.asarray(fit(src["color"])).astype(np.float32)
    lum = base.mean(axis=-1) / 255.0
    s = RESOLUTION
    wear = tiling_noise(s, 5, 41, octaves=6)
    chips = np.clip((wear - 0.36) * 14.0, 0, 1)
    cracks = np.clip((lum - lum.mean() * 0.55) * 10.0, 0, 1)
    coverage = chips * cracks
    grain = np.clip(0.8 + 0.5 * (lum / lum.mean() - 1.0), 0.55, 1.1)
    paint = np.array([255.0, 186.0, 92.0])
    albedo = base * 0.5 * (1 - coverage[..., None]) + paint * grain[..., None] * coverage[..., None]
    glow = np.array([255.0, 156.0, 60.0]) * (coverage * grain)[..., None]
    return {
        "color": Image.fromarray(albedo.clip(0, 255).astype(np.uint8)),
        "normal": fit(src["normal"]),
        "emission": Image.fromarray(glow.clip(0, 255).astype(np.uint8)),
    }


def recipe_puddle(_):
    s = 512
    y, x = np.mgrid[0:s, 0:s].astype(np.float64) / s
    r = rng(52)
    h = np.zeros((s, s))
    for _ in range(16):
        kx, ky = r.integers(-4, 5, size=2)
        if kx == 0 and ky == 0:
            continue
        h += np.sin(2 * math.pi * (kx * x + ky * y) + r.uniform(0, 2 * math.pi)) / math.hypot(kx, ky)
    h += tiling_noise(s, 12, 53, octaves=3)
    h = (h - h.min()) / (h.max() - h.min())
    deep = np.array([8.0, 12.0, 16.0])
    light = np.array([38.0, 46.0, 54.0])
    rgb = deep + (light - deep) * h[..., None]
    return {"color": Image.fromarray(rgb.clip(0, 255).astype(np.uint8)), "normal": normal_from_height(h, 2.0)}


def facade_cell(asset, key, col, row):
    """The window of one bay of a six by six ambientCG facade, frame and sill included, cropped from the map named key
    (color, emission), so it fits any wall."""
    img = acg_maps(asset)[key].convert("RGB")
    cell = img.width / 6
    # The bays start between two piers and floors on a ledge, 80 of 1024 pixels in from the corner, and the window
    # takes the middle of its bay.
    x, y = img.width * 80 / 1024 + col * cell, img.height * 80 / 1024 + row * cell
    box = (x + cell * 0.172, y + cell * 0.182, x + cell * 0.862, y + cell * 0.87)
    return img.crop(tuple(round(v) for v in box)).resize((256, 256), Image.LANCZOS)


def recipe_window_warm(_):
    return {"color": facade_cell("Facade020B", "color", 1, 3), "emission": facade_cell("Facade020B", "emission", 1, 3)}


def recipe_window_dark(_):
    return {"color": facade_cell("Facade018A", "color", 1, 3)}


def recipe_shop_sign(_):
    from PIL import ImageDraw, ImageFilter, ImageFont
    w, h = 1024, 256
    plate = relevel(fit(ph_maps("green_metal_rust")["color"]).crop((0, 0, w, h)), 38, 0.4)
    letters = Image.new("L", (w, h), 0)
    draw = ImageDraw.Draw(letters)
    # Pillow's bundled Aileron font (CC0), so the sign needs no system font.
    draw.text((w / 2, 104), "CORNER SHOP", font=ImageFont.load_default(size=118), anchor="mm", fill=255, stroke_width=3, stroke_fill=255)
    draw.text((w / 2, 208), "OPEN 24 HOURS", font=ImageFont.load_default(size=52), anchor="mm", fill=255, stroke_width=1, stroke_fill=255)
    tube = np.asarray(letters).astype(np.float32) / 255.0
    halo = np.asarray(letters.filter(ImageFilter.GaussianBlur(10))).astype(np.float32) / 255.0
    neon = np.array([255.0, 92.0, 70.0])
    rgb = np.asarray(plate).astype(np.float32) * (1 - tube[..., None]) + neon * tube[..., None] + neon * halo[..., None] * 0.35
    glow = neon * np.clip(tube + halo * 0.5, 0, 1)[..., None]
    return {"color": Image.fromarray(rgb.clip(0, 255).astype(np.uint8)), "emission": Image.fromarray(glow.clip(0, 255).astype(np.uint8))}


def recipe_chainlink(_):
    """Fence003 with its wires thickened a little in the cutout mask. Its material uses alpha hash, so the wires fade
    into a dither at a distance where a fixed scissor threshold would cut the averaged mipmaps away entirely."""
    from PIL import ImageFilter
    m = acg_maps("Fence003")
    color = fit(m["color"], "RGBA")
    color.putalpha(fit(m["opacity"], "L").filter(ImageFilter.MaxFilter(3)))
    return {"color": color, "normal": fit(m["normal"])}


def recipe_neon_red(_):
    s = 64
    y = np.abs(np.arange(s) - (s - 1) / 2) / (s / 2)
    core = np.clip(1.2 - y * y * 1.6, 0.35, 1)
    rgb = np.array([255.0, 70.0, 60.0]) * core[:, None, None] * np.ones((s, s, 1))
    img = Image.fromarray(rgb.clip(0, 255).astype(np.uint8))
    return {"color": img, "emission": img}


def world_size(t):
    if isinstance(t["size"], tuple):
        return t["size"]
    if t["size"] is not None:
        return float(t["size"])
    dims = ph_info(t["asset"])["dimensions"]
    return round(dims[0] / 1000.0 * UNITS_PER_METER)


def rel(t, suffix, ext):
    return f"{t['name']}{suffix}.{ext}"


def save(img, path, fmt):
    path.parent.mkdir(parents=True, exist_ok=True)
    if fmt == "jpg":
        img.convert("RGB").save(path, "JPEG", quality=JPEG_QUALITY, optimize=True, progressive=True)
    else:
        img.save(path, "PNG", optimize=True)


# Photos need mipmaps and VRAM compression, which Godot only picks on its own once the editor sees them in 3D, so
# the import settings are written up front. Godot fills in the rest of the file on the next import.
IMPORT_PARAMS = {"compress/mode": "2", "mipmaps/generate": "true", "detect_3d/compress_to": "0"}


def write_import(image_path, normal_map):
    params = dict(IMPORT_PARAMS, **{"compress/normal_map": "1" if normal_map else "0"})
    path = image_path.with_name(image_path.name + ".import")
    if not path.exists():
        path.write_text('[remap]\n\nimporter="texture"\ntype="CompressedTexture2D"\n\n[params]\n\n', newline="\n")
    lines = path.read_text().splitlines()
    for key, value in params.items():
        entry = f"{key}={value}"
        found = [i for i, line in enumerate(lines) if line.startswith(key + "=")]
        if found:
            lines[found[0]] = entry
        else:
            lines.append(entry)
    path.write_text("\n".join(lines) + "\n", newline="\n")


def material(t, files, size):
    lines = [f'[ext_resource type="Texture2D" path="{RES}/{files["color"]}" id="1_albedo"]']
    body = ['albedo_texture = ExtResource("1_albedo")']
    if "normal" in files:
        lines.append(f'[ext_resource type="Texture2D" path="{RES}/{files["normal"]}" id="2_normal"]')
        body += ["normal_enabled = true", 'normal_texture = ExtResource("2_normal")']
    if "roughness" in files:
        lines.append(f'[ext_resource type="Texture2D" path="{RES}/{files["roughness"]}" id="3_roughness"]')
        body.append('roughness_texture = ExtResource("3_roughness")')
    if "emission" in files:
        lines.append(f'[ext_resource type="Texture2D" path="{RES}/{files["emission"]}" id="4_emission"]')
        body.append('emission_texture = ExtResource("4_emission")')
    extra = t["extra"].strip()
    if extra:
        body += extra.splitlines()
    w, h = size if isinstance(size, tuple) else (size, size)
    body.append(f"metadata/texture_size = Vector2({w:g}, {h:g})")
    return f'[gd_resource type="StandardMaterial3D" load_steps={len(lines) + 1} format=3]\n\n' + "\n".join(lines) + "\n\n[resource]\n" + "\n".join(body) + "\n"


def build(t):
    print(t["name"])
    if t["source"] == MADE:
        maps = globals()[f"recipe_{t['recipe']}"](t)
    else:
        src = source_maps(t["source"], t["asset"])
        maps = {"color": fit(src["color"])}
        if t["normal"]:
            maps["normal"] = fit(src["normal"])
        if t["roughness"]:
            maps["roughness"] = fit(src["roughness"], "L")
        if "emission_enabled = true" in t["extra"] and "emission" in src:
            maps["emission"] = fit(src["emission"])
        if t["alpha"] and "opacity" in src:
            maps["color"].putalpha(fit(src["opacity"], "L"))
        if t["level"]:
            maps["color"] = relevel(maps["color"], *t["level"])
        if t["rotate"]:
            maps = {k: (rotate_normal(v) if k == "normal" else v.transpose(Image.Transpose.ROTATE_90)) for k, v in maps.items()}

    size = world_size(t)
    files = {"color": rel(t, "", t["fmt"])}
    save(maps["color"], OUT / files["color"], t["fmt"])
    write_import(OUT / files["color"], False)
    for key, suffix in (("normal", "_normal"), ("roughness", "_roughness"), ("emission", "_emission")):
        if key in maps:
            files[key] = rel(t, suffix, "jpg")
            save(maps[key], OUT / files[key], "jpg")
            write_import(OUT / files[key], key == "normal")
    (OUT / f"{t['name']}.tres").write_text(material(t, files, size), newline="\n")
    return files, size


def credit_link(source, asset):
    if source == PH:
        return f"[{asset}](https://polyhaven.com/a/{asset})"
    return f"[{asset}](https://ambientcg.com/a/{asset})"


def credits(sizes):
    rows = []
    for t in TEXTURES:
        size = sizes.get(t["name"])
        if isinstance(size, tuple):
            size_text = f"{size[0]:g} x {size[1]:g} units ({size[0] / UNITS_PER_METER:g} x {size[1] / UNITS_PER_METER:g} m)"
        else:
            size_text = f"{size:g} units ({size / UNITS_PER_METER:g} m)" if size else ""
        if t["source"] == MADE:
            origin = "drawn by tools/fetch_demo_textures.py" + (", from " + ", ".join(credit_link(s, a) for s, a in t["uses"]) if t["uses"] else "")
            scans = ", ".join("Poly Haven" if s == PH else "ambientCG" for s, _ in t["uses"])
            author = f"GodotTrench, scan by {scans}" if scans else "GodotTrench"
        elif t["source"] == PH:
            origin = "Poly Haven " + credit_link(PH, t["asset"])
            author = ", ".join(ph_info(t["asset"]).get("authors", {}).keys()) or "Poly Haven"
        else:
            origin = "ambientCG " + credit_link(ACG, t["asset"])
            author = "ambientCG (Lennart Demes)"
        rows.append(f"| `{t['name']}` | {origin} | {author} | {size_text} |")
    return (
        "# Demo texture credits\n\n"
        "Every photo texture here comes from [Poly Haven](https://polyhaven.com) or [ambientCG](https://ambientcg.com) and is\n"
        "released under [CC0 1.0](https://creativecommons.org/publicdomain/zero/1.0/). No attribution is required, it is\n"
        "given here anyway. The files are rebuilt by `python tools/fetch_demo_textures.py`, which holds the same table.\n\n"
        "The world size is what one repeat of the texture covers in the map, stored in each material as\n"
        "`metadata/texture_size` and taken from the real size of the scan where the source publishes one.\n\n"
        "| Material | Source | Author | World size |\n|---|---|---|---|\n" + "\n".join(rows) + "\n\n"
        "Glass, stained glass, water and the withered puddle have no fitting scan and are drawn by the script. They and the `special/*` tool\n"
        "textures were made for GodotTrench and fall under its MIT license. The other drawn textures recolour or rearrange\n"
        "the CC0 scans named in their row.\n"
    )


def check():
    missing = []
    for t in TEXTURES:
        tres = OUT / f"{t['name']}.tres"
        if not tres.exists() or "metadata/texture_size" not in tres.read_text():
            missing.append(str(tres))
        if not (OUT / rel(t, "", t["fmt"])).exists():
            missing.append(rel(t, "", t["fmt"]))
    for m in missing:
        print(f"missing or stale: {m}")
    return not missing


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("names", nargs="*", help="short names (planks) or full names (showcase/planks) to rebuild")
    ap.add_argument("--check", action="store_true", help="only verify the files on disk")
    args = ap.parse_args()
    if args.check:
        sys.exit(0 if check() else 1)

    wanted = set(args.names)
    sizes = {}
    for t in TEXTURES:
        if wanted and t["name"] not in wanted and t["name"].split("/")[-1] not in wanted:
            sizes[t["name"]] = world_size(t) if t["source"] != PH or (CACHE / f"ph/{t['asset']}.info.json").exists() else None
            continue
        _, sizes[t["name"]] = build(t)
    (OUT / "CREDITS.md").write_text(credits(sizes), newline="\n")
    total = sum(p.stat().st_size for p in OUT.rglob("*") if p.is_file() and p.suffix in (".jpg", ".png"))
    print(f"textures folder images: {total / 1e6:.1f} MB")


if __name__ == "__main__":
    main()
