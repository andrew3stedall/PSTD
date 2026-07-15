#!/usr/bin/env python3
"""Fail-closed recovery of HTML represented by Outlook \fromhtml1 RTF."""

from __future__ import annotations

import argparse
from pathlib import Path

SKIP_DESTINATIONS = {
    "fonttbl",
    "colortbl",
    "stylesheet",
    "info",
    "pict",
    "object",
    "generator",
}


class RtfHtmlError(ValueError):
    pass


def _read_control(data: str, index: int) -> tuple[str, int | None, int]:
    index += 1
    if index >= len(data):
        raise RtfHtmlError("trailing backslash")
    char = data[index]
    if char in "{}\\":
        return char, None, index + 1
    if char == "'":
        if index + 2 >= len(data):
            raise RtfHtmlError("truncated hex escape")
        try:
            value = int(data[index + 1 : index + 3], 16)
        except ValueError as error:
            raise RtfHtmlError("invalid hex escape") from error
        return "hex", value, index + 3
    if not char.isalpha():
        return char, None, index + 1
    start = index
    while index < len(data) and data[index].isalpha():
        index += 1
    word = data[start:index]
    sign = 1
    if index < len(data) and data[index] == "-":
        sign = -1
        index += 1
    number_start = index
    while index < len(data) and data[index].isdigit():
        index += 1
    number = None
    if index > number_start:
        number = sign * int(data[number_start:index])
    if index < len(data) and data[index] == " ":
        index += 1
    return word, number, index


def recover_html(rtf: str) -> str:
    if not rtf.startswith("{\\rtf") or "\\fromhtml1" not in rtf:
        raise RtfHtmlError("input is not validated fromhtml1 RTF")

    output: list[str] = []
    stack: list[dict[str, object]] = [{"skip": False, "htmltag": False, "ignorable": False}]
    index = 0
    while index < len(rtf):
        char = rtf[index]
        if char == "{":
            stack.append(dict(stack[-1]))
            stack[-1]["ignorable"] = False
            index += 1
            continue
        if char == "}":
            if len(stack) == 1:
                raise RtfHtmlError("unbalanced closing brace")
            stack.pop()
            index += 1
            continue
        if char == "\\":
            word, number, index = _read_control(rtf, index)
            state = stack[-1]
            if word == "*":
                state["ignorable"] = True
            elif word == "htmltag":
                state["htmltag"] = True
                state["skip"] = False
            elif word in SKIP_DESTINATIONS or word == "htmlrtf":
                state["skip"] = True
            elif word == "par" and not state["skip"]:
                output.append("\n")
            elif word == "tab" and not state["skip"]:
                output.append("\t")
            elif word == "line" and not state["skip"]:
                output.append("\n")
            elif word == "hex" and not state["skip"] and number is not None:
                output.append(bytes([number]).decode("cp1252", errors="strict"))
            elif word in "{}\\" and not state["skip"]:
                output.append(word)
            elif state["ignorable"] and not state["htmltag"]:
                state["skip"] = True
            continue
        if not stack[-1]["skip"] and char not in "\r\n":
            output.append(char)
        index += 1

    if len(stack) != 1:
        raise RtfHtmlError("unbalanced opening brace")
    html = "".join(output).strip()
    if not html or "<" not in html or ">" not in html:
        raise RtfHtmlError("no recoverable HTML markup")
    if "\\" in html or "{\\rtf" in html:
        raise RtfHtmlError("RTF control data leaked into HTML")
    return html


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("input_rtf", type=Path)
    parser.add_argument("output_html", type=Path)
    args = parser.parse_args()
    try:
        rtf = args.input_rtf.read_text(encoding="utf-8")
        html = recover_html(rtf)
        args.output_html.parent.mkdir(parents=True, exist_ok=True)
        args.output_html.write_text(html, encoding="utf-8", newline="\n")
    except (OSError, UnicodeError, RtfHtmlError) as error:
        parser.error(str(error))
    print(f"html_bytes={len(html.encode('utf-8'))}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
