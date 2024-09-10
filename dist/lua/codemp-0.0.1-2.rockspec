package = "codemp"
version = "0.0.1-2"

source = {
    url = "git+https://github.com/hexedtech/codemp",
	branch = "dev",
}

dependencies = {
	"Lua >= 5.1",
	"luarocks-build-rust-mlua",
}

description = {
    summary = "code multiplexer -- lua bindings",
    detailed = [[
		codemp is a cooperative live editing framework by hexed.technology

		it provides a batteries-included CRDT-powered client capable of keeping
		in sync local buffers and sharing cursor event to all workspace members.
		this is a native lua library, completely written in Rust
    ]],
    homepage = "https://code.mp",
    license = "GPL-3.0",
	-- labels = { "codemp", "cooperative", "ffi", "editor", "rust" },
	maintainer = "alemi <me@alemi.dev>",
}

build = {
	type = "rust-mlua",
	modules = {
		["codemp_native"] = "codemp",
	},
	target_path = "../..",
	include = {
		"codemp.lua",
		"annotations.lua",
	}
}
