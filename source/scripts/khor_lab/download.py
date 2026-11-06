"""Acquire hosted artifacts without forwarding GitHub credentials to signed storage URLs."""
import hashlib
import subprocess
from concurrent.futures import ThreadPoolExecutor
import threading
from urllib.error import HTTPError
from urllib.parse import urlsplit
from urllib.request import HTTPRedirectHandler, Request, build_opener, urlopen

from .artifacts import unpack


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, *_args, **_kwargs):
        return None


def _ranges(location, archive, size, progress):
    lock = threading.Lock()
    received = 0
    with archive.open('wb') as output:
        output.truncate(size)
    def part(start):
        nonlocal received
        end = min(start + 4 * 1024 * 1024, size) - 1
        request = Request(location, headers={'Range': f'bytes={start}-{end}'})
        with urlopen(request, timeout=60) as response, archive.open('r+b') as output:
            if response.status != 206 or response.headers.get('Content-Range') != f'bytes {start}-{end}/{size}':
                raise ValueError('Artifact server did not honor the requested range')
            output.seek(start)
            written = 0
            while chunk := response.read(64 * 1024):
                written += len(chunk)
                if written > end - start + 1:
                    raise ValueError('Artifact range exceeded its boundary')
                output.write(chunk)
                with lock:
                    received += len(chunk)
                    progress(received, size)
            if written != end - start + 1:
                raise ValueError('Incomplete artifact range')
    with ThreadPoolExecutor(max_workers=8) as pool:
        list(pool.map(part, range(0, size, 4 * 1024 * 1024)))


def acquire(repository, artifact, destination, progress=lambda _received, _total: None):
    token = subprocess.check_output(['gh', 'auth', 'token'], text=True, timeout=20).strip()
    api = f'https://api.github.com/repos/{repository}/actions/artifacts/{int(artifact["id"])}/zip'
    request = Request(api, headers={'Authorization': 'Bearer ' + token})
    try:
        response = build_opener(NoRedirect).open(request, timeout=30)
    except HTTPError as error:
        if error.code != 302:
            error.close()
            raise ValueError(f'GitHub artifact request failed ({error.code})') from None
        location = error.headers['Location']
        error.close()
        parts = urlsplit(location)
        if parts.scheme != 'https' or not (parts.hostname or '').endswith('.blob.core.windows.net') or parts.username:
            raise ValueError('Unexpected artifact storage destination')
        # A fresh request has no GitHub authorization header.
        response = None
    archive = destination / '.download.zip'
    try:
        size = int(artifact['size_in_bytes'])
        if not 0 < size <= 512 * 1024 * 1024:
            raise ValueError('Lab download exceeds its size budget')
        if response is None and size > 16 * 1024 * 1024:
            _ranges(location, archive, size, progress)
        else:
            response = response or urlopen(location, timeout=60)
            with response, archive.open('wb') as output:
                total = 0
                while chunk := response.read(64 * 1024):
                    total += len(chunk)
                    if total > size:
                        raise ValueError('Lab download exceeds its declared size')
                    output.write(chunk)
                    progress(total, size)
                if total != size:
                    raise ValueError('Incomplete artifact download')
        expected = artifact.get('digest')
        with archive.open('rb') as stream:
            actual_digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        if expected and expected != 'sha256:' + actual_digest:
            raise ValueError('GitHub artifact digest differs')
        unpack(archive, destination)
    except Exception as error:
        # Signed URLs are credentials; never include them in public status or logs.
        raise ValueError(f'Artifact acquisition failed ({type(error).__name__}); retry acquisition without rebuilding') from None
    finally:
        archive.unlink(missing_ok=True)
