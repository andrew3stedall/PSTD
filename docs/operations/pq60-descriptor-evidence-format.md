# PQ60: deterministic TCINFO descriptor evidence format

## Context

PQ57 exposed four bounded 14-bit row masks from the public PST fixture. PQ58 validated that TCINFO descriptor bitmap indices are unique, in range, and complete. PQ59 added a bounded builder that associates descriptor metadata with the raw row states.

The earlier PQ60 proposal would have integrated that evidence directly into the legacy table diagnostic. Review showed that the diagnostic already uses commas, colons, semicolons, and pipes as nested separators. Publishing a new multi-record payload before defining a stable wire format would make fixture evidence ambiguous and brittle.

## Revised PQ60 requirement

PQ60 defines and tests a deterministic, delimiter-safe representation for descriptor bitmap evidence. It does not yet alter extraction reporting.

Each record contains:

- bitmap index;
- original descriptor-order position;
- raw property tag;
- derived property type;
- data offset;
- data size;
- raw row states.

Records are sorted by bitmap index by the PQ59 builder and joined with `~`. Individual fields use fixed prefixes and `-` separators:

```text
b0-o1-t001f3001-y001f-d4-s4-r10~b1-o0-t001a0037-y001a-d0-s4-r01
```

The format deliberately avoids `,`, `:`, `;`, and `|`, which are already used by surrounding progress diagnostics.

Empty evidence is represented explicitly as `none`.

## Safety boundary

PQ60 remains structural evidence only. It does not:

- read fixed-width row values;
- follow HID or HNID references;
- interpret a raw set bit as semantic property presence;
- change extraction totals.

## Validation

The focused tests require:

1. deterministic bitmap-order output;
2. fixed-width hexadecimal property tags and property types;
3. preservation of descriptor order and row-state strings;
4. absence of the surrounding diagnostic delimiters;
5. explicit formatting of an empty evidence set;
6. continued rejection of malformed bitmap masks.

The full GitHub Actions workflow must remain green before merge.

## Following run: PQ61

PQ61 should carry validated `TcInfo` descriptor metadata through `TcHeapResolutionReport`, build descriptor evidence only when exact masks are available, and publish the PQ60 format in `TcHeapDiagnostic`.

PQ61 must fail closed when evidence construction fails and retain the existing mask and extraction evidence. It must not read row values or assign semantic presence.
