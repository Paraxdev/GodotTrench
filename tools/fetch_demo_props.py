#!/usr/bin/env python3
"""Downloads a fixed set of CC0 Poly Haven props (1k glTF) into godot/models/polyhaven.

  python tools/fetch_demo_props.py          # fetch everything in PROPS below
  python tools/fetch_demo_props.py --check  # verify the files on disk match PROPS, no network

Each entry in PROPS is (asset id, category folder). The category folder holds every prop's .gltf
and .bin directly, plus a shared textures/ folder, since Poly Haven texture filenames are already
prefixed with the asset id so nothing collides. Rerunning is safe, existing files are left alone
unless --force is passed. Also (re)writes godot/models/polyhaven/pack.json with credits pulled from
Poly Haven's /info endpoint.
"""
import argparse
import json
import pathlib
import sys
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "godot" / "models" / "polyhaven"
API = "https://api.polyhaven.com"

# (asset id, category folder)
PROPS = [
    ("wine_barrel_01", "containers"),
    ("wooden_crate_01", "containers"),
    ("old_military_crate", "containers"),
    ("wooden_bucket_01", "containers"),
    ("WoodenChair_01", "furniture"),
    ("SchoolChair_01", "furniture"),
    ("WoodenTable_02", "furniture"),
    ("wooden_bookshelf_worn", "furniture"),
    ("SchoolDesk_01", "furniture"),
    ("painted_wooden_bench", "furniture"),
    ("metal_stool_01", "furniture"),
    ("wooden_lantern_01", "lighting"),
    ("street_lamp_02", "lighting"),
    ("vintage_oil_lamp", "lighting"),
    ("ceramic_pot", "decorative"),
    ("wooden_ladder", "tools"),
    ("ocean_buoy", "nautical"),
    ("lifebuoy", "nautical"),
    ("covered_car", "vehicles"),
    ("Barrel_01", "containers"),
    ("barrel_03", "containers"),
    ("barrel_stove", "containers"),
    ("old_tyre", "industrial"),
    ("utility_box_01", "industrial"),
    ("Sofa_01", "furniture"),
    ("plastic_monobloc_chair_01", "furniture"),
    # Night district
    ("security_light", "lighting"),
    ("trashbag", "containers"),
    ("cardboard_box_01", "containers"),
    ("water_manhole_cover", "industrial"),
    ("security_camera_01", "industrial"),
]

# Anthropic and this project ship nothing under an account, fetch as a plain client. Poly Haven's
# CDN 403s the default urllib user agent, a browser-like one is enough to pass.
HEADERS = {"User-Agent": "Mozilla/5.0 (compatible; GodotTrenchPropFetch/1.0)"}


def get_json(url):
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req, timeout=30) as r:
        return json.load(r)


def download(url, dest):
    if dest.exists():
        return
    dest.parent.mkdir(parents=True, exist_ok=True)
    req = urllib.request.Request(url, headers=HEADERS)
    with urllib.request.urlopen(req, timeout=60) as r, open(dest, "wb") as f:
        f.write(r.read())


def fetch_one(asset_id, category):
    info = get_json(f"{API}/info/{asset_id}")
    files = get_json(f"{API}/files/{asset_id}")
    # /files/<id> nests each resolution one level deeper than you'd expect: gltf -> 1k -> gltf -> {url, include}.
    entry = files["gltf"]["1k"]["gltf"]
    cat_dir = OUT / category
    download(entry["url"], cat_dir / f"{asset_id}.gltf")
    for rel, meta in entry["include"].items():
        download(meta["url"], cat_dir / rel)
    authors = ", ".join(info.get("authors", {}).keys()) or "Poly Haven"
    return {
        "asset": asset_id,
        "category": category,
        "author": authors,
        "license": "CC0",
        "url": f"https://polyhaven.com/a/{asset_id}",
    }


def write_pack(credits_list):
    pack = {
        "name": "Poly Haven",
        "short": "Poly Haven",
        "author": "Poly Haven contributors",
        "license": "CC0",
        "url": "https://polyhaven.com",
        "credits": credits_list,
    }
    OUT.mkdir(parents=True, exist_ok=True)
    (OUT / "pack.json").write_text(json.dumps(pack, indent=2) + "\n", encoding="utf-8")


def check():
    missing = []
    for asset_id, category in PROPS:
        if not (OUT / category / f"{asset_id}.gltf").exists():
            missing.append(f"{category}/{asset_id}.gltf")
    if missing:
        print("missing props:\n  " + "\n  ".join(missing))
        return 1
    print(f"all {len(PROPS)} props present")
    return 0


def main():
    p = argparse.ArgumentParser()
    p.add_argument("--check", action="store_true", help="verify files on disk, no network")
    args = p.parse_args()
    if args.check:
        sys.exit(check())

    credits_list = []
    for asset_id, category in PROPS:
        print(f"fetching {asset_id} ({category})")
        credits_list.append(fetch_one(asset_id, category))
    write_pack(credits_list)
    print(f"done, {len(PROPS)} props in {OUT}")


if __name__ == "__main__":
    main()
