#!/usr/bin/env python3
"""Apply the pinned, fixture-only method-6 generator changes."""

from pathlib import Path
import sys


def replace_once(path: Path, old: str, new: str) -> None:
    text = path.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise SystemExit(f"{path}: expected one anchor, found {count}: {old!r}")
    path.write_text(text.replace(old, new), encoding="utf-8")


def main() -> None:
    if len(sys.argv) != 2:
        raise SystemExit("usage: prepare_generator.py EMLtoPST_CHECKOUT")
    root = Path(sys.argv[1])
    properties = root / "eml2pst/mapi/properties.py"
    message = root / "eml2pst/messaging/message.py"
    if not properties.is_file() or not message.is_file():
        raise SystemExit(f"not an EMLtoPST checkout: {root}")

    replace_once(
        properties,
        "PT_BINARY = 0x0102  # Binary blob\n",
        "PT_BINARY = 0x0102  # Binary blob\n",
    )
    replace_once(
        properties,
        "PR_ATTACH_DATA_BIN = prop_tag(0x3701, PT_BINARY)\n",
        "PR_ATTACH_DATA_BIN = prop_tag(0x3701, PT_BINARY)\n"
        "PR_ATTACH_DATA_OBJ = prop_tag(0x3701, PT_BINARY)\n",
    )
    replace_once(
        message,
        "    PR_ATTACH_LONG_FILENAME, PR_ATTACH_SIZE, PR_ATTACH_DATA_BIN,\n",
        "    PR_ATTACH_LONG_FILENAME, PR_ATTACH_SIZE, PR_ATTACH_DATA_BIN,\n"
        "    PR_ATTACH_DATA_OBJ,\n",
    )
    replace_once(
        message,
        "from ..utils import filetime_now\n\n\n",
        "from ..utils import filetime_now\n\n\n"
        "ATTACH_OLE = 6\n\n\n",
    )
    replace_once(
        message,
        "    return build_tc_node(column_tags, rows)\n\n\ndef build_attachments_tc",
        "    return build_tc_node(column_tags, rows)\n\n\n"
        "def attachment_method(attachment):\n"
        "    \"\"\"Use method 6 only for the fixture's OLE MIME marker.\"\"\"\n"
        "    if attachment.get('mime_type') == 'application/vnd.ms-ole-storage':\n"
        "        return ATTACH_OLE\n"
        "    return ATTACH_BY_VALUE\n\n\n"
        "def build_attachments_tc",
    )
    replace_once(
        message,
        "            PR_ATTACH_METHOD: ATTACH_BY_VALUE,\n"
        "            PR_ATTACH_LONG_FILENAME:",
        "            PR_ATTACH_METHOD: attachment_method(att),\n"
        "            PR_ATTACH_LONG_FILENAME:",
    )
    replace_once(
        message,
        "        (PR_ATTACH_METHOD, ATTACH_BY_VALUE),\n"
        "        (PR_ATTACH_LONG_FILENAME,",
        "        (PR_ATTACH_METHOD, attachment_method(attachment)),\n"
        "        (PR_ATTACH_LONG_FILENAME,",
    )
    replace_once(
        message,
        "        props.append((PR_ATTACH_DATA_BIN, attachment['data']))\n",
        "        data_tag = (\n"
        "            PR_ATTACH_DATA_OBJ\n"
        "            if attachment_method(attachment) == ATTACH_OLE\n"
        "            else PR_ATTACH_DATA_BIN\n"
        "        )\n"
        "        props.append((data_tag, attachment['data']))\n",
    )

    store = root / "eml2pst/messaging/store.py"
    replace_once(
        store,
        "    record_key = os.urandom(16)\n",
        "    record_key = bytes.fromhex('00112233445566778899aabbccddeeff')\n",
    )

    utils = root / "eml2pst/utils.py"
    replace_once(
        utils,
        "    return datetime_to_filetime(datetime.now(timezone.utc))\n",
        "    return datetime_to_filetime(datetime(2026, 8, 3, tzinfo=timezone.utc))\n",
    )


if __name__ == "__main__":
    main()
