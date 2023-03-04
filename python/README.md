# rspolib

[![pypi](https://img.shields.io/pypi/v/rspolib?logo=pypi&logoColor=white)](https://pypi.org/project/rspolib/) [![pyversions](https://img.shields.io/pypi/pyversions/rspolib?logo=python&logoColor=white)](https://pypi.org/project/rspolib/)

Python bindings for the Rust crate [rspolib].

## Install

```bash
pip install rspolib
```

## Usage

```python
import rspolib

po = rspolib.pofile("path/to/file.po")

for entry in po:
    print(entry.msgid)

po.save("path/to/other/file.po")
```

## Benchmarks

You can run some guidance benchmarks to compare with [polib] with:

```bash
pip install -r dev-requirements.txt
pytest -svv
```

[rspolib]: https://github.com/mondeja/rspolib
[polib]: https://github.com/izimobil/polib
