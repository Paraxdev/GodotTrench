#!/usr/bin/env python3
"""Minimal MCP client for a running GodotTrench editor (started with --mcp-http).

Usage:
  python tools/mcp_client.py list
  python tools/mcp_client.py call get_state
  python tools/mcp_client.py call create_brush '{"min":[0,0,0],"max":[64,64,64]}'
  python tools/mcp_client.py call screenshot '{"target":"3d"}' --out shot.png
"""
import argparse
import base64
import json
import sys
import urllib.request

URL = "http://127.0.0.1:7841/mcp"


def rpc(method, params=None, url=URL, rid=1):
    body = json.dumps({"jsonrpc": "2.0", "id": rid, "method": method, "params": params or {}}).encode()
    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json", "Accept": "application/json, text/event-stream"})
    with urllib.request.urlopen(req, timeout=180) as r:
        return json.loads(r.read())


def call(name, arguments=None, url=URL):
    res = rpc("tools/call", {"name": name, "arguments": arguments or {}}, url)
    if "error" in res:
        raise RuntimeError(res["error"])
    return res["result"]


def main():
    p = argparse.ArgumentParser()
    p.add_argument("cmd", choices=["list", "call"])
    p.add_argument("tool", nargs="?")
    p.add_argument("args", nargs="?", default="{}")
    p.add_argument("--out", help="where to write image results")
    p.add_argument("--url", default=URL)
    a = p.parse_args()
    rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "cli", "version": "1"}}, a.url)
    if a.cmd == "list":
        for t in rpc("tools/list", url=a.url)["result"]["tools"]:
            print(f"{t['name']}: {t['description']}")
        return
    result = call(a.tool, json.loads(a.args), a.url)
    for c in result.get("content", []):
        if c["type"] == "image":
            out = a.out or "screenshot.png"
            with open(out, "wb") as f:
                f.write(base64.b64decode(c["data"]))
            print(f"image written to {out}")
        else:
            print(c.get("text", ""))
    if result.get("isError"):
        sys.exit(1)


if __name__ == "__main__":
    main()
