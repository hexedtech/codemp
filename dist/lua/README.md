# Lua bindings
Lua allows directly `require`ing properly constructed shared objects, so glue code can live completely on the Rust side.

The Lua-compatible wrappers are built with [`mlua`](https://github.com/mlua-rs/mlua).

To build, just `cargo build --release --features=lua` and rename the resulting `libcodemp.so` / `codemp.dll` / `codemp.dylib` in `codemp_native.so/dll/dylib`.
This is important because Lua looks up the constructor symbol based on filename.

Type hints are provided in `annotations.lua`, just include them in your language server: `---@module 'annotations'`.

## Example loader
A simple loader is provided here:

```lua
---@module 'annotations'

---@return Codemp
local function load()
	local native, _ = require("codemp.native")
	return native
end

return {
	load = load,
}
```

