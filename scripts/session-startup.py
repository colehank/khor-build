"""Exercise the built desktop binary's real host path in a private, bounded shell."""
import json
import os
from pathlib import Path
import selectors
import shlex
import signal
import subprocess
import tempfile
import time

report = Path(os.environ['REPORT_DIR'])
app = (Path(os.environ['KHOR_STARTUP_APP']) if os.environ.get('KHOR_STARTUP_APP')
       else Path(os.environ['CARGO_TARGET_DIR']) / 'aarch64-apple-darwin/release/bundle/macos/Khor.app')
program = app / 'Contents/MacOS/khor-gui'
info = json.loads(subprocess.check_output([str(program), '_release_info'], timeout=15))
assert info['component'] == 'desktop-app' and info['build']['source'] == os.environ['SOURCE_SHA']
facts = {'source': os.environ['SOURCE_SHA'], 'program': str(program), 'private_shell_only': True}

with tempfile.TemporaryDirectory(prefix='khor-desktop-startup-', dir=os.environ['RUNNER_TEMP']) as temporary:
    root = Path(temporary)
    work = root / 'project'
    work.mkdir()
    environment = {key: value for key, value in os.environ.items() if key in {'TMPDIR', 'LANG', 'LC_ALL', 'SYSTEMROOT'}}
    environment.update(PATH='/usr/bin:/bin:/usr/sbin:/sbin', HOME=str(root), SHELL='/bin/sh',
                       KHOR_HOME=str(root), KHOR_NAME='startup-check', KHOR_NO_MDNS='1',
                       KHOR_RUNTIME_DIR=str(root / 'runtime'))
    version = subprocess.check_output([str(program), '--version'], env=environment, timeout=10, text=True).strip()
    assert version == info['build']['version']
    facts['desktop_version_without_window'] = version
    with (report / 'desktop-startup-stderr.log').open('wb') as stderr:
        process = subprocess.Popen([str(program), '_console', str(root), 'mcp'],
                                   stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=stderr,
                                   env=environment, cwd=root, start_new_session=True)
        sequence = 0
        active = None
        selector = selectors.DefaultSelector()
        selector.register(process.stdout, selectors.EVENT_READ)

        def request(method, params=None):
            global sequence
            sequence += 1
            process.stdin.write((json.dumps({'jsonrpc': '2.0', 'id': sequence, 'method': method,
                                            'params': params or {}}) + '\n').encode())
            process.stdin.flush()
            deadline = time.monotonic() + 25
            while time.monotonic() < deadline:
                if not selector.select(.2):
                    continue
                line = process.stdout.readline()
                if not line:
                    raise RuntimeError('desktop MCP ended before its reply')
                value = json.loads(line)
                if value.get('id') == sequence:
                    if 'error' in value or value.get('result', {}).get('isError'):
                        raise RuntimeError(str(value))
                    return value['result']
            raise TimeoutError('desktop request did not answer')

        def tool(name, arguments):
            return request('tools/call', {'name': name, 'arguments': arguments})

        try:
            request('initialize', {'protocolVersion': '2024-11-05', 'capabilities': {},
                                   'clientInfo': {'name': 'khor-owned-startup-check', 'version': '1'}})
            marker = root / 'ready'
            location = root / 'cwd'
            command = f'pwd > {shlex.quote(str(location))}; printf ready > {shlex.quote(str(marker))}; sleep 15'
            opened = tool('open', {'machine': 'startup-check', 'cwd': str(work), 'title': 'Private startup check', 'command': command})
            active = opened['content'][0]['text'].strip()
            assert active.startswith('shell/'), opened
            deadline = time.monotonic() + 15
            while not marker.exists() and time.monotonic() < deadline:
                time.sleep(.05)
            assert marker.read_text() == 'ready'
            assert Path(location.read_text().strip()).resolve() == work.resolve()
            manifests = [json.loads(path.read_text()) for path in (root / 'runtime').glob('*/manifest.json')]
            assert any(m.get('role') == 'desktop-host' and m.get('schema') == 2 and m.get('app_bundle') for m in manifests)
            payloads = list((root / 'runtime').glob('*/Khor.app/Contents/Info.plist'))
            assert payloads
            tool('close', {'session': active})
            active = None
            facts.update(shell_created=True, cwd_preserved=True, managed_full_bundle=True, own_session_closed=True)
        finally:
            if active and process.poll() is None:
                try:
                    tool('close', {'session': active})
                except (OSError, RuntimeError, TimeoutError, ValueError):
                    # The fixture shell is bounded independently of cleanup.
                    time.sleep(16)
            if process.stdin:
                process.stdin.close()
            try:
                process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                os.killpg(process.pid, signal.SIGKILL)
                process.wait(timeout=5)
            selector.close()
facts['complete'] = True
(report / 'desktop-session-startup.json').write_text(json.dumps(facts, indent=2) + '\n')
print(json.dumps(facts), flush=True)
