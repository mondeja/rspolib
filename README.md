# rspolib

Port to Rust of the Python library [polib]

## Install

```bash
cargo add rspolib
```

## Usage

```rust
use rspolib::pofile;

let po = pofile("path/to/file.po").unwrap();

for entry in po.entries {
    println!("{}", entry.msgid);
}

po.save("path/to/other/file.po").unwrap();
```

[polib]: https://github.com/izimobil/polib
