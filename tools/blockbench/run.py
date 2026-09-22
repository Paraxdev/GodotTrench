"""Builds the nature pack inside a running Blockbench and saves what Blockbench's project codec writes.

Start Blockbench with the DevTools port first (see bbcdp.py), then:
  python tools/blockbench/run.py [names...] [--shots] [--out DIR]

Models land in crates/gt_formats/assets/nature, previews and a contact sheet in the temp folder.
"""
import json
import os
import sys
import tempfile

import bbcdp
import models

HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.normpath(os.path.join(HERE, "..", "..", "crates", "gt_formats", "assets", "nature"))
SHOTS = os.path.join(tempfile.gettempdir(), "gt_bbshots")


def build(ws, fn, out):
    m = fn()
    js = open(os.path.join(HERE, "build.js"), encoding="utf-8").read().replace("__SPEC__", json.dumps(m.spec()))
    text = bbcdp.evaluate(ws, js)
    json.loads(text)
    os.makedirs(out, exist_ok=True)
    with open(os.path.join(out, f"{m.name}.bbmodel"), "w", encoding="utf-8", newline="\n") as f:
        f.write(text)
    return m


def shot(ws, name):
    """Frames the model in Blockbench's viewport and captures just the canvas through CDP."""
    import base64
    rect = bbcdp.evaluate(ws, """(() => {
        const b = new THREE.Box3(); Outliner.elements.forEach(e => e.mesh && b.expandByObject(e.mesh));
        const s = new THREE.Vector3(), c = new THREE.Vector3(); b.getSize(s); b.getCenter(c);
        const d = Math.max(s.x, s.y, s.z) * 1.35 + 6;
        const p = Preview.selected;
        p.camera.position.set(c.x + d * 0.75, c.y + d * 0.45, c.z + d * 0.85);
        p.controls.target.set(c.x, c.y, c.z); p.controls.update();
        const r = p.canvas.getBoundingClientRect();
        return [r.x, r.y, r.width, r.height];
    })()""")
    import time
    time.sleep(0.4)
    x, y, w, h = rect
    side = min(w, h)
    clip = {"x": x + (w - side) / 2, "y": y + (h - side) / 2, "width": side, "height": side, "scale": 0.5}
    data = bbcdp.send(ws, "Page.captureScreenshot", {"format": "png", "clip": clip})["data"]
    os.makedirs(SHOTS, exist_ok=True)
    with open(os.path.join(SHOTS, f"{name}.png"), "wb") as f:
        f.write(base64.b64decode(data))


def contact_sheet(names, path):
    from PIL import Image, ImageDraw
    ims = [Image.open(os.path.join(SHOTS, f"{n}.png")).convert("RGB") for n in names]
    s = 300
    cols = 6
    sheet = Image.new("RGB", (cols * s, ((len(ims) + cols - 1) // cols) * s), (30, 30, 34))
    for i, (n, im) in enumerate(zip(names, ims)):
        im = im.resize((s, s))
        ImageDraw.Draw(im).text((6, 4), n, fill=(255, 255, 255))
        sheet.paste(im, ((i % cols) * s, (i // cols) * s))
    sheet.save(path)


if __name__ == "__main__":
    args = sys.argv[1:]
    out = OUT
    if "--out" in args:
        out = args[args.index("--out") + 1]
        args.remove(out)
    names = [a for a in args if not a.startswith("--")]
    ws = bbcdp.connect()
    built = []
    for fn in models.ALL:
        if names and fn.__name__ not in names:
            continue
        m = build(ws, fn, out)
        print("built", m.name, len(m.elements), "elements")
        built.append(m.name)
        if "--shots" in sys.argv:
            shot(ws, m.name)
    if "--shots" in sys.argv:
        contact_sheet(built, os.path.join(SHOTS, "contact.png"))
        print("previews in", SHOTS)
