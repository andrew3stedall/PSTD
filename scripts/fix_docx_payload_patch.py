from pathlib import Path

path = Path("scripts/apply_docx_payload_vertical.py")
text = path.read_text(encoding="utf-8")
old = '''engine = replace_once(
    engine,
    "                                attachments.append(&mut property_context_attachments);\\n",
    "                                message_property_has_attachments = true;\\n",
    "defer metadata record emission",
)
'''
new = '''engine = replace_nth(
    engine,
    "                                attachments.append(&mut property_context_attachments);\\n",
    "                                message_property_has_attachments = true;\\n",
    1,
    "defer metadata record emission",
)
'''
if text.count(old) != 1:
    raise SystemExit(f"expected one defer-emission patch block, found {text.count(old)}")
path.write_text(text.replace(old, new, 1), encoding="utf-8")
