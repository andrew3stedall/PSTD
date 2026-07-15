from scripts.rtf_fromhtml_to_html import RtfHtmlError, recover_html


def test_recovers_html_tags_and_visible_text():
    rtf = (
        r"{\rtf1\ansi\fromhtml1"
        r"{\*\htmltag <html><body>}"
        r"{\*\htmltag <b>}This line is in bold.{\*\htmltag </b>}\par "
        r"{\*\htmltag <font color=blue>}This line is in blue color"
        r"{\*\htmltag </font></body></html>}}"
    )
    html = recover_html(rtf)
    assert "<b>This line is in bold.</b>" in html
    assert "<font color=blue>This line is in blue color</font>" in html
    assert "\\rtf" not in html


def test_skips_rtf_metadata_destinations():
    rtf = (
        r"{\rtf1\ansi\fromhtml1"
        r"{\fonttbl{\f0 Arial;}}"
        r"{\colortbl;\red0\green0\blue255;}"
        r"{\*\htmltag <p>}Readable{\*\htmltag </p>}}"
    )
    html = recover_html(rtf)
    assert html == "<p>Readable</p>"
    assert "Arial" not in html


def test_rejects_non_fromhtml_rtf_and_unbalanced_input():
    for value in (r"{\rtf1 plain}", r"{\rtf1\fromhtml1{\*\htmltag <b>}broken"):
        try:
            recover_html(value)
        except RtfHtmlError:
            pass
        else:
            raise AssertionError("invalid input was accepted")


def test_rejects_markup_free_output():
    try:
        recover_html(r"{\rtf1\fromhtml1 plain text}")
    except RtfHtmlError:
        pass
    else:
        raise AssertionError("markup-free output was accepted")
