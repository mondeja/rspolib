def test_parse(runner, tests_dir):
    def parse_complete(polib):
        polib.pofile(f"{tests_dir}/django-complete.po")

    runner.run(
        {"reps": 100},
        [
            parse_complete,
        ],
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
        {"reps": 100},
        [
            format_as_string,
        ],
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
        {"reps": 70},
        [
            edit_save,
        ],
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

    runner.run(
        {"reps": 1000},
        [
            percent_translated,
            untranslated_entries,
            translated_entries,
            fuzzy_entries,
        ],
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

    runner.run(
        {"reps": 3000},
        [
            find_by_msgid,
            find_by_msgid_msgctxt,
        ],
    )
