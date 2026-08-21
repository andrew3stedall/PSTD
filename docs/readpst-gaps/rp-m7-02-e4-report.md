# RP-M7-02 E4 semantic differential report

Reviewed 21 August 2026 against main `e55aae0816c5abace079c81aa4dcfca26c1003a3`
and pinned libpst/readpst revision
`cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89`.

## Decision

The differential harness is operational and passed its pinned-oracle run, but the
release-wide E4 gate is **not proven**. The exact reason is evidence breadth, not a
hidden test failure: one approved Unicode fixture is available for a true readpst/PSTD
run, while the remaining input families and output profiles do not yet have admissible
semantic corpora that cover every readpst-visible row. This report therefore records a
release blocker and does not promote any matrix row.

## Pinned differential evidence

| Field | Observed value |
|---|---|
| Upstream revision | `cc600ee98c4ed23b8ab0bc2cf6b6c6e9cb587e89` |
| Upstream binary | ReadPST / LibPST v0.6.76 |
| PSTD baseline | `e55aae0816c5abace079c81aa4dcfca26c1003a3` |
| Differential workflow | run `32512518536`, job `96866688030` |
| Differential artifact | `readpst-differential-evidence`, artifact `9457584897`, 2,986 bytes |
| Harness result | 18 tests passed, including configured pinned-oracle differential |
| Fixture | approved Apache Tika `testPST.pst`, SHA-256 `f2a6b1d2cad00f574e3d1c1211c4b1c854d6526caea77213adc3da92b7813ae3` |
| Repeatability | configured runner executes two isolated pairs and requires normalized-output equality |
| Safety | isolated roots, 30-second runtime, stdout/stderr/file/byte limits, path-escape rejection, explicit timeout/output-limit failures |

The workflow builds readpst from the pinned source, runs `readpst -e -8` and PSTD
canonical extraction in isolated roots, normalizes semantic records without using
filenames as identity, compares records and payload hashes, and writes a deterministic
report. The comparator and runner negative tests cover empty/mismatched output,
malformed/ambiguous outcomes, path traversal, timeouts, and output limits.

## Regression-profile admissibility

| Upstream profile | Evidence status | Reason / next admissible evidence |
|---|---|---|
| `default` | E2 differential; Partial | One approved Unicode fixture passes the pinned run; broader producer/item corpus remains required. |
| `separate` | E2/E3 adapter evidence; E4 unavailable | Adapter paths and negative decisions are tested, but no admitted corpus proves readpst semantic equivalence for all separate payload cases. |
| `recursive` | E2/E3 adapter evidence; E4 unavailable | Safe recursive output and deterministic paths are tested; mixed-folder and broad ownership corpus remains required. |
| `mh` | E2/E3 adapter evidence; E4 unavailable | MH/rfc822 boundaries are tested; independent readpst comparison across item classes is not admitted. |
| `kmail` | E2/E3 adapter evidence; E4 unavailable | KMail path/index policy is tested; import/read and broad readpst corpus evidence remains required. |
| `thunderbird` | E2/E3 adapter evidence; E4 unavailable | Sidecars and typed files are independently parsed; exact Thunderbird import compatibility remains open. |
| `debug` | E2/E3 operational evidence; E4 not applicable as semantic parity | Bounded structured diagnostics and truncation statuses are tested; debug formatting is not a capability-row promotion. |
| `valgrind-resource` | E2/E3 bounded-resource evidence; E4 unavailable | Rust limits, timeout, output, file, path, worker, and hardening tests pass; no upstream valgrind run is admitted as a release dependency. |

## Input-family and negative evidence

- Unicode: approved Tika differential is positive but remains E2/Partial.
- ANSI v14/v15 and OST 2013: controlled structural fixtures, malformed/truncated
  negatives, and repeat-run evidence pass; no admissible broad item/output corpus is
  available for E4 promotion.
- Crypt methods 0/1/2: pinned vectors and production bounded decoding pass; unknown
  methods remain explicit unsupported outcomes; encrypted semantic corpus remains open.
- Malformed, ambiguous, unsupported, and unsafe cases: fail-closed statuses, path
  confinement, symlink rejection, bounded discovery, diagnostic caps, and worker-count
  equality are covered by the RP-M6 evidence workflows.
- Private or unreviewed PST payloads are not admitted. Missing profiles remain
  `unavailable`/`waiting-evidence`, never implicit parity.

## Release consequence

The E4 harness gate is green as an executable control, but the release-wide E4 claim is
blocked by the 19 Gap and 54 Partial rows in the RP-M7-01 matrix report. RP-M7-03 must
publish a final decision that names those rows, distinguishes implementation gaps from
admissibility blockers, and avoids a full-parity claim.
