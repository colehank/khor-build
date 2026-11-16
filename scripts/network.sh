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
    rust='not executed (native-only diagnosis)', run_id=os.environ['GITHUB_RUN_ID'],
    control_commit=os.environ['GITHUB_SHA'], jobs=os.environ['CARGO_BUILD_JOBS']), indent=2) + '\n')
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
observer_source = REPORT / 'webkit-loopback-error-observer.swift'
text = fixture.read_text().replace('"error": error.localizedDescription', '"error": error.localizedDescription, "domain": (error as NSError).domain, "code": (error as NSError).code, "underlying": String(describing: (error as NSError).userInfo[NSUnderlyingErrorKey])')
observer_source.write_text(text)
observer_program = REPORT / 'webkit-loopback-error-observer'
observed_build = subprocess.run(['xcrun', 'swiftc', '-swift-version', '5', str(observer_source), '-o', str(observer_program)], capture_output=True, text=True)
(REPORT / 'swift-observer-build.log').write_text(observed_build.stdout + observed_build.stderr)
assert observed_build.returncode == 0

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

def case(index, host, mode, policy, observer=False):
    facts = dict(proxy_accepted=0, forward_accepted=0, connects=[], served=0, errors=[])
    stop = threading.Event()
    pool = concurrent.futures.ThreadPoolExecutor(max_workers=12)
    sockets = []
    threads = []
    proxy = listener('127.0.0.1')
    sockets.append(proxy)
    family_host = '::1' if host == '[::1]' else '127.0.0.1'
    if policy == 'v4_reserved':
        reserve = listener('127.0.0.1')
        chosen = reserve.getsockname()[1]
        reserve.close()
        first = listener(family_host, chosen)
    else:
        assert policy == 'family_allocated'
        first = listener(family_host)
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
    outcome = dict(index=index, host=host, mode=mode, port=port, facts=facts, port_policy=policy, error_observer=observer)
    try:
        env = dict(os.environ, CFNETWORK_DIAGNOSTICS='3')
        result = subprocess.run([str(observer_program if observer else program), mode, f'http://{target}/', str(proxy.getsockname()[1])], capture_output=True, text=True, timeout=30, env=env)
        (REPORT / f'webkit-{index}.stdout').write_text(result.stdout)
        (REPORT / f'webkit-{index}.stderr').write_text(result.stderr)
        outcome.update(returncode=result.returncode, result=json.loads(result.stdout), passed=result.returncode == 0 and json.loads(result.stdout).get('body') == 'BORROWED_EXIT')
    except Exception as error:
        outcome.update(error=repr(error), passed=False)
    finally:
        outcome['facts'] = json.loads(json.dumps(facts))
        if observer and not outcome.get('passed'):
            try:
                with socket.create_connection((family_host, port), timeout=2) as raw:
                    raw.sendall(f'GET / HTTP/1.1\r\nHost: {target}\r\nConnection: close\r\n\r\n'.encode())
                    response = b''
                    while len(response) < 8192:
                        chunk = raw.recv(4096)
                        if not chunk:
                            break
                        response += chunk
                outcome['raw_TCP_after_native_failure'] = dict(response=response.decode(errors='replace'), passed=response.endswith(b'BORROWED_EXIT'))
            except Exception as error:
                outcome['raw_TCP_after_native_failure'] = dict(error=repr(error), passed=False)
            outcome['oracle_facts_after'] = json.loads(json.dumps(facts))
        stop.set()
        for sock in sockets:
            sock.close()
        for thread in threads:
            thread.join(1)
        pool.shutdown(wait=True)
    return outcome

# Freeze the complete population before the first browser process.
policies = ['v4_reserved', 'family_allocated', 'family_allocated', 'v4_reserved', 'v4_reserved', 'family_allocated']
order = [(host, 'default', policy, False) for policy in policies for host in ['127.0.0.1', 'localhost', '[::1]']]
order += [('[::1]', mode, policy, False) for policy in ['v4_reserved', 'family_allocated'] for mode in ['direct', 'explicit']]
order += [('[::1]', mode, 'v4_reserved', True) for mode in ['default', 'direct', 'explicit']]
assert len(order) == 25
plan = dict(schema=1, original_fixture_sha256=hashlib.sha256(fixture.read_bytes()).hexdigest(), observer_source_sha256=hashlib.sha256(observer_source.read_bytes()).hexdigest(), order=order, primary_original_default_cases=18, original_mode_controls=4, error_observer_cases=3, runtime_executions_before_protocol=0, replacements=0, Rust_builds=0, product_changes=0, limits=['Independent Python forwarding isolates platform/port/proxy behavior; not Rust product acceptance', 'Only post-failure observer controls issue raw TCP oracles; primary cohorts have no priming probes'])
(REPORT / 'hosted-webkit-design.json').write_text(json.dumps(plan, indent=2) + '\n')
results = []
for index, (host, mode, policy, observer) in enumerate(order, 1):
    row = case(index, host, mode, policy, observer)
    results.append(row)
    (REPORT / 'hosted-webkit-diagnostic.json').write_text(json.dumps(dict(schema=1, plan=plan, results=results, accepted=False), indent=2) + '\n')
    print(json.dumps(row), flush=True)
raise SystemExit(0 if all(row['passed'] and not row['facts']['errors'] for row in results) else 1)

NATIVE_WEBKIT_DIAG
