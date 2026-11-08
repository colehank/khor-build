"""Compare immutable release URLs with fresh redirects; never execute downloads."""
import hashlib
import gzip
import json
import os
from pathlib import Path
import subprocess
import tempfile
import urllib.error
import urllib.request

shellcheck = "https://github.com/koalaman/shellcheck/releases/download/v0.11.0/shellcheck-v0.11.0.linux.x86_64.tar.xz"
previous = "https://github.com/colehank/khor/releases/download/v0.1.15-rc.3/khor-linux-x86_64.gz"
nonce = f"?khor-ci={os.environ['GITHUB_RUN_ID']}-{os.environ['GITHUB_RUN_ATTEMPT']}"
facts = []
for name, url in (("shellcheck", shellcheck), ("previous-cli", previous)):
    for fresh in (False, True):
        fact = {"name": name, "fresh_redirect": fresh}
        try:
            request = urllib.request.Request(url + (nonce if fresh else ""), method="HEAD")
            with urllib.request.urlopen(request, timeout=25) as response:
                fact["status"] = response.status
        except urllib.error.HTTPError as error:
            fact["status"] = error.code
        except OSError as error:
            fact["error"] = type(error).__name__
        facts.append(fact)
try:
    checksum = hashlib.sha256()
    with urllib.request.urlopen(shellcheck + nonce, timeout=45) as response:
        for chunk in iter(lambda: response.read(1024 * 1024), b""):
            checksum.update(chunk)
    digest = checksum.hexdigest()
    assert digest == "8c3be12b05d5c177a04c29e3c78ce89ac86f1595681cab149b65b97c4e227198", digest
    facts.append({"name": "shellcheck", "fresh_body_sha256": digest, "verified": True})
except (OSError, AssertionError) as error:
    facts.append({"name": "shellcheck", "fresh_body_error": type(error).__name__})
try:
    with tempfile.TemporaryDirectory() as temporary:
        subprocess.run(["gh", "release", "download", "v0.11.0", "--repo", "koalaman/shellcheck",
                        "--pattern", "shellcheck-v0.11.0.linux.x86_64.tar.xz", "--dir", temporary],
                       check=True, timeout=90)
        digest = hashlib.sha256((Path(temporary) / "shellcheck-v0.11.0.linux.x86_64.tar.xz").read_bytes()).hexdigest()
        assert digest == "8c3be12b05d5c177a04c29e3c78ce89ac86f1595681cab149b65b97c4e227198", digest
        facts.append({"name": "shellcheck", "api_body_sha256": digest, "verified": True})
        subprocess.run(["gh", "release", "download", "v0.1.15-rc.3", "--repo", "colehank/khor",
                        "--pattern", "khor-linux-x86_64.gz", "--output", str(Path(temporary) / "previous.gz")],
                       check=True, timeout=120)
        previous_file = Path(temporary) / "previous.gz"
        digest = hashlib.sha256(previous_file.read_bytes()).hexdigest()
        assert digest == "92bf83cca3773456f9da55630d4d7819dc52ade3079720b79379121aacf58208", digest
        checksum = hashlib.sha256()
        with gzip.open(previous_file) as program:
            for chunk in iter(lambda: program.read(1024 * 1024), b""):
                checksum.update(chunk)
        program_digest = checksum.hexdigest()
        assert program_digest == "499dec3513b3af917be3dc866950081a09224880591f1e6950a0524ef8a8a709", program_digest
        facts.append({"name": "previous-cli", "api_body_sha256": digest,
                      "program_sha256": program_digest, "verified": True, "executed": False})
finally:
    Path("asset-fetch.json").write_text(json.dumps(facts, indent=2) + "\n")
    print(json.dumps(facts))
