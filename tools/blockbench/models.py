"""The GodotTrench low poly nature pack, 16 Blockbench units per meter, standing on y = 0."""
import math

from builder import Model, fib_sphere
from tiles import *


def tree_atlas(leaf_pal, seed, bark_tile, needle=False):
    """`needle` switches to the small clumps that read as pine needles."""
    a = Atlas(64, 64)
    a.add("bark", bark_tile, 0, 0)
    side = leaf_clusters(32, 32, leaf_pal, seed, fringe=3, clump_r=(1.5, 2.5) if needle else (2, 4))
    a.add("leaf_side", side, 16, 0)
    a.add("leaf_up", leaf_clusters(32, 32, leaf_pal, seed + 1, light=0.22, clump_r=(1.5, 3) if needle else (2, 4)), 16, 32)
    a.add("leaf_down", leaf_clusters(16, 32, leaf_pal, seed + 2, light=-0.3), 48, 0)
    a.add("rings", rings(16, WOOD, BARK), 48, 32)
    a.add("leaf_small", leaf_clusters(16, 16, leaf_pal, seed + 3, light=0.08, fringe=2, clump_r=(1.5, 2.5)), 48, 48)
    return a


def cluster(m, name, c, w, h, depth=None, yaw=None, tilt=4.0, density=0.8):
    """A rounded leaf clump: a wide flat block crossed by a narrower tall one turned 45 degrees."""
    r = m.rnd
    depth = depth or w * r.uniform(0.85, 1.0)
    yaw = r.uniform(0, 90) if yaw is None else yaw
    rot = (r.uniform(-tilt, tilt), yaw, r.uniform(-tilt, tilt))
    m.box(f"{name}_a", c, [w, h * 0.7, depth], rot, side="leaf_side", up="leaf_up", down="leaf_down", density=density)
    rot = (r.uniform(-tilt, tilt), yaw + 45, r.uniform(-tilt, tilt))
    m.box(f"{name}_b", [c[0], c[1] + h * 0.06, c[2]], [w * 0.72, h, depth * 0.72], rot, side="leaf_side", up="leaf_up", down="leaf_down",
          density=density)


def roots(m, count, r0, r1, y0, thick, region="bark", spread=1.0):
    for k in range(count):
        a = k * math.tau / count + m.rnd.uniform(-0.3, 0.3)
        p0 = [math.cos(a) * r0, y0, math.sin(a) * r0]
        p1 = [math.cos(a) * r1 * spread, -1.5, math.sin(a) * r1 * spread]
        m.beam(f"root{k}", p0, p1, thick * m.rnd.uniform(0.8, 1.1), region)


def pine():
    m = Model("pine", tree_atlas(PINE, 11, bark(16, 64, BARK, 12), needle=True), seed=11)
    m.group = "trunk"
    m.prism("trunk", [(-2, 6.5, 0, 0), (6, 5, 0, 0), (60, 4, 0.5, 0), (120, 3, 1, 0.5), (168, 1.2, 1, 0.5)], 6, "bark")
    roots(m, 4, 3, 11, 9, 4)
    m.group = "needles"
    tiers = 7
    for i in range(tiers):
        t = i / (tiers - 1)
        y = 30 + i * 20
        w = 46 - 34 * t
        h = 14 - 5 * t
        yaw = m.rnd.uniform(0, 90)
        tilt = lambda: m.rnd.uniform(-3, 3)
        kw = dict(side="leaf_side", up="leaf_up", down="leaf_down", density=0.7)
        m.box(f"tier{i}_a", [0, y + h / 2, 0], [2 * w, h, 2 * w], (tilt(), yaw, tilt()), **kw)
        m.box(f"tier{i}_b", [0, y + h / 2 - 3, 0], [2 * w * 0.9, h * 0.6, 2 * w * 0.9], (tilt(), yaw + 45, tilt()), **kw)
        m.box(f"tier{i}_c", [0, y + h + 1.5, 0], [1.2 * w, 4, 1.2 * w], (tilt(), yaw + 22.5, tilt()), **kw)
    m.box("tip", [1, 164, 0.5], [7, 18, 7], (0, 45, 0), side="leaf_small", up="leaf_up", down="leaf_down", density=0.8)
    return m


def oak():
    m = Model("oak", tree_atlas(OAK, 21, bark(16, 64, BARK, 22)), seed=21)
    m.group = "trunk"
    m.prism("trunk", [(-2, 11, 0, 0), (4, 8, 0, 0), (30, 6.5, 0.5, 0), (58, 6, 1.5, 0.5), (72, 4.5, 2, 1)], 7, "bark")
    roots(m, 5, 5, 15, 10, 5)
    crown = [(0, 94, 0, 60, 34), (-30, 80, 8, 38, 26), (28, 82, -10, 40, 28), (6, 78, 30, 36, 24), (-8, 84, -30, 38, 26),
             (-14, 112, 6, 38, 26), (16, 110, -4, 34, 26), (2, 128, 2, 26, 16)]
    for k, (x, y, z, w, h) in enumerate(crown[1:5]):
        m.beam(f"branch{k}", [1 + x * 0.1, 52, z * 0.1], [x * 0.75, y - 4, z * 0.75], 4.5, "bark")
    m.group = "leaves"
    for k, (x, y, z, w, h) in enumerate(crown):
        cluster(m, f"crown{k}", [x, y, z], w, h)
    return m


def birch():
    m = Model("birch", tree_atlas(BIRCH_LEAF, 31, birch_bark(16, 64, 32)), seed=31)
    m.group = "trunk"
    m.prism("trunk", [(-1, 5, 0, 0), (40, 4, 1, 0), (90, 3, -1, 0.5), (132, 2, 0.5, 0)], 6, "bark")
    for k, (p1) in enumerate([[-16, 94, 6], [15, 100, -6], [9, 118, 11]]):
        m.beam(f"branch{k}", [0, p1[1] - 26, 0], p1, 2.2, "bark")
    m.group = "leaves"
    crown = [(0, 104, 0, 30, 34), (-14, 90, 6, 24, 22), (13, 96, -6, 24, 24), (4, 126, 2, 24, 24), (-4, 142, -2, 16, 14),
             (11, 116, 11, 20, 18), (-11, 118, -10, 20, 18), (-6, 80, -10, 18, 14), (8, 82, 10, 18, 14)]
    for k, (x, y, z, w, h) in enumerate(crown):
        cluster(m, f"crown{k}", [x, y, z], w, h, density=0.9)
    return m


def bush_atlas(pal, seed, with_berries=False):
    a = Atlas(64, 64)
    a.add("leaf_side", leaf_clusters(32, 32, pal, seed, fringe=2, clump_r=(1.5, 3)), 0, 0)
    a.add("leaf_up", leaf_clusters(32, 32, pal, seed + 1, light=0.22, clump_r=(1.5, 3)), 32, 0)
    a.add("leaf_down", leaf_clusters(32, 16, pal, seed + 2, light=-0.3), 0, 32)
    a.add("bark", bark(16, 16, BARK, seed + 3), 0, 48)
    if with_berries:
        a.add("berry", berries(4, seed + 4), 32, 32)
    return a


def bush_shape(m, clumps, twigs=3):
    m.group = "twigs"
    for k in range(twigs):
        a = k * math.tau / twigs + m.rnd.uniform(-0.4, 0.4)
        m.beam(f"twig{k}", [0, -1, 0], [math.cos(a) * 7, 8, math.sin(a) * 7], 1.8, "bark")
    m.group = "leaves"
    for k, (x, y, z, w, h) in enumerate(clumps):
        cluster(m, f"clump{k}", [x, y, z], w, h, tilt=6, density=1.0)


def bush():
    m = Model("bush", bush_atlas(BUSH, 41), seed=41)
    bush_shape(m, [(0, 12, 0, 26, 20), (-12, 8, 4, 18, 14), (11, 8, -5, 20, 14), (3, 7, 13, 16, 12), (-3, 19, -3, 16, 10)])
    return m


def bush_berries():
    m = Model("bush_berries", bush_atlas(OAK, 45, with_berries=True), seed=45)
    clumps = [(0, 11, 0, 24, 20), (-11, 8, -6, 18, 14), (10, 8, 6, 18, 14), (-6, 7, 12, 14, 12), (9, 16, -8, 14, 12)]
    bush_shape(m, clumps)
    m.group = "berries"
    r = m.rnd
    for k in range(24):
        x, y, z, w, h = clumps[k % len(clumps)]
        a = r.uniform(0, math.tau)
        rad = w * 0.56
        c = [x + math.cos(a) * rad, y + r.uniform(-h * 0.25, h * 0.35), z + math.sin(a) * rad]
        m.box(f"berry{k}", c, [2.5, 2.5, 2.5], (0, r.uniform(0, 90), 0), faces={f: [32, 32, 34, 34] if k % 2 else [34, 34, 36, 36] for f in
                                                                       ["north", "south", "east", "west", "up", "down"]})
    return m


def bush_round():
    m = Model("bush_round", bush_atlas(BUSH, 43), seed=43)
    m.group = "leaves"
    kw = dict(side="leaf_side", up="leaf_up", down="leaf_down", density=1.0)
    m.box("core", [0, 12, 0], [30, 16, 30], (0, 0, 0), **kw)
    m.box("core45", [0, 12, 0], [30, 16, 30], (0, 45, 0), **kw)
    m.box("tall", [0, 13, 0], [22, 24, 22], (0, 22.5, 0), **kw)
    m.box("tall45", [0, 13, 0], [22, 24, 22], (0, 67.5, 0), **kw)
    m.box("skirt", [0, 4.5, 0], [34, 10, 34], (0, 22.5, 0), **kw)
    m.box("crown", [0, 25, 0], [14, 5, 14], (0, 10, 0), **kw)
    return m


def rock_atlas(pal, seed, mossy=False):
    a = Atlas(64, 64)
    a.add("side", stone(32, 32, pal, seed), 0, 0)
    a.add("top", stone(32, 32, pal, seed + 1, light=0.14), 32, 0)
    a.add("down", stone(32, 32, pal, seed + 2, light=-0.2), 0, 32)
    if mossy:
        a.add("moss", leaf_clusters(32, 32, MOSS, seed + 3, light=0.1, clump_r=(1, 2.2)), 32, 32)
    return a


def rock_regions(n):
    if n[1] > 0.65:
        return "top"
    return "down" if n[1] < -0.35 else "side"


def mossy_regions(n):
    if n[1] > 0.6:
        return "moss"
    return "down" if n[1] < -0.35 else "side"


def rock():
    m = Model("rock", rock_atlas(STONE, 51), seed=51)
    m.hull("rock", fib_sphere(22, m.rnd, (20, 13, 16), lift=1), rock_regions)
    m.hull("chip", [[x + 18, y, z + 10] for x, y, z in fib_sphere(12, m.rnd, (7, 6, 6), lift=0.5)], rock_regions)
    return m


def rock_flat():
    m = Model("rock_flat", rock_atlas(STONE_WARM, 53), seed=53)
    m.hull("slab", fib_sphere(20, m.rnd, (26, 7, 20), jitter=0.1, lift=1.5), rock_regions)
    m.hull("shard", [[x - 14, y + 3, z - 12] for x, y, z in fib_sphere(12, m.rnd, (10, 5, 7), jitter=0.1)], rock_regions)
    return m


def boulder():
    m = Model("boulder", rock_atlas(STONE, 55), seed=55)
    m.hull("boulder", fib_sphere(30, m.rnd, (40, 30, 34), lift=4), rock_regions)
    m.hull("chunk", [[x - 36, y, z + 18] for x, y, z in fib_sphere(16, m.rnd, (16, 13, 14), lift=2)], rock_regions)
    return m


def boulder_mossy():
    m = Model("boulder_mossy", rock_atlas(STONE_WARM, 57, mossy=True), seed=57)
    m.hull("boulder", fib_sphere(28, m.rnd, (36, 28, 32), lift=4), mossy_regions)
    m.hull("chunk", [[x + 30, y, z - 20] for x, y, z in fib_sphere(14, m.rnd, (14, 10, 12), lift=1)], mossy_regions)
    return m


def tuft(m, name, cards, region, width, height, lean=1.5):
    for k in range(cards):
        yaw = k * 180 / cards + m.rnd.uniform(-12, 12)
        off = [m.rnd.uniform(-1.5, 1.5), 0, m.rnd.uniform(-1.5, 1.5)]
        m.card(f"{name}{k}", off, yaw, width * m.rnd.uniform(0.85, 1.1), height * m.rnd.uniform(0.85, 1.1), region,
               lean=m.rnd.uniform(-lean, lean), flip_u=k % 2 == 1)


def grass():
    a = Atlas(64, 32).add("a", blades(32, 32, GRASS, 61), 0, 0).add("b", blades(32, 32, GRASS, 62, count=12, min_h=0.3), 32, 0)
    m = Model("grass", a, seed=61)
    tuft(m, "blades", 3, "a", 16, 11)
    for k in range(2):
        yaw = m.rnd.uniform(0, 180)
        c = [m.rnd.uniform(-6, 6), 0, m.rnd.uniform(-6, 6)]
        m.card(f"short{k}", c, yaw, 10, 7, "b", lean=m.rnd.uniform(-2, 2))
    return m


def grass_tall():
    a = Atlas(64, 64).add("tall", blades(32, 64, GRASS, 63, count=14, min_h=0.5, heads=DRY), 0, 0)
    a.add("short", blades(32, 32, GRASS, 64, count=12, min_h=0.3), 32, 0)
    m = Model("grass_tall", a, seed=63)
    tuft(m, "tall", 4, "tall", 14, 24, lean=3)
    for k in range(3):
        c = [m.rnd.uniform(-5, 5), 0, m.rnd.uniform(-5, 5)]
        m.card(f"short{k}", c, m.rnd.uniform(0, 180), 12, 9, "short", lean=m.rnd.uniform(-2, 2))
    return m


PINKS = [ramp("#7a2d5a", "#b04a86", "#e07ab0", "#f7b2d4"), ramp("#6f6f86", "#b8b8cc", "#ececf2", "#ffffff"),
         ramp("#3a2a6e", "#5d45a8", "#8a73d6", "#b9a8f0")]
YELLOWS = [ramp("#8a5a0e", "#d19a1a", "#f2cc2e", "#fff08a")]


def flower_model(name, petals, center, seed):
    a = Atlas(64, 32).add("flowers", flower_card(32, 32, petals, center, seed), 0, 0)
    a.add("flowers2", flower_card(32, 32, petals[::-1], center, seed + 5, count=3), 32, 0)
    m = Model(name, a, seed=seed)
    tuft(m, "flowers", 2, "flowers", 14, 12)
    for k in range(2):
        c = [m.rnd.uniform(-5, 5), 0, m.rnd.uniform(-5, 5)]
        m.card(f"more{k}", c, m.rnd.uniform(0, 180), 11, 9, "flowers2", lean=m.rnd.uniform(-1.5, 1.5))
    return m


def flowers():
    return flower_model("flowers", PINKS, "#f0c83c", 71)


def flowers_yellow():
    return flower_model("flowers_yellow", YELLOWS, "#c8641e", 73)


def fern():
    a = Atlas(32, 32).add("frond", frond(16, 32, GRASS, 81), 0, 0).add("young", frond(16, 32, OAK, 82), 16, 0)
    m = Model("fern", a, seed=81)
    fronds = 8
    for k in range(fronds):
        yaw = k * math.tau / fronds + m.rnd.uniform(-0.25, 0.25)
        reach = m.rnd.uniform(0.85, 1.1)
        d = [math.cos(yaw), 0, math.sin(yaw)]
        side = [-d[2], 0, d[0]]
        prof = [(0, 0), (4, 7), (10, 11), (16, 11.5), (21, 9)]
        spine = [[d[0] * s * reach, y * reach, d[2] * s * reach] for s, y in prof]
        m.strip(f"frond{k}", spine, [3, 6, 7, 6, 5], side, "frond", flip_u=k % 2 == 1)
    for k in range(3):
        yaw = k * math.tau / 3 + 0.5
        d = [math.cos(yaw), 0, math.sin(yaw)]
        side = [-d[2], 0, d[0]]
        spine = [[d[0] * s, y, d[2] * s] for s, y in [(0, 0), (2, 7), (5, 13), (8, 16)]]
        m.strip(f"young{k}", spine, [3, 5, 5, 4], side, "young")
    return m


def wood_atlas(seed):
    a = Atlas(64, 32)
    a.add("bark", bark(32, 32, BARK_DARK, seed), 0, 0)
    a.add("rings", rings(16, WOOD, BARK_DARK), 32, 0)
    a.add("fungus_top", cap(16, 8, ramp("#5a3a1e", "#7e5530", "#a67442", "#c99a5e"), None, seed + 1), 48, 0)
    a.add("fungus_side", solid(16, 4, ramp("#3e2814", "#5a3a1e", "#7e5530", "#e6d2a8"), seed + 2, 0.9, 0.2), 48, 8)
    a.add("moss", leaf_clusters(32, 16, MOSS, seed + 3, light=0.08, clump_r=(1, 2), fringe=2), 32, 16)
    return a


def stump():
    m = Model("stump", wood_atlas(91), seed=91)
    m.group = "wood"
    m.prism("stump", [(-2, 11, 0, 0), (3, 9.5, 0, 0), (13, 9, 0.3, 0)], 8, "bark", cap="rings", twist=0.2)
    roots(m, 5, 6, 16, 6, 5)
    m.group = "fungus"
    for k, (y, a) in enumerate([(6, 0.4), (9, 0.9), (4, 3.6)]):
        c = [math.cos(a) * 10, y, math.sin(a) * 10]
        m.box(f"shelf{k}", c, [7 - k, 1.6, 7 - k], (0, -math.degrees(a), 0), side="fungus_side", up="fungus_top", down="fungus_side", density=1.5)
    m.box("moss", [-2, 13.2, 3], [9, 0.8, 7], (0, 25, 0), side="moss", up="moss", down="moss", density=1.5)
    return m


def log():
    m = Model("log", wood_atlas(93), seed=93)
    m.group = "wood"
    m.prism("log", [(-32, 7.5, 0, 0), (0, 7, 0, 0.3), (32, 6.5, 0, 0)], 8, "bark", cap="rings", twist=0.2)
    el = m.elements[-1]
    # Lay it down: rotate -90 degrees about Z, so the trunk runs along +X, and rest it on the ground.
    el["vertices"] = [[round(y, 3), round(-x + 6.5, 3), round(z, 3)] for x, y, z in el["vertices"]]
    # The prism's base is open, close it with rings too.
    verts = el["vertices"]
    base = len(verts)
    verts.append([-32.0, 6.5, 0.0])
    cu0, cv0, cu1, cv1 = m.atlas["rings"]
    for i in range(8):
        a, b = i, (i + 1) % 8
        uv = lambda p: [round(cu0 + (cu1 - cu0) * (0.5 + p[2] / 15), 3), round(cv0 + (cv1 - cv0) * (0.5 - (p[1] - 6.5) / 15), 3)]
        el["faces"].append({"v": [base, a, b], "uv": {str(base): uv(verts[base]), str(a): uv(verts[a]), str(b): uv(verts[b])}})
    m.beam("stub", [6, 12, 2], [10, 20, 8], 3, "bark", end="rings")
    m.group = "moss"
    m.box("moss", [-10, 13.6, 0], [22, 1.2, 8], (0, 3, 4), side="moss", up="moss", down="moss", density=1.2)
    m.box("moss2", [14, 13.0, -1], [10, 1.0, 6], (0, -8, -3), side="moss", up="moss", down="moss", density=1.2)
    return m


def mushrooms():
    a = Atlas(32, 32)
    red = ramp("#5a1010", "#8e1c18", "#c22e22", "#e24a32")
    brown = ramp("#4a3020", "#6e4a2e", "#93673e", "#b88a55")
    a.add("red_top", cap(16, 8, red, (240, 236, 228), 101), 0, 0)
    a.add("red_side", solid(16, 4, red, 102, 0.2, 0.8), 0, 8)
    a.add("gills", solid(16, 4, ramp("#9c8a70", "#c2b08e", "#ddd0b0"), 103, 0.2, 0.8), 0, 12)
    a.add("brown_top", cap(16, 8, brown, None, 104), 0, 16)
    a.add("brown_side", solid(16, 4, brown, 105, 0.2, 0.8), 0, 24)
    a.add("stem", solid(8, 16, ramp("#a89a84", "#cfc4ae", "#ebe3d2", "#f7f2e6"), 106, 0.3, 0.9), 16, 0)
    m = Model("mushrooms", a, seed=101)
    for k, (x, z, h, w, kind) in enumerate([(0, 0, 7, 7, "red"), (5, 3, 4.5, 5, "red"), (-4, 4, 3.5, 4.5, "brown"), (-2, -5, 2.5, 3.5, "brown")]):
        tilt = (m.rnd.uniform(-8, 8), 0, m.rnd.uniform(-8, 8))
        s = w * 0.3
        m.cube(f"stem{k}", [x - s / 2, -0.5, z - s / 2], [x + s / 2, h, z + s / 2], tilt, origin=[x, 0, z], side="stem", up="stem", down="stem",
               density=2)
        yaw = m.rnd.uniform(0, 90)
        m.cube(f"cap{k}", [x - w / 2, h - 0.5, z - w / 2], [x + w / 2, h + 1.5, z + w / 2], (tilt[0], yaw, tilt[2]), origin=[x, 0, z],
               side=f"{kind}_side", up=f"{kind}_top", down="gills", density=2)
        m.cube(f"dome{k}", [x - w * 0.33, h + 1.5, z - w * 0.33], [x + w * 0.33, h + 2.6, z + w * 0.33], (tilt[0], yaw + 45, tilt[2]),
               origin=[x, 0, z], side=f"{kind}_side", up=f"{kind}_top", down="gills", density=2)
    return m


ALL = [pine, oak, birch, bush, bush_berries, bush_round, fern, rock, rock_flat, boulder, boulder_mossy, grass, grass_tall, flowers,
       flowers_yellow, stump, log, mushrooms]
