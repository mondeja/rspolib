def test_parse(runner, tests_dir):
    def parse_complete(polib):
        polib.pofile(f"{tests_dir}/django-complete.po")

    runner.run(
        parse_complete,
    )


def test_format(runner, tests_dir):
    import polib
    import rspolib

    rspo = rspolib.pofile(f"{tests_dir}/django-complete.po")
    pypo = polib.pofile(f"{tests_dir}/django-complete.po")

    def format_as_string(polib):
        assert (
            (rspo if polib.__name__ == "rspolib" else pypo)
            .__str__()
            .startswith("# This file is distributed")
        )

    runner.run(
        format_as_string,
    )


def test_edit_save(runner, tests_dir, output_dir):
    def edit_save(polib):
        po = polib.pofile(f"{tests_dir}/django-complete.po")
        po.metadata["Project-Id-Version"] = "test"
        entries = po.entries if polib.__name__ == "rspolib" else po
        for entry in entries:
            entry.msgstr = "test"
        po.save(f"{output_dir}/pofile_edit_save.po")
        po.save_as_mofile(f"{output_dir}/pofile_edit_save.mo")

    runner.run(
        edit_save,
    )


def test_methods(runner, tests_dir):
    def percent_translated(polib):
        po = polib.pofile(f"{tests_dir}/2-translated-entries.po")
        assert po.percent_translated() == 40

    def untranslated_entries(polib):
        po = polib.pofile(f"{tests_dir}/2-translated-entries.po")
        assert len(po.untranslated_entries()) == 3

    def translated_entries(polib):
        po = polib.pofile(f"{tests_dir}/2-translated-entries.po")
        assert len(po.translated_entries()) == 2

    def fuzzy_entries(polib):
        po = polib.pofile(f"{tests_dir}/fuzzy-no-fuzzy.po")
        assert len(po.fuzzy_entries()) == 1

    def find(polib):
        po = polib.pofile(f"{tests_dir}/flags.po")
        entry = po.find("msgstr 5", by="msgstr")
        if polib.__name__ == "rspolib":
            entry = entry[0]
        assert entry.msgid == "msgid 5"

    runner.run(
        percent_translated,
        untranslated_entries,
        translated_entries,
        fuzzy_entries,
        find,
    )


def test_find_entry(runner, tests_dir):
    import polib
    import rspolib

    pypo = polib.pofile(f"{tests_dir}/django-complete.po")
    rspo = rspolib.pofile(f"{tests_dir}/django-complete.po")

    def find_by_msgid(polib):
        if polib.__name__ == "rspolib":
            entry = rspo.find_by_msgid("Get started with Django")
        else:
            entry = pypo.find("Get started with Django")
        assert entry.msgstr == "Comienza con Django"

    def find_by_msgid_msgctxt(polib):
        if polib.__name__ == "rspolib":
            entry = rspo.find_by_msgid_msgctxt(
                "July",
                "abbrev. month",
            )
        else:
            entry = pypo.find(
                "July",
                msgctxt="abbrev. month",
            )
        assert entry.msgstr == "Jul."

    def find_by_msgid_plural(polib):
        if polib.__name__ == "rspolib":
            entries = rspo.find("Please submit %d or fewer forms.", by="msgid_plural")
            entry = entries[0]
        else:
            entry = pypo.find("Please submit %d or fewer forms.", by="msgid_plural")
        assert entry.msgstr_plural[0] == "Por favor, envíe %d formulario o menos."

    runner.run(
        find_by_msgid,
        find_by_msgid_msgctxt,
        find_by_msgid_plural,
    )


def test_remove_entry(runner, tests_dir):
    import rspolib
    import polib as pypolib

    pypo = pypolib.pofile(f"{tests_dir}/django-complete.po")
    rspo = rspolib.pofile(f"{tests_dir}/django-complete.po")
    msgid = "This is not a valid IPv6 address."

    def remove_entry(polib):
        po = pypo if polib.__name__ == "polib" else rspo
        first_len = len(po)
        entry = po.find(msgid, by="msgid", include_obsolete_entries=False)
        if polib.__name__ == "rspolib":
            entry = entry[0]
        po.remove(entry)
        assert len(po) == first_len - 1

        not_entry = po.find(msgid, by="msgid", include_obsolete_entries=False)
        if polib.__name__ == "rspolib":
            assert not_entry == []
        else:
            assert not_entry is None

        po.append(entry)

    runner.run(
        remove_entry,
    )


def test_magic_methods(runner, tests_dir):
    def iter__(polib):
        po = polib.pofile(f"{tests_dir}/django-complete.po")
        assert hasattr(po, "__iter__")

        iterated = False
        for entry in po:
            assert entry.msgid
            iterated = True
        assert iterated

    def len__(polib):
        po = polib.pofile(f"{tests_dir}/django-complete.po")
        assert hasattr(po, "__len__")
        assert len(po) > 320

    runner.run(
        iter__,
        len__,
    )


def test_metadata(runner, tests_dir):
    import rspolib
    import polib as pypolib

    pypo = pypolib.pofile(f"{tests_dir}/metadata.po")
    rspo = rspolib.pofile(f"{tests_dir}/metadata.po")

    def pypolib_metadata_get(polib):
        assert len(pypo.metadata) == 11

    def rspolib_metadata_get(polib):
        assert len(rspo.metadata) == 11

    runner.run(
        (pypolib_metadata_get, rspolib_metadata_get),
    )
