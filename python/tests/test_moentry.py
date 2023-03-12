def test_constructor(runner):
    def msgid_msgstr_kwargs(polib):
        entry = polib.MOEntry(
            msgid="msgid 1",
            msgstr="msgstr 1",
        )
        assert entry.msgid == "msgid 1"

    def msgid_plural_polib(polib):
        entry = polib.MOEntry(
            msgid="msgid 1",
            msgid_plural="msgid_plural 1",
            msgstr_plural=[
                "msgstr_plural 1",
                "msgstr_plural 2",
            ],
        )
        assert entry.msgid == "msgid 1"
        assert entry.msgid_plural == "msgid_plural 1"
        assert entry.msgstr_plural == [
            "msgstr_plural 1",
            "msgstr_plural 2",
        ]

    def msgid_plural_rspolib(polib):
        entry = polib.MOEntry(
            msgid="msgid 1",
            msgid_plural="msgid_plural 1",
            msgstr_plural=[
                "msgstr_plural 1",
                "msgstr_plural 2",
            ],
        )
        assert entry.msgid == "msgid 1"
        assert entry.msgid_plural == "msgid_plural 1"
        assert entry.get_msgstr_plural() == [
            "msgstr_plural 1",
            "msgstr_plural 2",
        ]

    def get_set_all(polib):
        entry = polib.MOEntry()
        entry.msgid = "msgid 1"
        entry.msgid_plural = "msgid_plural 1"

        msgstr_plural = [
            "msgstr_plural 1",
            "msgstr_plural 2",
        ]
        entry.msgstr_plural = msgstr_plural
        entry.msgstr = "msgstr 1"
        entry.msgctxt = "msgctxt 1"
        assert entry.msgid == "msgid 1"
        assert entry.msgid_plural == "msgid_plural 1"
        if polib.__name__ == "polib":
            assert entry.msgstr_plural == msgstr_plural
        else:
            assert entry.get_msgstr_plural() == msgstr_plural
        assert entry.msgstr == "msgstr 1"
        assert entry.msgctxt == "msgctxt 1"

    runner.run(
        msgid_msgstr_kwargs,
        (msgid_plural_polib, msgid_plural_rspolib),
        get_set_all,
    )
