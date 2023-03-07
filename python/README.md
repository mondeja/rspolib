# rspolib

[![pypi](https://img.shields.io/pypi/v/rspolib?logo=pypi&logoColor=white)](https://pypi.org/project/rspolib/) [![pyversions](https://img.shields.io/pypi/pyversions/rspolib?logo=python&logoColor=white)](https://pypi.org/project/rspolib/)

Python bindings for the Rust crate [rspolib].

## Install

```bash
pip install rspolib
```

## Usage

### Read and save a PO file

```python
import rspolib

try:
    po = rspolib.pofile("path/to/file.po")
except rspolib.SyntaxError as e:
    print(e)
    exit(1)

for entry in po:
    print(entry.msgid)

po.save("path/to/other/file.po")
```

### Read and save a MO file

```python
import rspolib

try:
    mo = rspolib.mofile("path/to/file.mo")
except rspolib.IOError as e:
    print(e)
    exit(1)

for entry in mo:
    print(entry.msgid)

mo.save("path/to/other/file.mo")
```

[rspolib]: https://github.com/mondeja/rspolib
