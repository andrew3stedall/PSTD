#!/usr/bin/env python3
"""Compare isolated release-build output phases against a specified git baseline.

Timing is informational; byte/hash equivalence is mandatory. Peak RSS includes
synthetic input construction and JSONL buffering, not only the timed output phase.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import statistics
import subprocess
import tempfile


def run(*args, **kwargs):
    return subprocess.run(args, check=True, **kwargs)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--output", default="content-output-performance.json")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    baseline_sha = subprocess.check_output(
        ["git", "rev-parse", "--verify", f"{args.baseline}^{{commit}}"], cwd=root, text=True
    ).strip()
    head_sha = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=root, text=True).strip()
    source = root / "examples/content_output_benchmark.rs"
    results = []
    with tempfile.TemporaryDirectory(prefix="pstd-content-perf-") as temporary:
        temp = Path(temporary)
        baseline = temp / "baseline"
        run("git", "worktree", "add", "--detach", str(baseline), baseline_sha, cwd=root)
        try:
            # Use the identical fixture and timer on both revisions. Older revisions
            # expose only the collecting API; newer revisions use their iterator.
            example = source.read_text()
            for module, iterator, collecting in [
                ("reconstruction", "iter_email_content_records", "build_email_content_records"),
                ("attachment_text", "iter_attachment_text_records", "parse_attachment_text_records"),
            ]:
                if f"pub fn {iterator}" not in (baseline / f"src/output/{module}.rs").read_text():
                    example = example.replace(iterator, collecting)
            (baseline / "examples").mkdir(exist_ok=True)
            (baseline / source.relative_to(root)).write_text(example)
            # The repository does not track Cargo.lock. Resolve once, then use
            # exactly the same dependency versions for both release builds.
            if not (root / "Cargo.lock").exists():
                run("cargo", "generate-lockfile", cwd=root)
            lockfile = (root / "Cargo.lock").read_bytes()
            (baseline / "Cargo.lock").write_bytes(lockfile)
            lock_sha256 = hashlib.sha256(lockfile).hexdigest()
            toolchain = subprocess.check_output(["rustc", "--version"], text=True).strip()
            binaries = {}
            for label, checkout in [("baseline", baseline), ("head", root)]:
                run("cargo", "build", "--locked", "--release", "--example", "content_output_benchmark", cwd=checkout)
                binary = temp / label / "content-output-benchmark" if label == "baseline" else temp / "head-benchmark"
                shutil.copy2(checkout / "target/release/examples/content_output_benchmark", binary)
                binaries[label] = binary
            cases = [("email", count, 128) for count in (2000, 8000, 16000)]
            cases += [("email", 2000, 16384), ("text", 16000, 0), ("disk", 4000, 0)]
            for mode, count, body_bytes in cases:
                samples = {"baseline": [], "head": []}
                # Alternate revisions so host drift does not consistently favor one.
                for repeat in range(3):
                    for label in (["baseline", "head"] if repeat % 2 == 0 else ["head", "baseline"]):
                        rss = temp / "rss.txt"
                        completed = run("/usr/bin/time", "-f", "%M", "-o", str(rss),
                                        str(binaries[label]), mode, str(count), str(body_bytes),
                                        capture_output=True, text=True)
                        row = json.loads(completed.stdout)
                        row["peak_rss_kib"] = int(rss.read_text().strip())
                        samples[label].append(row)
                signatures = {(row["output_bytes"], row["output_sha256"])
                              for values in samples.values() for row in values}
                if len(signatures) != 1:
                    raise RuntimeError(f"output contract drift: {mode}, {count}, {body_bytes}: {signatures}")
                case = {"mode": mode, "records": count, "body_bytes": body_bytes,
                        "output_bytes": next(iter(signatures))[0], "output_sha256": next(iter(signatures))[1]}
                for label, values in samples.items():
                    case[label] = {"median_ms": statistics.median(v["elapsed_ms"] for v in values),
                                   "median_peak_rss_kib": statistics.median(v["peak_rss_kib"] for v in values),
                                   "samples": values}
                case["speedup"] = case["baseline"]["median_ms"] / case["head"]["median_ms"]
                results.append(case)
                print(json.dumps(case), flush=True)
        finally:
            run("git", "worktree", "remove", "--force", str(baseline), cwd=root)
    Path(args.output).write_text(json.dumps({"baseline_sha": baseline_sha, "head_sha": head_sha,
        "cargo_lock_sha256": lock_sha256, "rustc": toolchain,
        "scope": "synthetic output phases; excludes PST parsing; RSS includes setup and buffered JSONL",
        "cases": results}, indent=2) + "\n")


if __name__ == "__main__":
    main()
