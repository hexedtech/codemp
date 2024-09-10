[![codemp](https://code.mp/static/banner.png)](https://code.mp)

[![Actions Status](https://github.com/hexedtech/codemp/actions/workflows/test.yml/badge.svg)](https://github.com/hexedtech/codemp/actions)
[![docs.rs](https://img.shields.io/docsrs/codemp)](https://docs.rs/codemp/0.7.0-beta.2/codemp/)
[![Gitter](https://img.shields.io/gitter/room/hexedtech/codemp)](https://gitter.im/hexedtech/codemp)
[![Crates.io Version](https://img.shields.io/crates/v/codemp)](https://crates.io/crates/codemp)
[![NPM Version](https://img.shields.io/npm/v/codemp)](https://npmjs.org/package/codemp)
[![PyPI - Version](https://img.shields.io/pypi/v/codemp)](https://pypi.org/project/codemp)

> `codemp` is a **collaborative** text editing solution to work remotely.

It seamlessly integrates in your editor providing remote cursors and instant text synchronization,
as well as a remote virtual workspace for you and your team.

> `codemp` is build with state-of-the-art CRDT technology, guaranteeing eventual consistency.

This means that everyone is guaranteed to converge to a consistent state once all changes are received
no matter the order or the timing due to unreliable networks or constrained resources. And similarly, your
changes will always carry their original intention. On top of this baseline, `codemp`'s protocol is optimized for speed 
and low network footprint, meaning even slow connections can provide stable real-time editing.

The full documentation is available on [docs.rs](https://docs.rs/codemp/0.7.0-beta.2/codemp/).

# Usage
`codemp` is primarily used as a plugin in your editor of choice.

## Installation
> [!IMPORTANT]
> The editor plugins are in active development. Expect frequent changes.

`codemp` is available as a plugin for a growing number of text editors. Currently we support:
 - [NeoVim](https://github.com/hexedtech/codemp-nvim)
 - [VSCode](https://github.com/hexedtech/codemp-vscode)
 - [Sublime Text](https://github.com/hexedtech/codemp-sublime)
<!-- - [IntelliJ Platform](https://github.com/hexedtech/codemp-intellij) -->

## Registration
The `codemp` protocol is [openly available](https://github.com/hexedtech/codemp-proto/) and servers may be freely developed with it.

A reference instance is provided by hexed.technology at [code.mp](https://code.mp). You may create an account for it [here](https://code.mp/signup).
During the initial closed beta, registrations will require an invite code. Get in contact if interested.

An open beta is going to follow with free access to a single workspace per user.
After such period, [code.mp](https://code.mp) will switch to a subscription-based model.

# Development
This is the main client library for `codemp`. It provides a batteries-included fully-featured `Client`, managed by the library itself, and exposes a number of functions to interact with it. The host program can obtain a `Client` handle by connecting, and from that reference can retrieve every other necessary component.

`codemp` is primarily a rlib and can be used as such, but is also available in other languages via FFI.

Adding a dependency on `codemp` is **easy**:

### From Rust
Just `cargo add codemp` and check the docs for some examples.

### From supported languages
We provide first-class bindings for:
 - [![JavaScript](https://github.com/hexedtech/codemp/actions/workflows/javascript.yml/badge.svg)](./dist/js/README.md) available from `npm` as [`codemp`](https://npmjs.org/package/codemp)
 - [![Python](https://github.com/hexedtech/codemp/actions/workflows/python.yml/badge.svg)](./dist/lua/README.md) available from `PyPI` as [`codemp`](https://pypi.org/project/codemp)
 - [![Lua](https://github.com/hexedtech/codemp/actions/workflows/lua.yml/badge.svg)](./dist/lua/README.md) run `cargo build --features=lua`
 - [![Java](https://github.com/hexedtech/codemp/actions/workflows/java.yml/badge.svg)](./dist/java/README.md) run `gradle build` in `dist/java/` (requires Gradle)

As a design philosophy, our binding APIs attempt to perfectly mimic their Rust counterparts, so the main documentation can still be referenced as source of truth.
Refer to specific language documentation for specifics, differences and quirks.

### From other languages
> [!IMPORTANT]
> The common C bindings are not available yet!

Any other language with C FFI capabilities will be able to use `codemp` via its bare C bindings.
This may be more complex and may require wrapping the native calls underneath.

# Get in Touch
We love to hear back from users! Be it to give feedback, propose new features or highlight bugs, don't hesitate to reach out!

## Contacts
We have a public [Gitter](https://gitter.im) room available on [gitter.im/hexedtech/codemp](https://gitter.im/hexedtech/codemp).
It's possible to freely browse the room history, but to send new messages it will be necessary to sign in with your GitHub account.

If you have a [Matrix](https://matrix.org) account, you can join the gitter room directly at [#hexedtech_codemp:gitter.im](https://matrix.to/#/#hexedtech_codemp:gitter.im)

## Contributing
If you find bugs or would like to see new features implemented, be sure to open an issue on this repository.

In case you wish to contribute code, that's great! We love external contributions, feel free to open a PR!
