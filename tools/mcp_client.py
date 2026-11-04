#!/usr/bin/env python3
"""Minimal MCP client for a running TrenchDot editor (started with --mcp-http).

When the editor asks for a token, pass it with --token or TRENCHDOT_MCP_TOKEN, File > Preferences shows it.

Usage:
  python tools/mcp_client.py list
  python tools/mcp_client.py call get_state
  python tools/mcp_client.py call create_brush '{"min":[0,0,0],"max":[64,64,64]}'
  python tools/mcp_client.py call screenshot '{"target":"3d"}' --out shot.png
"""
import argparse
import base64
import json
import os
import sys
import urllib.error
import urllib.request


def env(name, default=""):
    """TRENCHDOT_<name>, or the variable's old GODOTTRENCH_ spelling."""
    return os.environ.get("TRENCHDOT_" + name) or os.environ.get("GODOTTRENCH_" + name) or default

# TRENCHDOT_MCP_URL overrides this, useful when the editor was started with --mcp-http on another port.
URL = env("MCP_URL", "http://127.0.0.1:7841/mcp")
TOKEN = env("MCP_TOKEN")


def rpc(method, params=None, url=URL, rid=1, token=TOKEN):
    body = json.dumps({"jsonrpc": "2.0", "id": rid, "method": method, "params": params or {}}).encode()
    headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream"}
    if token:
        headers["Authorization"] = f"Bearer {token}"
    req = urllib.request.Request(url, data=body, headers=headers)
    # The editor gives up on ordinary calls itself after 120 s, scripts may run for as long as they need.
    with urllib.request.urlopen(req, timeout=3600) as r:
        return json.loads(r.read())


def call(name, arguments=None, url=URL, token=TOKEN):
    res = rpc("tools/call", {"name": name, "arguments": arguments or {}}, url, token=token)
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
    p.add_argument("--token", default=TOKEN, help="the editor's MCP token when it asks for one, defaults to TRENCHDOT_MCP_TOKEN")
    a = p.parse_args()
    try:
        rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "cli", "version": "1"}}, a.url, token=a.token)
    except urllib.error.HTTPError as e:
        if e.code == 401:
            sys.exit("the editor asks for its MCP token, pass the one File > Preferences shows with --token <token> or TRENCHDOT_MCP_TOKEN")
        raise
    if a.cmd == "list":
        for t in rpc("tools/list", url=a.url, token=a.token)["result"]["tools"]:
            print(f"{t['name']}: {t['description']}")
        return
    result = call(a.tool, json.loads(a.args), a.url, a.token)
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
