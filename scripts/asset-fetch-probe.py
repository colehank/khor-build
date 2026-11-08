"""Compare immutable release URLs with fresh redirects; never execute downloads."""
import hashlib
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
finally:
    Path("asset-fetch.json").write_text(json.dumps(facts, indent=2) + "\n")
    print(json.dumps(facts))
