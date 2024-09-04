# JavaScript bindings
NodeJS allows directly `require`ing properly formed shared objects, so the glue can live mostly on the Rust side.

Our JavaScript glue is built with [`napi`](https://napi.rs).

To get a usable shared object just `cargo build --release --features=js`, however preparing a proper javascript package to be included as dependency requires more steps.

## `npm`

`codemp` is directly available on `npm` as [`codemp`](https://npmjs.org/package/codemp).

## Building

To build a node package, `napi-cli` must first be installed: `npm install napi-cli`.

You can then `npx napi build` in the project root to compile the native extension and create the type annotations (`index.d.ts`).
A package.json is provided for publishing, but will require some tweaking.

