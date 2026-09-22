"""Model spec helpers. A spec is plain data (cubes, meshes, one texture atlas) that build.js turns into a
Blockbench project through the Blockbench API, so the saved .bbmodel comes from Blockbench's own codec."""
import base64
import io
import itertools
import math
import random

FACES = ["north", "south", "east", "west", "up", "down"]


def v_add(a, b):
    return [a[0] + b[0], a[1] + b[1], a[2] + b[2]]


def v_sub(a, b):
    return [a[0] - b[0], a[1] - b[1], a[2] - b[2]]


def v_mul(a, k):
    return [a[0] * k, a[1] * k, a[2] * k]


def dot(a, b):
    return a[0] * b[0] + a[1] * b[1] + a[2] * b[2]


def cross(a, b):
    return [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]


def norm(a):
    l = math.sqrt(dot(a, a)) or 1.0
    return [a[0] / l, a[1] / l, a[2] / l]


def newell(pts):
    n = [0.0, 0.0, 0.0]
    for i, p in enumerate(pts):
        q = pts[(i + 1) % len(pts)]
        n[0] += (p[1] - q[1]) * (p[2] + q[2])
        n[1] += (p[2] - q[2]) * (p[0] + q[0])
        n[2] += (p[0] - q[0]) * (p[1] + q[1])
    return n


def r2(v):
    return [round(x, 3) for x in v]


class Model:
    def __init__(self, name, atlas, seed=1):
        self.name = name
        self.atlas = atlas
        self.elements = []
        self.rnd = random.Random(seed)
        self.group = "root"

    # Cubes -----------------------------------------------------------------------------------------------------------

    def crop(self, region, fw, fh, density, bottom=False, flip=None):
        """A sub rectangle of `region` matching a face of fw x fh units at `density` pixels per unit."""
        u0, v0, u1, v1 = region
        rw, rh = u1 - u0, v1 - v0
        pw = max(2, min(rw, round(fw * density)))
        ph = max(2, min(rh, round(fh * density)))
        u = u0 + self.rnd.randint(0, rw - pw)
        v = v1 - ph if bottom else v0 + self.rnd.randint(0, rh - ph)
        if flip is None:
            flip = self.rnd.random() < 0.5
        return [u + pw, v, u, v + ph] if flip else [u, v, u + pw, v + ph]

    def cube(self, name, frm, to, rotation=(0, 0, 0), origin=None, side=None, up=None, down=None, density=0.8, faces=None,
             inflate=0.0, side_bottom=True, skip=()):
        """`side`, `up`, `down` are atlas region names, cropped to keep pixels roughly square."""
        size = [abs(to[i] - frm[i]) for i in range(3)]
        dims = {"north": (size[0], size[1]), "south": (size[0], size[1]), "east": (size[2], size[1]), "west": (size[2], size[1]),
                "up": (size[0], size[2]), "down": (size[0], size[2])}
        out = {}
        for f in FACES:
            if f in skip:
                continue
            if faces and f in faces:
                out[f] = {"uv": faces[f]}
                continue
            reg = {"up": up, "down": down}.get(f, side)
            if reg is None:
                continue
            fw, fh = dims[f]
            out[f] = {"uv": self.crop(self.atlas[reg], fw, fh, density, bottom=side_bottom and f not in ("up", "down"))}
        if origin is None:
            origin = [(frm[i] + to[i]) / 2 for i in range(3)]
        self.elements.append({"type": "cube", "name": name, "group": self.group, "from": r2(frm), "to": r2(to), "origin": r2(origin),
                              "rotation": r2(rotation), "inflate": inflate, "faces": out})

    def box(self, name, center, size, rotation=(0, 0, 0), **kw):
        h = [s / 2 for s in size]
        self.cube(name, v_sub(center, h), v_add(center, h), rotation, origin=center, **kw)

    def beam(self, name, p0, p1, thick, region, density=1.0, end=None):
        """A square beam from p0 to p1 (a branch or root), built as a +Y cube tilted into place."""
        d = v_sub(p1, p0)
        length = math.sqrt(dot(d, d))
        dn = norm(d)
        a = math.degrees(math.acos(max(-1, min(1, dn[1]))))
        b = math.degrees(math.atan2(dn[0], dn[2]))
        t = thick / 2
        self.cube(name, [p0[0] - t, p0[1], p0[2] - t], [p0[0] + t, p0[1] + length, p0[2] + t], (a, b, 0), origin=p0, side=region,
                  up=end or region, down=end or region, density=density, side_bottom=False)

    # Meshes ----------------------------------------------------------------------------------------------------------

    def mesh(self, name, verts, faces):
        """faces: (vertex indices, {index: [u, v]}) wound counter clockwise seen from outside."""
        self.elements.append({"type": "mesh", "name": name, "group": self.group, "vertices": [r2(v) for v in verts],
                              "faces": [{"v": list(f), "uv": {str(k): [round(x, 3) for x in uv[k]] for k in f}} for f, uv in faces]})

    def prism(self, name, rings, sides, region, cap=None, twist=0.0, jitter=0.0):
        """Tapered trunk: rings are (y, radius, dx, dz). Bark wraps around once, stretched along the height."""
        u0, v0, u1, v1 = self.atlas[region]
        verts, faces = [], []
        total = rings[-1][0] - rings[0][0]
        for y, r, dx, dz in rings:
            for i in range(sides):
                a = twist + i * math.tau / sides
                rr = r * (1 + self.rnd.uniform(-jitter, jitter))
                verts.append([dx + math.cos(a) * rr, y, dz + math.sin(a) * rr])
        for k in range(len(rings) - 1):
            ta = (rings[k][0] - rings[0][0]) / total
            tb = (rings[k + 1][0] - rings[0][0]) / total
            va, vb = v1 - ta * (v1 - v0), v1 - tb * (v1 - v0)
            for i in range(sides):
                j = (i + 1) % sides
                a, b = k * sides + i, k * sides + j
                c, d = (k + 1) * sides + j, (k + 1) * sides + i
                ua = u0 + (u1 - u0) * i / sides
                ub = u0 + (u1 - u0) * (i + 1) / sides
                # (a, d, c, b) winds counter clockwise seen from outside the trunk.
                faces.append(((a, d, c, b), {a: [ua, va], b: [ub, va], c: [ub, vb], d: [ua, vb]}))
        if cap:
            cu0, cv0, cu1, cv1 = self.atlas[cap]
            top = len(rings) - 1
            y, r, dx, dz = rings[top]
            center = len(verts)
            verts.append([dx, y, dz])
            cuv = lambda p: [cu0 + (cu1 - cu0) * (0.5 + (p[0] - dx) / (2 * r)), cv0 + (cv1 - cv0) * (0.5 + (p[2] - dz) / (2 * r))]
            for i in range(sides):
                a, b = top * sides + i, top * sides + (i + 1) % sides
                faces.append(((center, b, a), {center: cuv(verts[center]), a: cuv(verts[a]), b: cuv(verts[b])}))
        self.mesh(name, verts, faces)

    def strip(self, name, spine, widths, side_dir, region, flip_u=False):
        """A double sided ribbon through `spine` points (a leaf, frond or grass card), texture v runs root to tip."""
        u0, v0, u1, v1 = self.atlas[region]
        if flip_u:
            u0, u1 = u1, u0
        verts, faces = [], []
        lengths = [0.0]
        for a, b in zip(spine, spine[1:]):
            lengths.append(lengths[-1] + math.dist(a, b))
        for p, w in zip(spine, widths):
            verts.append(v_sub(p, v_mul(side_dir, w / 2)))
            verts.append(v_add(p, v_mul(side_dir, w / 2)))
        for k in range(len(spine) - 1):
            a, b, c, d = 2 * k, 2 * k + 1, 2 * k + 3, 2 * k + 2
            ta, tb = lengths[k] / lengths[-1], lengths[k + 1] / lengths[-1]
            uv = {a: [u0, v1 - ta * (v1 - v0)], b: [u1, v1 - ta * (v1 - v0)], c: [u1, v1 - tb * (v1 - v0)], d: [u0, v1 - tb * (v1 - v0)]}
            quad = [a, b, c, d]
            faces.append((quad, uv))
            faces.append((quad[::-1], uv))
        self.mesh(name, verts, faces)

    def card(self, name, center, yaw, width, height, region, lean=0.0, flip_u=False):
        """Upright double sided card, rotated `yaw` degrees about Y, leaning `lean` units at the top."""
        a = math.radians(yaw)
        side = [math.cos(a), 0, -math.sin(a)]
        normal = [math.sin(a), 0, math.cos(a)]
        top = v_add(v_add(center, [0, height, 0]), v_mul(normal, lean))
        self.strip(name, [center, top], [width, width], side, region, flip_u)

    def hull(self, name, points, pick_region, density=0.9):
        """Convex hull of `points`, every triangle textured by a planar crop of the region `pick_region(normal)` returns."""
        tris = convex_hull(points)
        used = sorted({i for t in tris for i in t})
        remap = {old: new for new, old in enumerate(used)}
        verts = [points[i] for i in used]
        faces = []
        for t in tris:
            p = [points[i] for i in t]
            n = norm(newell(p))
            region = self.atlas[pick_region(n)]
            ref = [0, 1, 0] if abs(n[1]) < 0.9 else [0, 0, -1]
            u_axis = norm(cross(ref, n))
            v_axis = norm(cross(n, u_axis))
            # Texture v points down the slope.
            if v_axis[1] > 0:
                v_axis = v_mul(v_axis, -1)
                u_axis = v_mul(u_axis, -1)
            flat = [(dot(q, u_axis) * density, dot(q, v_axis) * density) for q in p]
            minu, minv = min(f[0] for f in flat), min(f[1] for f in flat)
            w = max(f[0] for f in flat) - minu
            h = max(f[1] for f in flat) - minv
            rw, rh = region[2] - region[0], region[3] - region[1]
            k = min(1.0, (rw - 0.01) / max(w, 1e-6), (rh - 0.01) / max(h, 1e-6))
            ou = region[0] + self.rnd.uniform(0, max(0.0, rw - w * k))
            ov = region[1] + self.rnd.uniform(0, max(0.0, rh - h * k))
            idx = [remap[i] for i in t]
            uv = {remap[i]: [ou + (f[0] - minu) * k, ov + (f[1] - minv) * k] for i, f in zip(t, flat)}
            faces.append((idx, uv))
        self.mesh(name, verts, faces)

    def spec(self):
        buf = io.BytesIO()
        self.atlas.img.save(buf, "PNG")
        return {"name": self.name, "tw": self.atlas.img.size[0], "th": self.atlas.img.size[1],
                "png": "data:image/png;base64," + base64.b64encode(buf.getvalue()).decode(), "elements": self.elements}


def convex_hull(points):
    """Brute force hull for a few dozen points in general position: triangles wound outward."""
    n = len(points)
    c = [sum(p[i] for p in points) / n for i in range(3)]
    tris = []
    for i, j, k in itertools.combinations(range(n), 3):
        a, b, d = points[i], points[j], points[k]
        nn = cross(v_sub(b, a), v_sub(d, a))
        if dot(nn, nn) < 1e-9:
            continue
        side = [dot(nn, v_sub(points[m], a)) for m in range(n) if m not in (i, j, k)]
        if all(s <= 1e-7 for s in side):
            tris.append((i, j, k))
        elif all(s >= -1e-7 for s in side):
            tris.append((i, k, j))
    return tris


def fib_sphere(n, rnd, radii, jitter=0.12, squash_bottom=0.35, lift=0.0):
    pts = []
    golden = math.pi * (3 - math.sqrt(5))
    for i in range(n):
        y = 1 - (i + 0.5) / n * 2
        r = math.sqrt(1 - y * y)
        a = i * golden + rnd.uniform(-0.3, 0.3)
        p = [math.cos(a) * r, y, math.sin(a) * r]
        s = 1 + rnd.uniform(-jitter, jitter)
        q = [p[0] * radii[0] * s, p[1] * radii[1] * s, p[2] * radii[2] * s]
        if q[1] < 0:
            q[1] *= squash_bottom
        q[1] += lift
        pts.append(q)
    return pts
