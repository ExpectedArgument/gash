"""Tiny local server for render.html: serves assets/ and saves rendered videos.

Usage: python assets/video/server.py   then open http://localhost:8765/video/render.html
Only listens on 127.0.0.1. POST /save/<name>.mp4 writes the body to assets/<name>.mp4.
"""

import http.server
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]  # assets/
PORT = 8765


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def end_headers(self):
        self.send_header("Cache-Control", "no-store")
        super().end_headers()

    def do_POST(self):
        m = re.fullmatch(r"/save/([A-Za-z0-9._-]+\.mp4)", self.path)
        if not m:
            self.send_error(404)
            return
        length = int(self.headers.get("Content-Length", "0"))
        data = self.rfile.read(length)
        out = ROOT / m.group(1)
        out.write_bytes(data)
        self.send_response(200)
        self.send_header("Content-Type", "text/plain")
        self.end_headers()
        self.wfile.write(f"saved {out.name} ({len(data)} bytes)".encode())


if __name__ == "__main__":
    print(f"serving {ROOT} on http://127.0.0.1:{PORT}/", flush=True)
    http.server.ThreadingHTTPServer(("127.0.0.1", PORT), Handler).serve_forever()
