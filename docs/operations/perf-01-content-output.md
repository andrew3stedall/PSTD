# PERF-01: canonical content output scaling

_Point-in-time evidence, 6 September 2026. Issue #595; PR #596._

## Scope and implementation

The ATT-11 baseline repeatedly scanned every body, recipient, header and attachment
for each message, then retained expanded records beside serialized JSONL. Attachment
text and disk output performed quadratic record/payload joins. ID retrieval loaded
the complete manifest and duplicated the returned payload during validation.

PERF-01 uses borrowed BTreeMap/BTreeSet indexes, stable comparisons without allocating
sort keys, and iterator APIs consumed directly by the extraction runner. Existing
collecting APIs remain source-compatible. Manifest retrieval uses a reusable line
buffer and validates the returned payload directly.

Message joins now cost O(N log N), including index construction and deterministic
sorting, for N source records; attachment joins have the same bound. Payload encoding,
hashing, parsing, serialization and disk I/O still scale with bytes processed. Index
memory scales with source record count. Expanded content is retained for one message
or attachment at a time in the runner.

## Correctness fixes

- Duplicate attachment payload IDs produce `attachment_payload_duplicate_id` with no
  parsed text instead of silently selecting the first payload.
- Disk export validates duplicate IDs, duplicate/unsafe paths and ambiguous payload
  mappings before writing any attachment files. I/O failures are not transactional.
- A body key resolving only to another message's payload remains unavailable.
- ID retrieval still scans to EOF: duplicate target IDs and malformed later rows
  remain errors. Blank lines, CRLF, final unterminated rows, canonical manifest
  fallback, and size/hash validation are covered.

## Release benchmark

Baseline: `ad22804728575fdedd1e2abf1da1ac5c9f420de3`.
Measured implementation: `671463e264c44c1c17c12ec753cd768fe8dc898f`.
[Benchmark workflow run 34048333695](https://github.com/andrew3stedall/PSTD/actions/runs/34048333695)
passed all six output byte-count/SHA-256 comparisons. The subsequent serialization
borrow correction is lint-only; final CI also reruns the benchmark.

Identical synthetic driver, release builds, same resolved Cargo dependencies,
Linux GitHub runner with Rust 1.98.1. Three alternating samples per revision;
median elapsed time covers the output phase. Peak RSS includes synthetic input
construction and buffered JSONL. It is not an isolated allocation counter.

| Case | Records | Body bytes each | Baseline ms | New ms | Speedup | Baseline peak MiB | New peak MiB |
|---|---:|---:|---:|---:|---:|---:|---:|
| Email content | 2,000 | 128 | 117.93 | 16.59 | 7.11× | 30.07 | 19.23 |
| Email content | 8,000 | 128 | 1830.55 | 70.89 | 25.82× | 113.85 | 65.16 |
| Email content | 16,000 | 128 | 7048.74 | 149.64 | 47.10× | 224.84 | 128.47 |
| Email content | 2,000 | 16,384 | 273.27 | 123.66 | 2.21× | 207.80 | 122.77 |
| Attachment text joins | 16,000 | 0 | 1190.87 | 35.81 | 33.25× | 95.68 | 80.33 |
| Attachment disk export | 4,000 | 0 | 580.14 | 467.52 | 1.24× | 27.78 | 23.79 |

The 16,000-message case reduces measured peak RSS by
42.9%.
The text case intentionally uses unsupported binary attachments to isolate joins and
serialization; it does not measure DOCX/XLSX/PPTX/PDF parsing throughput.
Disk timing includes temporary-directory creation, writes and cleanup.

All measured bytes and hashes, including individual samples, are retained in the
workflow's `content-output-performance` artifact. Final workflow artifacts also
include the exact generated Cargo.lock for reproduction. Reproduce with:

```text
python scripts/benchmark_content_output.py --baseline ad22804728575fdedd1e2abf1da1ac5c9f420de3
```

## Validation

The five new integration regressions passed on the first implementation. Full
existing tests, formatting, clippy, CLI/Python/Docker checks and approved fixture
workflows are required on the final PR head. The repository's existing clippy gate
uses `-D warnings -A clippy::too_many_arguments`; this PR does not broaden that
allowance. The version command is now included in the CI CLI smoke checks.

Implementation revision `645ae4badec06f6f806025a368505cfc765544c6` passed all
28 triggered workflows, including [CI 34048431238](https://github.com/andrew3stedall/PSTD/actions/runs/34048431238),
[content contract 34048431192](https://github.com/andrew3stedall/PSTD/actions/runs/34048431192),
and [repeat performance comparison 34048431209](https://github.com/andrew3stedall/PSTD/actions/runs/34048431209).
The final documentation/workflow-artifact commit must pass the same exact-head gate;
its results are recorded in the PR before merge.

The original public-progress artifact `9993831193` was compared directly with main
artifact `9989674797`. All run-summary fields except timestamps, duration and run ID
are identical, and recipients JSONL is byte-identical. Fixture: 271,360 bytes,
SHA-256 `ee997fc7dd5c40bef49b753b782f76b17109057b18c19232cc87e0b63e0711fe`;
11 folders, one extracted message, no ordinary attachments, 115,998 recorded output
bytes and one TAR shard. Body/recipient/property coverage is unchanged.

[Tika run 34048431165](https://github.com/andrew3stedall/PSTD/actions/runs/34048431165)
and artifact `9993817999` retain 8 messages, 10 body rows, 9 recipients, 3 attachment
rows, 6 body files / 271 bytes, 3 attachment files / 40,756 bytes, two EML outputs
and the 687,104-byte archive. Fixture SHA-256:
`f2a6b1d2cad00f574e3d1c1211c4b1c854d6526caea77213adc3da92b7813ae3`.
The older two-attachment / 453-byte child / 234,496-byte archive figures in current
status pages were stale; they are corrected from this inspected evidence while
historical Vertical-38 records remain intact.

## Remaining measured/code-review boundaries

- The parser still retains recovered metadata and payloads, and JsonlBuffer still
  retains complete serialized TAR entries. This is not whole-PST bounded-memory
  streaming or a validated 10 GB PST throughput result.
- Mailbox/MSG adapters and metadata projection still contain full-collection scans.
  They require a separate measured vertical slice; these benchmarks do not claim to
  improve their rendering or PST parse times.
- On-disk lookup uses O(manifest rows) time to retain duplicate-ID detection, with
  memory proportional to the longest row plus the returned attachment. Repeated
  random retrieval could benefit from a versioned persistent index.
- Office/PDF parser breadth, representative Purview exports and readpst parity
  remain at the existing evidence boundaries.

