def test_parse(runner, tests_dir):
    def parse_all_features(polib):
        polib.mofile(f"{tests_dir}/all.mo")

    runner.run(
        parse_all_features,
    )


def test_format(runner, tests_dir):
    import polib
    import rspolib

    rspo = rspolib.mofile(f"{tests_dir}/all.mo")
    pypo = polib.mofile(f"{tests_dir}/all.mo")

    def format_as_string(polib):
        prefix = "#\n" if polib.__name__ == "rspolib" else ""
        assert (
            (rspo if polib.__name__ == "rspolib" else pypo)
            .__str__()
            .startswith(f'{prefix}msgid ""\n')
        )

    runner.run(
        format_as_string,
    )


def test_edit_save(runner, tests_dir, output_dir):
    def edit_save(polib):
        mo = polib.mofile(f"{tests_dir}/all.mo")
        if polib.__name__ == "rspolib":
            mo.update_metadata({"Project-Id-Version": "test"})
        else:
            mo.metadata["Project-Id-Version"] = "test"
        mo.save(f"{output_dir}/mofile_edit_save.mo")
        mo.save_as_pofile(f"{output_dir}/mofile_edit_save.po")

    runner.run(edit_save)


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
