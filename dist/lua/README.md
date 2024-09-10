# Lua bindings
Lua allows directly `require`ing properly constructed shared objects, so glue code can live completely on the Rust side.

The Lua-compatible wrappers are built with [`mlua`](https://github.com/mlua-rs/mlua).

To build, just `cargo build --release --features=lua` and rename the resulting `libcodemp.so` / `codemp.dll` / `codemp.dylib` in `codemp_native.so/dll/dylib`.
This is important because Lua looks up the constructor symbol based on filename.

Type hints are provided in `annotations.lua`, just include them in your language server: `---@module 'annotations'`.

## LuaRocks
`codemp` is available as a rock on [LuaRocks](https://luarocks.org/modules/alemi/codemp)

## Manual bundling
LuaRocks compiles from source, which only works if have the rust toolchain available. To provide a reasonable NeoVim experience, we provide pre-built binaries.

> Download latest build and annotations from [here](https://codemp.dev/releases/lua/)

You will need a loader file to provide annotations: you can use provided `codemp.lua`
