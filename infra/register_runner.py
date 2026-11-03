import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import sys
import time

ROOT = Path('/Users/z13/khor-builder')
INSTALL = ROOT / 'tools/actions-runner-2.338.2'
URL = 'https://github.com/colehank/khor-build'
NAME = 'khor-mac-mini-m4'
LABEL = 'com.khor.github-runner.macmini'
ENV = {
    'HOME': '/Users/z13', 'USER': 'z13', 'LOGNAME': 'z13', 'LANG': 'en_US.UTF-8',
    'PATH': '/Users/z13/khor-builder/tools/node-v22.19.0-darwin-arm64/bin:/Users/z13/khor-builder/tools/bun-1.3.14/bun-darwin-aarch64:/Users/z13/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin',
    'DEVELOPER_DIR': str(ROOT / 'tools/xcode-26.5/Xcode.app/Contents/Developer'),
    'RUSTUP_TOOLCHAIN': '1.95.0', 'CARGO_HOME': str(ROOT / 'cache/cargo'),
    'CARGO_TARGET_DIR': str(ROOT / 'cache/target'), 'CARGO_BUILD_JOBS': '2',
    'CARGO_PROFILE_DEV_INCREMENTAL': 'true', 'CARGO_PROFILE_TEST_INCREMENTAL': 'true',
    'CARGO_PROFILE_RELEASE_INCREMENTAL': 'false', 'CARGO_PROFILE_DEV_DEBUG': '0',
    'CARGO_PROFILE_TEST_DEBUG': '0', 'npm_config_cache': str(ROOT / 'cache/npm'),
    'BUN_INSTALL_CACHE_DIR': str(ROOT / 'cache/bun'), 'TMPDIR': str(ROOT / 'tmp'),
    'GIT_CONFIG_NOSYSTEM': '1', 'GIT_CONFIG_GLOBAL': str(ROOT / 'cache/runner.gitconfig'), 'GIT_TERMINAL_PROMPT': '0',
}


def machine_guard():
    actual = subprocess.check_output(['/usr/bin/ssh-keygen', '-lf', '/etc/ssh/ssh_host_ed25519_key.pub'], text=True)
    if 'SHA256:00zmPFXf6a0eJOGWGyW/PkYmYO160+B+Q7TgUNJnjos' not in actual:
        raise RuntimeError('Wrong machine')


def install_service():
    machine_guard()
    if os.geteuid() != 0:
        raise RuntimeError('Service installation requires sudo')
    if not (INSTALL / '.runner').exists() or URL not in (INSTALL / '.runner').read_text():
        raise RuntimeError('Expected private-repository runner configuration missing')
    plist = Path('/Library/LaunchDaemons') / (LABEL + '.plist')
    if plist.exists():
        raise RuntimeError('Service already exists; preserve it for inspection')
    git_config = ROOT / 'cache/runner.gitconfig'
    git_config.touch(exist_ok=True)
    os.chown(git_config, 501, 20)
    git_config.chmod(0o600)
    entry = INSTALL / 'runsvc.sh'
    shutil.copy2(INSTALL / 'bin/runsvc.sh', entry)
    os.chown(entry, 501, 20)
    entry.chmod(0o700)
    logs = []
    for suffix in ['stdout', 'stderr']:
        path = ROOT / 'logs' / ('github-runner-' + suffix + '.log')
        path.touch(exist_ok=True)
        os.chown(path, 501, 20)
        path.chmod(0o600)
        logs.append(str(path))
    data = {'Label': LABEL, 'ProgramArguments': [str(entry)], 'WorkingDirectory': str(INSTALL),
            'UserName': 'z13', 'GroupName': 'staff', 'SessionCreate': True,
            'EnvironmentVariables': ENV, 'RunAtLoad': True, 'KeepAlive': True,
            'ThrottleInterval': 30, 'ProcessType': 'Background',
            'StandardOutPath': logs[0], 'StandardErrorPath': logs[1]}
    plist.write_bytes(plistlib.dumps(data))
    os.chown(plist, 0, 0)
    plist.chmod(0o644)
    subprocess.run(['/usr/bin/plutil', '-lint', str(plist)], check=True)
    service = INSTALL / '.service'
    service.write_text(str(plist) + '\n')
    os.chown(service, 501, 20)
    service.chmod(0o600)
    subprocess.run(['/bin/launchctl', 'bootstrap', 'system', str(plist)], check=True)
    receipt = ROOT / 'logs/runner-service.json'
    receipt.write_text(json.dumps({'label': LABEL, 'plist': str(plist), 'user': 'z13',
        'entry': str(entry), 'repository': URL, 'installed_at': time.time()}, indent=2) + '\n')
    os.chown(receipt, 501, 20)
    receipt.chmod(0o600)
    print('SERVICE_INSTALLED', LABEL, 'user=z13', 'official runsvc.sh entry')


def register(secret):
    machine_guard()
    if os.geteuid() != 501:
        raise RuntimeError('Runner configuration must execute as z13')
    if (INSTALL / '.runner').exists():
        raise RuntimeError('Runner is already configured; preserve existing registration')
    args = ['./config.sh', '--unattended', '--url', URL, '--token', secret,
            '--name', NAME, '--labels', 'khor-mac-mini,khor-dev', '--no-default-labels', '--work', '_work']
    result = subprocess.run(args, cwd=INSTALL, env=ENV, capture_output=True, text=True, timeout=240)
    safe = (result.stdout + result.stderr).replace(secret, '[REDACTED]')
    path = ROOT / 'logs/runner-config.log'
    path.write_text(safe)
    path.chmod(0o600)
    print(safe)
    if result.returncode:
        raise RuntimeError('Runner configuration failed; sanitized log preserved')
    for name in ['.credentials', '.credentials_rsaparams']:
        path = INSTALL / name
        if path.exists():
            path.chmod(0o600)
    result = subprocess.run(['/usr/bin/sudo', '-n', '/usr/bin/python3', str(ROOT / 'bin/register_runner.py'), '--install-service'], capture_output=True, text=True, timeout=60)
    print((result.stdout + result.stderr).replace(secret, '[REDACTED]'))
    if result.returncode:
        raise RuntimeError('Runner registered but service installation failed; do not register twice')


secret = ''
try:
    if sys.argv[1:] == ['--install-service']:
        install_service()
    else:
        request = json.load(sys.stdin)
        secret = request['token']
        if not isinstance(secret, str) or not secret:
            raise RuntimeError('Missing short-lived registration token')
        register(secret)
except BaseException as error:
    message = str(error)
    if secret:
        message = message.replace(secret, '[REDACTED]')
    print(type(error).__name__ + ': ' + message, file=sys.stderr)
    sys.exit(1)
