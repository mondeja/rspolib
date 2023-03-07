from rspolib import escape, unescape

ESCAPES_EXPECTED = (
    r'foo \ \\ \t \r bar \n \v \b \f " baz',
    r"foo \\ \\\\ \\t \\r bar \\n \\v \\b \\f \" baz",
)


def test_escape():
    assert escape(ESCAPES_EXPECTED[0]) == ESCAPES_EXPECTED[1]


def test_unescape():
    assert unescape(ESCAPES_EXPECTED[1]) == ESCAPES_EXPECTED[0]
