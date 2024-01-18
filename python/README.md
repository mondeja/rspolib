# rspolib

[![pypi](https://img.shields.io/pypi/v/rspolib?logo=pypi&logoColor=white)](https://pypi.org/project/rspolib/) [![Bindings docs](https://img.shields.io/badge/bindings-docs-blue?logo=python&logoColor=white)](https://github.com/mondeja/rspolib/blob/master/python/REFERENCE.md)

Python bindings for the Rust crate [rspolib]. Check the [reference](https://github.com/mondeja/rspolib/blob/master/python/REFERENCE.md) for more information.

## Install

[![pyversions](https://img.shields.io/pypi/pyversions/rspolib?logo=python&logoColor=white)](https://pypi.org/project/rspolib/)

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
