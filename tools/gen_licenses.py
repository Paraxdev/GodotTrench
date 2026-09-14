#!/usr/bin/env python3
"""Regenerates the Rust crate table in LICENSES.md from `cargo metadata`.

  python tools/gen_licenses.py          # rewrite the table
  python tools/gen_licenses.py --check  # exit 1 when the table is out of date
"""
import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
FILE = ROOT / "LICENSES.md"
BEGIN = "<!-- crates:begin -->"
END = "<!-- crates:end -->"


def crate_table():
    meta = json.loads(subprocess.run(["cargo", "metadata", "--format-version", "1", "--locked"], cwd=ROOT, capture_output=True, check=True, text=True).stdout)
    members = set(meta["workspace_members"])
    packages = {p["id"]: p for p in meta["packages"]}
    nodes = {n["id"]: n for n in meta["resolve"]["nodes"]}

    def walk(starts, include_dev):
        seen, stack = set(), list(starts)
        while stack:
            node = nodes[stack.pop()]
            for dep in node["deps"]:
                kinds = {k["kind"] for k in dep["dep_kinds"]}
                allowed = kinds - {"dev"} if not (include_dev and node["id"] in members) else kinds
                if allowed and dep["pkg"] not in seen:
                    seen.add(dep["pkg"])
                    stack.append(dep["pkg"])
        return seen

    shipped = walk(members, include_dev=False)
    everything = walk(members, include_dev=True)
    rows = []
    for pid in everything - members:
        p = packages[pid]
        use = "editor" if pid in shipped else "tests"
        link = p.get("repository") or f"https://crates.io/crates/{p['name']}"
        rows.append((p["name"].lower(), f"| [{p['name']}]({link}) | {p['version']} | {p['license'] or 'see crate'} | {use} |"))
    lines = ["| Crate | Version | License | Used by |", "|-------|---------|---------|---------|"]
    lines += [r for _, r in sorted(rows)]
    return f"{BEGIN}\n" + "\n".join(lines) + f"\n{END}"


def main():
    text = FILE.read_text(encoding="utf-8")
    start, end = text.index(BEGIN), text.index(END) + len(END)
    new = text[:start] + crate_table() + text[end:]
    if "--check" in sys.argv:
        if new != text:
            print("LICENSES.md is out of date, run python tools/gen_licenses.py")
            sys.exit(1)
        return
    FILE.write_text(new, encoding="utf-8", newline="\n")
    print(f"updated {FILE.name}")


if __name__ == "__main__":
    main()
