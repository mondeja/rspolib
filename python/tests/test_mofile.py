def test_parse(runner, tests_dir):
    def parse_all_features(polib):
        polib.mofile(f"{tests_dir}/all.mo")

    runner.run(
        {"reps": 700},
        [
            parse_all_features,
        ],
    )


def test_format(runner, tests_dir):
    import polib
    import rspolib

    rspo = rspolib.mofile(f"{tests_dir}/all.mo")
    pypo = polib.mofile(f"{tests_dir}/all.mo")

    def format_as_string(polib):
        assert (
            (rspo if polib.__name__ == "rspolib" else pypo)
            .__str__()
            .startswith('msgid ""\n')
        )

    runner.run(
        {"reps": 500},
        [
            format_as_string,
        ],
    )


def test_edit_save(runner, tests_dir, output_dir):
    def edit_save(polib):
        mo = polib.mofile(f"{tests_dir}/all.mo")
        mo.metadata["Project-Id-Version"] = "test"
        entries = mo.entries if polib.__name__ == "rspolib" else mo
        for entry in entries:
            entry.msgstr = "test"
        mo.save(f"{output_dir}/mofile_edit_save.mo")
        mo.save_as_pofile(f"{output_dir}/mofile_edit_save.po")

    runner.run(
        {"reps": 1000},
        [
            edit_save,
        ],
    )
