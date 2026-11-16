#!/bin/bash
# Trusted control driver: portable network checks on Linux/Mac; netlab requires Linux.
set -euo pipefail
case "$DEV_TASK" in net-unit|net-node|net-vendor|net-wan-build|net-wan-lifetime|net-wan-peer|net-congestion|netlab-measure-check|netlab-cpu-check|netlab-mobility-calibrate|netlab-mobility-node|netlab-mobility-roam|netlab-peer-mobility|netlab-peer-mobility-samples|netlab-nat-rebind|netlab-nat-rebind-lifecycle|netlab-hop-reference|netlab-hop-reference-lifecycle|netlab-hop-ownership|netlab-hop-lifecycle|netlab-smoke|netlab-dynamic|netlab-compare|netlab-hop|netlab-recovery|netlab-stability|netlab-stalls|netlab-diagnostic|netlab-hop-diagnostic|netlab) ;; *) exit 2 ;; esac
mkdir -p "$REPORT_DIR"
cp "$SOURCE_PACKAGE/source.json" "$REPORT_DIR/source.json"
source_dir="${KHOR_NETWORK_SOURCE_DIR:-$RUNNER_TEMP/network-source}"
if [[ -z "${KHOR_NETWORK_SOURCE_DIR:-}" ]]; then
  git clone --no-hardlinks "$SOURCE_PACKAGE/source.git" "$source_dir"
fi
cd "$source_dir"
python3 - <<'PY'
import json, os, platform, subprocess
from pathlib import Path
report = Path(os.environ['REPORT_DIR'])
source = json.loads((report / 'source.json').read_text())
if os.environ.get('KHOR_NETWORK_SOURCE_DIR'):
    cache = json.loads((report / 'source-cache.json').read_text())
    assert cache['path'] == os.getcwd() == os.environ['KHOR_NETWORK_SOURCE_DIR']
    assert cache['tree'] == source['tree'] and cache['tracked_clean'] is True
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip() == source['build_commit']
assert subprocess.check_output(['git', 'rev-parse', 'HEAD^{tree}'], text=True).strip() == source['tree']
(report / 'environment.json').write_text(json.dumps(dict(task=os.environ['DEV_TASK'], system=platform.platform(),
    rust=subprocess.check_output(['rustc', '--version'], text=True).strip(), run_id=os.environ['GITHUB_RUN_ID'],
    control_commit=os.environ['GITHUB_SHA'], jobs=os.environ['CARGO_BUILD_JOBS']), indent=2) + '\n')
# Preserve the mtime of the same isolated web placeholder across cached builds.
for name, content in {
    'apps/gui/dist/assets/index-netlab.js': '// network test fixture\n',
    'apps/gui/dist/index.html': '<!doctype html><script type="module" src="/assets/index-netlab.js"></script>\n',
}.items():
    path = Path(name)
    path.parent.mkdir(parents=True, exist_ok=True)
    if not path.exists() or path.read_text() != content:
        path.write_text(content)
PY
python3 - <<'NATIVE_WEBKIT_DIAG'
"""Fixed native platform diagnosis; no Rust/product change or acceptance credit."""
import concurrent.futures
import hashlib
import json
import os
from pathlib import Path
import selectors
import socket
import subprocess
import threading
import time

REPORT = Path(os.environ['REPORT_DIR'])
SOURCE = Path.cwd()
fixture = SOURCE / 'apps/gui/core/tests/fixtures/webkit_loopback.swift'
program = REPORT / 'webkit-loopback-original'
build = subprocess.run(['xcrun', 'swiftc', '-swift-version', '5', str(fixture), '-o', str(program)], capture_output=True, text=True)
(REPORT / 'swift-build.log').write_text(build.stdout + build.stderr)
assert build.returncode == 0

def listener(address, port=0):
    sock = socket.socket(socket.AF_INET6 if ':' in address else socket.AF_INET)
    if ':' in address:
        sock.setsockopt(socket.IPPROTO_IPV6, socket.IPV6_V6ONLY, 1)
    sock.bind((address, port))
    sock.listen(16)
    sock.settimeout(.2)
    return sock

def head(sock):
    value = b''
    while not value.endswith(b'\r\n\r\n'):
        chunk = sock.recv(1)
        if not chunk:
            return value
        value += chunk
        assert len(value) <= 8192
    return value

def case(index, host, mode):
    facts = dict(proxy_accepted=0, forward_accepted=0, connects=[], served=0, errors=[])
    stop = threading.Event()
    pool = concurrent.futures.ThreadPoolExecutor(max_workers=12)
    sockets = []
    threads = []
    proxy = listener('127.0.0.1')
    sockets.append(proxy)
    first = listener('::1' if host == '[::1]' else '127.0.0.1')
    port = first.getsockname()[1]
    sockets.append(first)
    if host == 'localhost':
        sockets.append(listener('::1', port))
    target = f'{host}:{port}'

    def respond(client):
        with client:
            client.settimeout(5)
            request = head(client)
            if not request:
                return
            line = request.split(b'\r\n')[0].decode()
            facts['connects'].append(line)
            assert line == f'CONNECT {target} HTTP/1.1', line
            client.sendall(b'HTTP/1.1 200 Connection Established\r\n\r\n')
            request = head(client)
            if not request:
                return
            assert request.startswith(b'GET / '), request
            assert f'Host: {target}'.lower().encode() in request.lower(), request
            client.sendall(b'HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 13\r\nConnection: close\r\n\r\nBORROWED_EXIT')
            facts['served'] += 1

    def forward(client):
        with client, socket.create_connection(proxy.getsockname(), timeout=5) as upstream:
            upstream.sendall(f'CONNECT {target} HTTP/1.1\r\nHost: {target}\r\n\r\n'.encode())
            assert head(upstream).startswith(b'HTTP/1.1 200 ')
            with selectors.DefaultSelector() as selected:
                selected.register(client, selectors.EVENT_READ, upstream)
                selected.register(upstream, selectors.EVENT_READ, client)
                while not stop.is_set():
                    for key, _ in selected.select(.2):
                        value = key.fileobj.recv(65536)
                        if not value:
                            return
                        key.data.sendall(value)

    def accept(sock, handler, counter):
        def guarded(client):
            try:
                handler(client)
            except Exception as error:
                facts['errors'].append(repr(error))
        while not stop.is_set():
            try:
                client, _ = sock.accept()
            except socket.timeout:
                continue
            except OSError:
                return
            facts[counter] += 1
            pool.submit(guarded, client)

    for sock in sockets:
        thread = threading.Thread(target=accept, args=(sock, respond if sock is proxy else forward, 'proxy_accepted' if sock is proxy else 'forward_accepted'))
        thread.start()
        threads.append(thread)
    outcome = dict(index=index, host=host, mode=mode, port=port, facts=facts)
    try:
        env = dict(os.environ, CFNETWORK_DIAGNOSTICS='3')
        result = subprocess.run([str(program), mode, f'http://{target}/', str(proxy.getsockname()[1])], capture_output=True, text=True, timeout=30, env=env)
        (REPORT / f'webkit-{index}.stdout').write_text(result.stdout)
        (REPORT / f'webkit-{index}.stderr').write_text(result.stderr)
        outcome.update(returncode=result.returncode, result=json.loads(result.stdout), passed=result.returncode == 0 and json.loads(result.stdout).get('body') == 'BORROWED_EXIT')
    except Exception as error:
        outcome.update(error=repr(error), passed=False)
    finally:
        stop.set()
        for sock in sockets:
            sock.close()
        for thread in threads:
            thread.join(1)
        pool.shutdown(wait=True)
    return outcome

# Exactly three independent default-store rounds, plus two IPv6 mode controls.
order = [(host, 'default') for _ in range(3) for host in ['127.0.0.1', 'localhost', '[::1]']]
order += [('[::1]', 'direct'), ('[::1]', 'explicit')]
results = []
for index, (host, mode) in enumerate(order, 1):
    row = case(index, host, mode)
    results.append(row)
    (REPORT / 'webkit-platform-diagnostic.json').write_text(json.dumps(dict(schema=1, fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(), fixed_cases=11, replacements=0, product_changes=0, Rust_tests_run=0, results=results, accepted=False), indent=2) + '\n')
    print(json.dumps(row), flush=True)
raise SystemExit(0 if all(row['passed'] and not row['facts']['errors'] for row in results) else 1)

NATIVE_WEBKIT_DIAG
