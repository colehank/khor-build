"""Measure native WKWebView loopback proxy routing with owned local fixtures."""
import http.server
import json
import os
from pathlib import Path
import select
import socket
import socketserver
import subprocess
import tempfile
import threading


class HTTP(http.server.BaseHTTPRequestHandler):
    def do_GET(self):
        body = self.server.marker.encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/html')
        self.send_header('Content-Length', str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def log_message(self, *_):
        pass


requests = []


class Proxy(socketserver.BaseRequestHandler):
    def handle(self):
        self.request.settimeout(10)
        header = bytearray()
        while not header.endswith(b'\r\n\r\n') and len(header) < 8192:
            byte = self.request.recv(1)
            if not byte:
                return
            header.extend(byte)
        line = header.decode().splitlines()[0]
        requests.append(line)
        assert line.startswith('CONNECT 127.0.0.1:'), line
        with socket.create_connection(remote.server_address, timeout=10) as upstream:
            self.request.sendall(b'HTTP/1.1 200 Connection Established\r\n\r\n')
            ends = [self.request, upstream]
            while True:
                ready, _, _ = select.select(ends, [], [], 10)
                if not ready:
                    return
                for source in ready:
                    data = source.recv(65536)
                    if not data:
                        return
                    ends[1 if source is ends[0] else 0].sendall(data)


local = http.server.ThreadingHTTPServer(('127.0.0.1', 0), HTTP)
local.marker = 'LOCAL_DEVICE'
remote = http.server.ThreadingHTTPServer(('127.0.0.1', 0), HTTP)
remote.marker = 'BORROWED_EXIT'
proxy = socketserver.ThreadingTCPServer(('127.0.0.1', 0), Proxy)
proxy.daemon_threads = True
for server in [local, remote, proxy]:
    threading.Thread(target=server.serve_forever, daemon=True).start()

report = Path(os.environ['REPORT_DIR'])
report.mkdir(parents=True, exist_ok=True)
with tempfile.TemporaryDirectory(prefix='webkit-loopback-', dir=os.environ['RUNNER_TEMP']) as temporary:
    binary = Path(temporary) / 'probe'
    subprocess.run(['xcrun', 'swiftc', '-swift-version', '5', str(Path(__file__).with_suffix('.swift')),
                    '-o', str(binary)], check=True, timeout=90)
    results = []
    for mode in ['direct', 'default', 'explicit']:
        previous = len(requests)
        completed = subprocess.run([str(binary), mode, f'http://127.0.0.1:{local.server_port}/',
                                    str(proxy.server_address[1])], capture_output=True, text=True, timeout=30)
        (report / f'{mode}-stderr.txt').write_text(completed.stderr)
        assert completed.returncode == 0, completed.stderr
        result = json.loads(completed.stdout)
        result['proxy_requests'] = requests[previous:]
        results.append(result)
    (report / 'routing.json').write_text(json.dumps(results, indent=2) + '\n')
    print(json.dumps(results, indent=2))
    assert results[0]['body'] == 'LOCAL_DEVICE'
    assert all('body' in value for value in results), results

for server in [local, remote, proxy]:
    server.shutdown()
    server.server_close()
