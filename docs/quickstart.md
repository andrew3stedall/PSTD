# PSTD Quickstart

PSTD is an experimental, email-focused PST-to-EML extractor written in Rust. It currently has its strongest evidence on Unicode PST files. Use copies of PST files while testing it, review diagnostics, and do not assume that a successful run proves complete mailbox coverage.

## Choose how to run PSTD

There are three practical options:

1. **Build the Rust command-line program locally.** Best for developers and current testing.
2. **Run the compiled program from Python.** Best when a Python application needs PSTD today.
3. **Use Docker.** Best when installing Rust directly is inconvenient.

The current Python package is intentionally a thin wrapper around the compiled Rust executable. Python does not parse PST internals. This keeps one extraction implementation and avoids duplicating safety rules in two languages.

A native Python extension built with tools such as PyO3 or maturin may be useful later, but it is not the recommended interface yet. PSTD's public Rust API and extraction records should stabilise before introducing platform-specific Python wheels.

## Requirements

For a local build:

- Git;
- a recent stable Rust toolchain installed with `rustup`;
- Python 3.10 or newer only when using the Python wrapper;
- sufficient free disk space for the PST and generated extraction output.

Rust can compile on Windows, macOS, and Linux, but the resulting executable is specific to the operating system and CPU architecture on which it was built. A Windows executable will not run on macOS or Linux, for example.

## 1. Clone and build

```bash
git clone https://github.com/andrew3stedall/PSTD.git
cd PSTD
cargo build --release
```

The compiled executable will be created at:

```text
target/release/pstd          # Linux and macOS
target\release\pstd.exe      # Windows
```

Check the build:

```bash
./target/release/pstd version
./target/release/pstd --help
```

On Windows PowerShell:

```powershell
.\target\release\pstd.exe version
.\target\release\pstd.exe --help
```

## 2. Inspect a PST before extracting

Inspection reads bounded structural evidence and reports diagnostics without creating the full extraction output.

```bash
./target/release/pstd inspect --input /path/to/mailbox.pst
```

For machine-readable output:

```bash
./target/release/pstd inspect --input /path/to/mailbox.pst --json
```

Review the reported variant, diagnostics, unsupported structures, and completeness status before proceeding.

## 3. Extract structured data

```bash
./target/release/pstd extract \
  --input /path/to/mailbox.pst \
  --output ./pstd-output
```

PSTD writes deterministic structured output containing records such as folders, messages, recipients, bodies, attachments, summaries, and diagnostics when those records are supported by validated evidence.

Do not treat an exit code of zero by itself as proof that every item in the PST was extracted. Check the summary and diagnostic records for unavailable, partial, unsupported, ambiguous, corrupt, or non-mail objects.

## 4. Generate EML files

The repository includes the `pstd-eml` binary for generating admissible EML output from validated message evidence.

Build all binaries:

```bash
cargo build --release --bins
```

Run it with a PST and destination directory:

```bash
./target/release/pstd-eml /path/to/mailbox.pst ./eml-output
```

Only messages meeting the current validated EML requirements are emitted. PSTD fails closed rather than inventing missing sender, recipient, Date, body, attachment, ownership, or embedded-message evidence.

## 5. Call PSTD from Python

### Install the thin Python wrapper

From the repository root:

```bash
python -m venv .venv
```

Activate it:

```bash
source .venv/bin/activate        # Linux or macOS
.venv\Scripts\Activate.ps1       # Windows PowerShell
```

Install the wrapper:

```bash
python -m pip install -e ./python
```

Point the wrapper at the compiled Rust executable:

```bash
export PSTD_BINARY="$PWD/target/release/pstd"          # Linux or macOS
$env:PSTD_BINARY="$PWD\target\release\pstd.exe"       # Windows PowerShell
```

Then use the same command surface through Python:

```bash
python -m pstd version
python -m pstd inspect --input /path/to/mailbox.pst --json
python -m pstd extract --input /path/to/mailbox.pst --output ./pstd-output
```

If `pstd` is already on `PATH`, `PSTD_BINARY` is not required.

### Call it from Python code

For application code, invoke the compiled executable as a subprocess and require an explicit return-code check:

```python
from __future__ import annotations

import json
import os
import subprocess
from pathlib import Path


def inspect_pst(pst_path: Path, pstd_binary: Path) -> dict:
    completed = subprocess.run(
        [
            str(pstd_binary),
            "inspect",
            "--input",
            str(pst_path),
            "--json",
        ],
        check=False,
        capture_output=True,
        text=True,
        env=os.environ.copy(),
    )

    if completed.returncode != 0:
        raise RuntimeError(
            "PSTD inspection failed "
            f"with exit code {completed.returncode}:\n{completed.stderr}"
        )

    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise RuntimeError("PSTD returned invalid JSON") from exc


result = inspect_pst(
    Path("mailbox.pst"),
    Path("target/release/pstd"),
)
print(result)
```

For extraction, pass `extract`, `--input`, and `--output` in the same way. Keep the output directory outside the source PST directory and capture both standard output and standard error for audit and troubleshooting.

## 6. Use a precompiled executable with Python

End users do not need the Rust compiler when they receive a compatible precompiled PSTD executable. The executable must match their platform, for example:

- Windows x86-64;
- macOS Apple silicon;
- macOS Intel;
- Linux x86-64;
- Linux ARM64.

A Python application can bundle the appropriate executable or install it beside the application, then set `PSTD_BINARY` to its absolute path.

Until automated release binaries and signed Python wheels exist, build from source or produce controlled binaries in CI for each supported platform. Record the PSTD commit SHA used to build each executable.

## 7. Use Docker instead of installing Rust

Build the image:

```bash
docker build -t pstd:local -f docker/Dockerfile .
```

Mount an input and output directory when running it. Adapt the paths for your host operating system:

```bash
docker run --rm \
  -v "$PWD/data:/data:ro" \
  -v "$PWD/output:/output" \
  pstd:local \
  inspect --input /data/mailbox.pst --json
```

For extraction:

```bash
docker run --rm \
  -v "$PWD/data:/data:ro" \
  -v "$PWD/output:/output" \
  pstd:local \
  extract --input /data/mailbox.pst --output /output
```

## Testing guidance

Start with a copied Unicode PST containing ordinary email, then compare PSTD's results with the source mailbox using counts and sampled messages.

Check at least:

- folder paths and message ownership;
- total discovered, extracted, unavailable, and unsupported objects;
- sender and To/Cc/Bcc values;
- subject and Date;
- plain, HTML, and RTF body availability;
- attachment names, sizes, hashes, and MIME structure;
- inline images and Content-ID behaviour;
- embedded messages;
- diagnostics for native Exchange addresses, corrupt records, and unsupported objects.

Do not use contacts, appointments, tasks, journals, or distribution lists as email completeness counts. They are non-mail objects and must be classified separately rather than converted to EML.

## Current limitations

PSTD is not yet a general-purpose or production-ready PST converter. Current validation is fixture-limited. Important incomplete areas include broader Unicode producers, real ANSI traversal, inline attachments, authoritative Exchange-to-SMTP resolution, deeper nested messages, uncommon attachment layouts, and corrupt or unusual PST structures.

For current capability and fixture evidence, see:

- [Project status](product/project-status.md)
- [Compatibility matrix](product/compatibility-matrix.md)
- [Roadmap](product/pstd-v1-roadmap.md)
- [Unsupported and deferred areas](operations/v1-unsupported-deferred-areas.md)
