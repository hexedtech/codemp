# Python bindings
Python allows directly `import`ing properly formed shared objects, so the glue can live mostly on the Rust side.

Our Python glue is built with [`PyO3`](https://pyo3.rs).

To get a usable shared object just `cargo build --release --features=python`, however preparing a proper python package to be included as dependency requires more steps.

## `PyPI`

`codemp` is directly available on `PyPI` as [`codemp`](https://pypi.org/project/codemp).

## Building
To distribute the native extension we can leverage python wheels. It will be necessary to build the relevant wheels with [`maturin`](https://github.com/PyO3/maturin).
After installing with `pip install maturin`, run `maturin build` to obtain an `import`able package.
