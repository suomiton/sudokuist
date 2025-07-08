#!/usr/bin/env python3
"""
Simple HTTP server with proper WASM MIME type support
"""

import http.server
import socketserver
import mimetypes
import os

# Add WASM MIME type
mimetypes.add_type('application/wasm', '.wasm')

class CustomHTTPRequestHandler(http.server.SimpleHTTPRequestHandler):
    def end_headers(self):
        self.send_header('Cross-Origin-Embedder-Policy', 'require-corp')
        self.send_header('Cross-Origin-Opener-Policy', 'same-origin')
        super().end_headers()

if __name__ == "__main__":
    PORT = 3000
    os.chdir('dist')
    
    with socketserver.TCPServer(("", PORT), CustomHTTPRequestHandler) as httpd:
        print(f"🚀 Serving Sudokuist at http://localhost:{PORT}")
        print(f"📱 Test page at http://localhost:{PORT}/test.html")
        print("Press Ctrl+C to stop")
        try:
            httpd.serve_forever()
        except KeyboardInterrupt:
            print("\n👋 Server stopped")
