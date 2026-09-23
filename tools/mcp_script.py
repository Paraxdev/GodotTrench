#!/usr/bin/env python3
"""Replays GodotTrench MCP scripts against a running editor (godottrench --mcp-http=PORT).

  python tools/mcp_script.py run examples/mcp/mountain_house.json
  python tools/mcp_script.py shot out.png --pos 2700,1150,900 --look 1640,960,0 [--shade lit]
  python tools/mcp_script.py call get_state
  python tools/mcp_script.py call scatter --json args.json

Scripts run inside the editor through the run_script tool, so every step is an ordinary MCP tool call.
"""
import argparse
import base64
import json
import os
import sys
import urllib.request


TIMEOUT = 3600
# GODOTTRENCH_MCP_URL overrides this, useful when the editor was started with --mcp-http on another port.
DEFAULT_URL = os.environ.get("GODOTTRENCH_MCP_URL", "http://127.0.0.1:7841/mcp")


def rpc(url, method, params=None):
    body = json.dumps({"jsonrpc": "2.0", "id": 1, "method": method, "params": params or {}}).encode()
    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"})
    with urllib.request.urlopen(req, timeout=TIMEOUT) as r:
        return json.loads(r.read())


def call(url, name, arguments=None):
    res = rpc(url, "tools/call", {"name": name, "arguments": arguments or {}})
    if "error" in res:
        raise RuntimeError(res["error"])
    return res["result"]


def text_of(result):
    return "\n".join(c.get("text", "") for c in result.get("content", []) if c.get("type") == "text")


def parse_summary(result):
    """The run_script summary, or None when the editor answered with plain error text such as a timeout."""
    if isinstance(result.get("structuredContent"), dict):
        return result["structuredContent"]
    try:
        summary = json.loads(text_of(result) or "{}")
    except json.JSONDecodeError:
        return None
    return summary if isinstance(summary, dict) else None


def vec(text):
    return [float(v) for v in text.split(",")]


def main():
    global TIMEOUT
    p = argparse.ArgumentParser()
    p.add_argument("cmd", choices=["run", "shot", "call"])
    p.add_argument("target", nargs="?")
    p.add_argument("--url", default=DEFAULT_URL)
    p.add_argument("--pos")
    p.add_argument("--look")
    p.add_argument("--view", default="3d")
    p.add_argument("--shade")
    p.add_argument("--json", help="file with tool arguments")
    p.add_argument("--continue-on-error", action="store_true")
    p.add_argument("--timeout", type=float, default=TIMEOUT, help="seconds to wait for one call")
    a = p.parse_args()
    TIMEOUT = a.timeout
    rpc(a.url, "initialize", {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "mcp_script", "version": "1"}})
    if a.cmd == "run":
        result = call(a.url, "run_script", {"path": os.path.abspath(a.target), "continue_on_error": a.continue_on_error})
        summary = parse_summary(result)
        if summary is None:
            print(f"error: {text_of(result)}")
            sys.exit(1)
        errors = summary.get("errors", [])
        print(f"ran {summary.get('ran')} of {summary.get('steps')} steps, {summary.get('notes', 0)} notes, {len(errors)} errors")
        for e in errors:
            print(f"  step {e.get('step')} {e.get('tool')}: {e.get('error')}")
        print(json.dumps(summary.get("state", {}), indent=1))
        sys.exit(1 if errors or result.get("isError") else 0)
    if a.cmd == "shot":
        if a.shade:
            call(a.url, "set_editor", {"shade": a.shade})
        if a.pos and a.look:
            call(a.url, "set_camera", {"view": "3d", "position": vec(a.pos), "look_at": vec(a.look)})
        call(a.url, "simulate_input", {"events": [{"type": "move", "x": 5, "y": 5}]})
        result = call(a.url, "screenshot", {"target": a.view})
        for c in result.get("content", []):
            if c["type"] == "image":
                with open(a.target, "wb") as f:
                    f.write(base64.b64decode(c["data"]))
                print(f"wrote {a.target}")
        return
    args = json.load(open(a.json)) if a.json else {}
    result = call(a.url, a.target, args)
    print(text_of(result))
    sys.exit(1 if result.get("isError") else 0)


if __name__ == "__main__":
    main()
