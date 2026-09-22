"""MCP client for the Blockbench MCP plugin (streamable HTTP, default http://127.0.0.1:3000/bb-mcp).

Load the plugin once per session through bbcdp.py. Note: loadFromURL(url, true) opens a native trust dialog that
blocks the renderer, passing false skips it:
  python bbcdp.py eval "new Plugin('mcp').loadFromURL('https://jasonjgardner.github.io/blockbench-mcp-plugin/mcp.js', false)"

Usage:
  python bbmcp.py list
  python bbmcp.py call <tool> '<json args>' [--out shot.png]
"""
import argparse
import base64
import json
import sys
import urllib.request

URL = "http://127.0.0.1:3000/bb-mcp"


class Client:
    def __init__(self, url=URL):
        self.url = url
        self.session = None
        self.rid = 0
        self.rpc("initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "gt", "version": "1"}})
        self.notify("notifications/initialized")

    def _post(self, body):
        headers = {"Content-Type": "application/json", "Accept": "application/json, text/event-stream"}
        if self.session:
            headers["mcp-session-id"] = self.session
        req = urllib.request.Request(self.url, data=json.dumps(body).encode(), headers=headers)
        with urllib.request.urlopen(req, timeout=300) as r:
            self.session = r.headers.get("mcp-session-id") or self.session
            text = r.read().decode()
            if r.headers.get("content-type", "").startswith("text/event-stream"):
                data = [line[5:].strip() for line in text.splitlines() if line.startswith("data:")]
                return json.loads(data[-1]) if data else None
            return json.loads(text) if text.strip() else None

    def notify(self, method, params=None):
        self._post({"jsonrpc": "2.0", "method": method, "params": params or {}})

    def rpc(self, method, params=None):
        self.rid += 1
        res = self._post({"jsonrpc": "2.0", "id": self.rid, "method": method, "params": params or {}})
        if "error" in res:
            raise RuntimeError(res["error"])
        return res["result"]

    def call(self, name, args=None):
        return self.rpc("tools/call", {"name": name, "arguments": args or {}})


if __name__ == "__main__":
    p = argparse.ArgumentParser()
    p.add_argument("cmd", choices=["list", "call", "schema"])
    p.add_argument("tool", nargs="?")
    p.add_argument("args", nargs="?", default="{}")
    p.add_argument("--out")
    a = p.parse_args()
    c = Client()
    if a.cmd == "list":
        for t in c.rpc("tools/list")["tools"]:
            print(f"{t['name']}: {t.get('description', '')[:110]}")
    elif a.cmd == "schema":
        t = next(t for t in c.rpc("tools/list")["tools"] if t["name"] == a.tool)
        print(json.dumps(t, indent=1))
    else:
        res = c.call(a.tool, json.loads(a.args))
        for part in res.get("content", []):
            if part["type"] == "image":
                with open(a.out or "shot.png", "wb") as f:
                    f.write(base64.b64decode(part["data"]))
                print("image written to", a.out or "shot.png")
            else:
                print(part.get("text", "")[:4000])
        if res.get("isError"):
            sys.exit(1)
