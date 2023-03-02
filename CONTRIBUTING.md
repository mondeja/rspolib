# Contributing

## Rust library

### Setup

```bash
pip install -U pre-commit
pre-commit install
```

### Run tests

```bash
cd rust
cargo test
```

### Code coverage

```bash
cd rust
CARGO_INCREMENTAL=0 RUSTFLAGS='-Cinstrument-coverage' LLVM_PROFILE_FILE='coverage-%p-%m.profraw' cargo test
mkdir -p ../target/coverage
grcov . --binary-path ../target/debug/deps/ -s . -t html --branch --ignore-not-existing --ignore '../*' --ignore "/*" -o ../target/coverage/html
python3 -m http.server -d ../target/coverage/html
```

## Python bindings

### Setup

```bash
pip install -U pre-commit
pre-commit install
cd python
python3 -m virtualenv venv
source venv/bin/activate
pip install -r dev-requirements.txt
```

### Run benchmarks and tests

```bash
cd python
maturin develop --release && python3 test.py
```
