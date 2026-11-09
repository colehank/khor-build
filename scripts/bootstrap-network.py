"""Materialize an exact source bundle using only repository-read GitHub API access."""
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import quote
from urllib.request import Request, urlopen


MAX_BUNDLE = 64 * 1024 * 1024
CONTROL_FILES = ["prepare-development.py", "native.sh", "development.sh", "package-lab.py",
                 "update-lab.py", "cli-update-lab.py", "worker-storage.py", "network.sh", "netlab-vz.py", "wan-probe-lease.py"]


def api(path, maximum):
    request = Request(os.environ['GITHUB_API_URL'] + '/repos/' + os.environ['GITHUB_REPOSITORY'] + '/' + path,
                      headers={'Authorization': 'Bearer ' + os.environ['GH_TOKEN'],
                               'Accept': 'application/vnd.github+json', 'User-Agent': 'Khor network development'})
    with urlopen(request, timeout=120) as response:
        data = response.read(maximum * 2 + 8193)
    if len(data) > maximum * 2 + 8192:
        raise ValueError('repository response exceeds the transport limit')
    return json.loads(data)


def blob(data, maximum):
    if data.get('encoding') != 'base64' or not 0 <= data['size'] <= maximum:
        raise ValueError('unexpected repository blob encoding or size')
    content = base64.b64decode(''.join(data['content'].split()), validate=True)
    digest = hashlib.sha1(b'blob ' + str(len(content)).encode() + b'\0' + content).hexdigest()
    if len(content) != data['size'] or digest != data['sha']:
        raise ValueError('repository blob identity mismatch')
    return content


def main():
    snapshot = os.environ['SNAPSHOT_SHA']
    control = os.environ['GITHUB_SHA']
    bundle_blob = os.environ['SOURCE_BUNDLE_BLOB_SHA']
    bundle_hash = os.environ['SOURCE_BUNDLE_SHA256']
    if not all(re.fullmatch(r'[0-9a-f]{40}', x) for x in [snapshot, control, bundle_blob]):
        raise ValueError('expected exact source, control and blob identities')
    if not re.fullmatch(r'[0-9a-f]{64}', bundle_hash):
        raise ValueError('expected a source bundle SHA-256')
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', os.environ['GITHUB_REPOSITORY']):
        raise ValueError('invalid private control repository name')

    temporary = Path(os.environ['RUNNER_TEMP']).resolve()
    controls = temporary / 'mac-network-control' / 'scripts'
    checkout = temporary / 'mac-bundle-snapshot'
    package = Path(os.environ['SOURCE_PACKAGE'])
    if controls.exists() or checkout.exists() or package.exists():
        raise ValueError('preserving existing materialization directories')

    response = api('git/blobs/' + bundle_blob, MAX_BUNDLE)
    content = blob(response, MAX_BUNDLE)
    if response['sha'] != bundle_blob or hashlib.sha256(content).hexdigest() != bundle_hash:
        raise ValueError('source bundle checksum mismatch')
    bundle = temporary / 'network-source.bundle'
    bundle.write_bytes(content)

    # These are trusted control files at the workflow's exact commit, not
    # scripts from the source snapshot. No git transport to github.com is used.
    controls.mkdir(parents=True)
    identities = {}
    for name in CONTROL_FILES:
        response = api('contents/scripts/' + name + '?ref=' + quote(control), 256 * 1024)
        (controls / name).write_bytes(blob(response, 256 * 1024))
        identities[name] = response['sha']

    # The read token belongs only to this trusted bootstrap step. It is not
    # inherited by local Git, source preparation, or the following build step.
    clean_env = {key: value for key, value in os.environ.items() if key not in {'GH_TOKEN', 'GITHUB_TOKEN'}}
    clean_env.update(GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1')
    subprocess.run(['git', '-c', 'core.hooksPath=/dev/null', 'clone', '--quiet', str(bundle), str(checkout)],
                   env=clean_env, check=True)
    actual = subprocess.check_output(['git', '-C', str(checkout), 'rev-parse', 'HEAD'],
                                     env=clean_env, text=True).strip()
    if actual != snapshot:
        raise ValueError('source bundle contains a different snapshot')
    subprocess.run([sys.executable, str(controls / 'prepare-development.py'), str(checkout), str(package)],
                   env=clean_env, check=True)

    source = json.loads((package / 'source.json').read_text())
    report = Path(os.environ['REPORT_DIR'])
    report.mkdir(parents=True, exist_ok=True)
    (report / 'bundle-transport.json').write_text(json.dumps({
        'transport': 'github_api_git_bundle', 'snapshot_commit': snapshot, 'source_tree': source['tree'],
        'bundle_blob_sha': bundle_blob, 'bundle_sha256': bundle_hash,
        'control_commit': control, 'control_files': identities,
    }, indent=2) + '\n')
    with Path(os.environ['GITHUB_ENV']).open('a') as output:
        output.write('NETWORK_CONTROL_DIR=' + str(controls.parent) + '\n')
    print('Verified source bundle:', snapshot, source['tree'])


if __name__ == '__main__':
    main()
