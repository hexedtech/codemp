[![codemp](https://codemp.dev/static/banner.png)](https://codemp.dev)

> `codemp` is a **collaborative** text editing solution to work remotely.

It seamlessly integrates in your editor providing remote cursors and instant text synchronization,
as well as a remote virtual workspace for you and your team.

> `codemp` is build with state-of-the-art CRDT technology, guaranteeing eventual consistency.

This means everyone in a workspace will always be working on the exact same file _eventually_:
even under unreliable networks or constrained resources, the underlying CRDT will always reach a 
convergent state across all users. Even with this baseline, `codemp`'s protocol is optimized for speed 
and low network footprint, meaning even slow connections can provide stable real-time editing.

The full documentation is available on [docs.rs](https://docs.rs/codemp).

# Usage
`codemp` is primarily used as a plugin in your editor of choice.

## Installation
> [!WARNING]
> The editor plugins are in active development. Expect frequent changes.

`codemp` is available as a plugin for a growing number of text editors. Currently we support:
 - [NeoVim](https://github.com/hexedtech/codemp-nvim)
 - [VSCode](https://github.com/hexedtech/codemp-vscode)
 - [Sublime Text](https://github.com/hexedtech/codemp-sublime)
<!-- - [IntelliJ Platform](https://github.com/hexedtech/codemp-intellij) -->

## Registration
The `codemp` protocol is [openly available](https://github.com/hexedtech/codemp-proto/) and servers may be freely developed with it.

A reference instance is provided by hexed.technology at [codemp.dev](https://codemp.dev). You may create an account for it [here](https://codemp.dev/register).

During the initial closed beta, registrations will require an invite code. Get in contact if interested.

An open beta is going to follow with free access to a single workspace. After the open beta period, the [codemp.dev] will switch to a subscription-based model.

# Development
This is the main client library for `codemp`. It provides a batteries-included fully-featured `Client`, managed by the library itself, and exposes a number of functions to interact with it. The host program can obtain a `Client` handle by connecting, and from that reference can retrieve every other necessary component.

`codemp` is primarily a rlib and can be used as such, but is also available in other languages via FFI.

Adding a dependency on `codemp` is **easy**:

### From Rust
Just `cargo add codemp` and check the docs for some examples.

### From supported languages
We provide first-class bindings for:
 - [JavaScript](./dist/js/README.md): available from `npm` as [`codemp`](https://npmjs.org/package/codemp)
 - [Python](./dist/lua/README.md): available from `PyPI` as [`codemp`](https://pypi.org/project/codemp)
 - [Lua](./dist/lua/README.md): run `cargo build --features=lua`
 - [Java](./dist/java/README.md): run `gradle build` in `dist/java/` (requires Gradle)

As a design philosophy, our binding APIs attempt to perfectly mimic their Rust counterparts, so the main documentation can still be referenced as source of truth.
Refer to specific language documentation for specifics, differences and quirks.

### From other languages
> [!WARNING]
> The common C bindings are not available yet!

Any other language with C FFI capabilities will be able to use `codemp` via its bare C bindings.
This may be more complex and may require wrapping the native calls underneath.

# Contributing
If you find bugs or would like to see new features implemented, be sure to open an issue on this repository.

> [!WARNING]
> The CLA necessary for code contributions is not yet available!

In case you wish to contribute code, that's great! We love external contributions, but we require you to sign our CLA first (available soon).
