def test_constructor(runner):
    def msgid_msgstr_kwargs(polib):
        entry = polib.POEntry(
            msgid="msgid 1",
            msgstr="msgstr 1",
        )
        assert entry.msgid == "msgid 1"
        assert entry.msgstr == "msgstr 1"

    def get_set_all(polib):
        msgstr_plural = [
            "msgstr_plural 1",
            "msgstr_plural 2",
        ]
        flags = ["flag 1", "flag 2"]
        occurrences = [
            ("path/to/file1", "1"),
            ("path/to/file2", "2"),
        ]

        entry = polib.POEntry()
        entry.msgid = "msgid 1"
        entry.msgstr = "msgstr 1"
        entry.msgctxt = "msgctxt 1"
        entry.msgid_plural = "msgid_plural 1"
        entry.msgstr_plural = msgstr_plural
        entry.obsolete = True
        entry.comment = "comment 1"
        entry.tcomment = "tcomment 1"
        entry.flags = flags
        entry.previous_msgctxt = "previous_msgctxt 1"
        entry.previous_msgid = "previous_msgid 1"
        entry.previous_msgid_plural = "previous_msgid_plural 1"
        entry.occurrences = occurrences

        assert entry.msgid == "msgid 1"
        assert entry.msgstr == "msgstr 1"
        assert entry.msgctxt == "msgctxt 1"
        assert entry.msgid_plural == "msgid_plural 1"
        if polib.__name__ == "polib":
            assert entry.msgstr_plural == msgstr_plural
            assert entry.flags == flags
            assert entry.occurrences == occurrences
        else:
            assert entry.get_msgstr_plural() == msgstr_plural
            assert entry.get_flags() == flags
            assert entry.get_occurrences() == occurrences
        assert entry.obsolete
        assert entry.comment == "comment 1"
        assert entry.tcomment == "tcomment 1"
        assert entry.previous_msgctxt == "previous_msgctxt 1"
        assert entry.previous_msgid == "previous_msgid 1"
        assert entry.previous_msgid_plural == "previous_msgid_plural 1"

    runner.run(
        msgid_msgstr_kwargs,
        get_set_all,
    )


def test_methods(runner):
    def fuzzy(polib):
        entry = polib.POEntry(msgid="msgid 1", msgstr="msgstr 1")
        assert not entry.fuzzy
        entry.flags = ["fuzzy"]
        assert entry.fuzzy
        entry.flags = []
        assert not entry.fuzzy

    def to_string_with_wrapwidth(polib):
        entry = polib.POEntry(
            msgid="msgid 1 msgid 1 msgid 1",
            msgstr="msgstr 1 msgstr 1 msgstr 1",
        )
        if polib.__name__ == "rspolib":
            result = entry.to_string_with_wrapwidth(8)
        else:
            result = entry.__unicode__(wrapwidth=8)
        assert 20 > len(result.splitlines()) > 10

    runner.run(
        fuzzy,
        to_string_with_wrapwidth,
    )


def test_magic_methods(runner):
    def __cmp__(polib):
        entry1 = polib.POEntry(msgid="msgid 1", msgstr="msgstr 1")
        entry2 = polib.POEntry(msgid="msgid 1", msgstr="msgstr 1")
        entry3 = polib.POEntry(msgid="msgid 2", msgstr="msgstr 2")
        assert entry1.__cmp__(entry2) == 0
        assert entry1.__cmp__(entry3) == -1
        assert entry3.__cmp__(entry2) == 1

    runner.run(
        __cmp__,
    )
