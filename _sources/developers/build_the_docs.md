# Build The Docs

The main documentation is built using jupyter-book which you can install with pip:

```bash
pip install jupyter-book
```

Then, to build the docs, run the following command from the root of the repository:

```bash
jupyter-book build docs/
```

This, build build the docs as html and place them in `docs/_build/html/`. You can then view the docs by opening `docs/_build/html/index.html` in your browser.

## Rust API Docs

To build the rust api docs you can do:

```bash
cd rust/
cargo doc --no-deps --open
```
