# Method-6 OLE attachment fixture

This fixture is generated in CI from the repository's pinned MIT-licensed
[EMLtoPST](https://github.com/igrbtn/EMLtoPST) source. It contains one synthetic
message with one recognizable Compound File payload stored as
`PR_ATTACH_METHOD=6` and `PR_ATTACH_DATA_OBJ`.

No private mailbox data or opaque PST bytes are checked in. The workflow clones
EMLtoPST at commit `6fe9025390a96fe0095457b56f12ce241ee4ba53`, runs the
checked-in `prepare_generator.py` transformer against that pinned checkout,
generates the PST twice, and requires byte-for-byte equality. It then runs both
PSTD and readpst against that same generated file.

The expected payload is a deterministic 4,096-byte object whose first 24
bytes are a Compound File signature test sequence. Its size intentionally
forces EMLtoPST to store `PR_ATTACH_DATA_OBJ` through the standard subnode
reference path that readpst consumes:

```text
d0 cf 11 e0 a1 b1 1a e1 00 01 02 03 04 05 06 07
08 09 0a 0b 0c 0d 0e 0f
```

Bytes after the prefix are deterministic: byte `i` is `(i * 31 + 7) % 256`.

The generator transformer is fixture-only. It does not become a PSTD runtime or
build dependency.
