# PSTD v1 M14 Implementation Plan

## Implementation intent

M14 adds safe recursive subnode layout exploration without claiming unsupported PST layouts are decoded. The implementation classifies loaded subnode blocks, follows a known child-reference layout under parser limits, and records unsupported layouts explicitly for later compatibility work.

## Current foundation

- M12 loads bounded subnode root blocks and parses attachment tables from loaded blocks.
- M13 records compatibility diagnostics for table parse failures and expands synthetic payload coverage.
- M11 writes payload bytes to TAR archives when extraction produces payloads.

## Implemented M14 slice

1. Subnode layout classification:
   - Adds `SubnodeBlockLayout`.
   - Adds `SubnodeLayoutReport`.
   - Classifies loaded subnode blocks as table-compatible, known child-reference, or unsupported.
2. Recursive bounded loading:
   - Adds `load_recursive_subnode_blocks`.
   - Uses duplicate guards to avoid repeated block loading.
   - Uses `ParserLimits::max_subnode_depth` to prevent unbounded recursion.
   - Reports recursive child references and decoded child counts.
3. Extraction-path integration:
   - Main metadata extraction now uses recursive bounded subnode loading for attachment subnodes.
   - Extraction status records child references, recursive child decodes, unsupported layouts, and attachment table parse errors.
4. Synthetic validation:
   - Adds layout classification tests.
   - Adds mixed layout compatibility tests.
   - Adds recursive child loading tests.
   - Adds depth-limit tests.

## Status behaviour

M14 introduces these status values:

- `subnode_layouts_classified`
- `subnode_layouts_partially_classified`
- `subnode_layouts_unsupported`
- `subnode_layout_child_references_classified`
- `subnode_layout_child_references_truncated`
- `subnode_recursive_blocks_loaded`
- `subnode_recursive_blocks_partially_loaded`
- `subnode_recursive_depth_limit_reached`
- `subnode_recursive_blocks_unavailable`

## Remaining work

- Add compatibility handling for observed real-world child-subnode layouts.
- Expand fixture validation using public or sanitized samples where safe.
- Improve recursive layout identification beyond the current known/synthetic child-reference shape.
- Keep unknown layouts explicit until decoded with tests.

## Definition of done

- M14 branch keeps CI green.
- Recursive child loading is bounded and tested.
- Layout reports preserve unsupported structures instead of masking them.
- Project status and changelog document the M14 slice and remaining limitations.
