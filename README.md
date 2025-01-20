# rspolib

[![crates.io](https://img.shields.io/crates/v/rspolib?logo=rust)](https://crates.io/crates/rspolib) [![PyPI](https://img.shields.io/pypi/v/rspolib?logo=pypi&logoColor=white)](https://pypi.org/project/rspolib) [![docs.rs](https://img.shields.io/docsrs/rspolib?logo=docs.rs)](https://docs.rs/rspolib) [![Bindings docs](https://img.shields.io/badge/bindings-docs-blue?logo=python&logoColor=white)](https://github.com/mondeja/rspolib/blob/master/python/REFERENCE.md)

Port to Rust of the Python library [polib].

## Install

```bash
cargo add rspolib
```

## Usage

```rust
use rspolib::{pofile, prelude::*};

let po = pofile("./tests-data/flags.po").unwrap();

for entry in &po.entries {
    println!("{}", entry.msgid);
}

po.save("./file.po");
```

See the documentation at [docs.rs/rspolib](https://docs.rs/rspolib)

## Python bindings

[![Python versions](https://img.shields.io/pypi/pyversions/rspolib?logo=python&logoColor=white)](https://pypi.org/project/rspolib/#files)

- [Quickstart](https://github.com/mondeja/rspolib/tree/master/python#readme)
- [Reference](https://github.com/mondeja/rspolib/blob/master/python/REFERENCE.md)

### Usage

```python
import polib
import rspolib

rspo = rspolib.pofile(f"{tests_dir}/django-complete.po")
pypo = polib.pofile(f"{tests_dir}/django-complete.po")
```

[polib]: https://github.com/izimobil/polib
