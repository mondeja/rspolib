import rspolib

import pytest


def test_syntax_error(tests_dir):
    path = f"{tests_dir}/unescaped-double-quote-msgid.po"
    with pytest.raises(rspolib.SyntaxError, match="unescaped double quote found"):
        rspolib.pofile(path)


def test_io_error(tests_dir):
    path = f"{tests_dir}/invalid-version-number.mo"
    with pytest.raises(
        rspolib.IOError, match="Invalid mo file, expected revision number 0 or 1"
    ):
        rspolib.mofile(path)
