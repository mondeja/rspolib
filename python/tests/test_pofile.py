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
        if polib.__name__ == "rspolib":
            po.update_metadata({"Project-Id-Version": "test"})
        else:
            po.metadata["Project-Id-Version"] = "test"
        po.save(f"{output_dir}/pofile_edit_save.po")
        po.save_as_mofile(f"{output_dir}/pofile_edit_save.mo")

    runner.run(
        edit_save,
    )


def test_setters(runner):
    import rspolib
    import polib as pypolib

    pypo = pypolib.POFile()
    rspo = rspolib.POFile()

    def _get_entries(file, polib):
        if polib.__name__ == "polib":
            return file.entries
        return file.get_entries()

    def set_entries(polib):
        entry1 = polib.POEntry(msgid="test1", msgstr="test1")
        entry2 = polib.POEntry(msgid="test2", msgstr="test2")
        entry3 = polib.POEntry(msgid="test3", msgstr="test3")
        entry4 = polib.POEntry(msgid="test4", msgstr="test4")

        po = pypo if polib.__name__ == "polib" else rspo
        po.entries = [entry1, entry2, entry3, entry4]
        assert len(_get_entries(po, polib)) == 4

        po.entries = []
        assert len(po) == 0

    runner.run(
        set_entries,
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

    def remove_metadata_field(polib):
        po = polib.pofile(f"{tests_dir}/metadata.po")
        assert "Project-Id-Version" in po.get_metadata()
        po.remove_metadata_field("Project-Id-Version")
        assert "Project-Id-Version" not in po.get_metadata()

    def update_metadata(polib):
        po = polib.pofile("")
        po.update_metadata({"Project-Id-Version": "test"})
        metadata = po.get_metadata()
        assert metadata["Project-Id-Version"] == "test"
        assert len(metadata) == 1

    runner.run(
        percent_translated,
        untranslated_entries,
        translated_entries,
        fuzzy_entries,
        find,
    )
    runner.run(
        remove_metadata_field,
        update_metadata,
        run_polib=False,
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

    def find_by_msgid_plural_polib(polib):
        entry = pypo.find("Please submit %d or fewer forms.", by="msgid_plural")
        assert entry.msgstr_plural[0] == "Por favor, envíe %d formulario o menos."

    def find_by_msgid_plural_rspolib(polib):
        entries = rspo.find("Please submit %d or fewer forms.", by="msgid_plural")
        entry = entries[0]
        assert entry.get_msgstr_plural()[0] == "Por favor, envíe %d formulario o menos."

    runner.run(
        find_by_msgid,
        find_by_msgid_msgctxt,
        (find_by_msgid_plural_polib, find_by_msgid_plural_rspolib),
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
    def __iter__(polib):
        po = polib.pofile(f"{tests_dir}/django-complete.po")
        assert hasattr(po, "__iter__")

        iterated = False
        for entry in po:
            assert entry.msgid
            iterated = True
        assert iterated

    def __len__(polib):
        po = polib.pofile(f"{tests_dir}/django-complete.po")
        assert hasattr(po, "__len__")
        assert len(po) > 320

    def __contains__(polib):
        po = polib.POFile()
        assert hasattr(po, "__contains__")

        entry = polib.POEntry(msgid="foo", msgstr="bar")
        assert entry not in po
        po.append(entry)
        assert entry in po

    def __getitem__(polib):
        po = polib.POFile()
        assert hasattr(po, "__getitem__")

        entry = polib.POEntry(msgid="foo", msgstr="bar")
        assert entry not in po
        po.append(entry)
        assert po[0].msgid == "foo"
        assert po[0].msgstr == "bar"

    runner.run(
        __iter__,
        __len__,
        __contains__,
        __getitem__,
    )


def test_metadata(runner, tests_dir):
    import rspolib
    import polib as pypolib

    pypo = pypolib.pofile(f"{tests_dir}/metadata.po")
    rspo = rspolib.pofile(f"{tests_dir}/metadata.po")

    def pypolib_metadata_get(polib):
        assert len(pypo.metadata) == 11

    def rspolib_metadata_get(polib):
        assert len(rspo.get_metadata()) == 11

    runner.run(
        (pypolib_metadata_get, rspolib_metadata_get),
    )
