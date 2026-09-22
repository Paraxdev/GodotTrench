"""Drive a running Blockbench through the Chrome DevTools Protocol, evaluating JavaScript against its API.

Blockbench is Electron, so the portable build from github.com/JannisX11/blockbench/releases opens a DevTools port:
  Blockbench_portable.exe --remote-debugging-port=9333 --user-data-dir=<scratch>/profile
Needs `pip install websocket-client`.

Usage:
  python bbcdp.py eval "Format.id"
  python bbcdp.py file script.js
  python bbcdp.py shot out.png
"""
import base64
import json
import sys
import urllib.request

import websocket

PORT = 9333


def connect():
    targets = json.loads(urllib.request.urlopen(f"http://127.0.0.1:{PORT}/json/list").read())
    page = next(t for t in targets if t["type"] == "page" and "index.html" in t["url"])
    return websocket.create_connection(page["webSocketDebuggerUrl"], timeout=120, suppress_origin=True)


_id = 0


def send(ws, method, params=None):
    global _id
    _id += 1
    ws.send(json.dumps({"id": _id, "method": method, "params": params or {}}))
    while True:
        msg = json.loads(ws.recv())
        if msg.get("id") == _id:
            if "error" in msg:
                raise RuntimeError(msg["error"])
            return msg["result"]


def evaluate(ws, js):
    res = send(ws, "Runtime.evaluate", {"expression": js, "awaitPromise": True, "returnByValue": True})
    if "exceptionDetails" in res:
        d = res["exceptionDetails"]
        raise RuntimeError(d.get("exception", {}).get("description") or d.get("text"))
    return res["result"].get("value")


def screenshot(ws, path):
    data = send(ws, "Page.captureScreenshot", {"format": "png"})["data"]
    with open(path, "wb") as f:
        f.write(base64.b64decode(data))


if __name__ == "__main__":
    ws = connect()
    cmd = sys.argv[1]
    if cmd == "eval":
        print(json.dumps(evaluate(ws, sys.argv[2]), indent=1))
    elif cmd == "file":
        print(json.dumps(evaluate(ws, open(sys.argv[2], encoding="utf-8").read()), indent=1))
    elif cmd == "shot":
        screenshot(ws, sys.argv[2])
        print("wrote", sys.argv[2])
