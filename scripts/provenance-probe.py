"""Measure the unmodified synthetic provenance fixture on a hosted Linux disk."""
import contextlib
import io
import json
from pathlib import Path
import sys
import unittest

sys.path.insert(0, "scripts/tests")
import test_native_provenance as fixture

original = fixture.p.generated_output
measurements = []


def observe(row, prepared, package, priors):
    out = Path(row["out_dir"])
    facts = {
        "marker_delta_ns": (out.parent / "output").stat().st_mtime_ns - prepared["prepared_ns"],
        "files": [{"name": f.name, "mtime_delta_ns": f.stat().st_mtime_ns - prepared["prepared_ns"],
                   "ctime_delta_ns": f.stat().st_ctime_ns - prepared["prepared_ns"]}
                  for f in out.rglob("*") if f.is_file()],
    }
    try:
        return original(row, prepared, package, priors)
    except ValueError as error:
        facts["error"] = str(error)
        raise
    finally:
        measurements.append(facts)


fixture.p.generated_output = observe
name = "test_gui_capabilities_directory_is_frozen_and_admitted_as_declared"
suite = unittest.TestSuite(fixture.ProvenanceTests(name) for _ in range(1000))
stream = io.StringIO()
with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(io.StringIO()):
    result = unittest.TextTestRunner(stream=stream).run(suite)
report = {"runs": result.testsRun, "errors": len(result.errors), "failures": len(result.failures),
          "measurements": measurements, "log": stream.getvalue()}
Path("provenance-clock.json").write_text(json.dumps(report, indent=2) + "\n")
print(json.dumps({"runs": result.testsRun, "errors": len(result.errors), "failures": len(result.failures),
                  "negative_markers": sum(m["marker_delta_ns"] < 0 for m in measurements),
                  "examples": [m for m in measurements if "error" in m][:5]}))
sys.exit(not result.wasSuccessful())
