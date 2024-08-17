# codemp

<a href="https://codemp.dev"><img alt="codemp logo" align="center" src="https://codemp.dev/codemp-t.png" height="100" /></a>

### code multiplexer

> CodeMP is a **collaborative** text editing plugin to work remotely.
It seamlessly integrates in your editor providing remote cursors and instant text synchronization,
as well as a remote virtual workspace for you and your team.

> CodeMP is build with state-of-the-art CRDT technology, guaranteeing eventual consistency.
This means everyone in a workspace will always be working on the exact same file _eventually_:
even under unreliable networks or constrained resources, the underlying CRDT will always reach a 
convergent state across all users. Even with this baseline, CodeMP's proto is optimized for speed 
and low network footprint, meaning even slow connections can provide stable real-time editing.

# using this project
CodeMP is available for many editors as plugins.

Currently we support:
 - [VSCode](https://github.com/hexedtech/codemp-vscode)
 - [Intellij](https://github.com/hexedtech/codemp-intellij)
 - [NeoVim](https://github.com/hexedtech/codemp-nvim)
 - [Sublime Text](https://github.com/hexedtech/codemp-sublime)

# using this library
This is the main client library for codemp. It exposes functions to interact with the codemp client itself, its workspaces and buffers.

All memory is managed by the library itself, which gives out always atomic reference-counted pointers to internally mutable objects. The host program needs only to connect a client first, and from that reference can retrieve every other necessary component.

### from rust
This library is primarily a rust crate, so rust applications will get the best possible integration.

Just `cargo add codemp` and check the docs for some examples.

### from supported languages
This library provides first-class bindings for:
 - java
 - javascript
 - python
 - lua

For any of these languages, just add `codemp` as a dependency in your project.

The API should perfectly mimic what rust exposes underneath, so the main rust docs can still be used as reference for available methods and objects.

### from other languages
> [!WARNING]
> The common C bindings are still not available

Any other language with C ffi capabilities can use codemp via its bare C bindings.
This will be more complex and may require wrapping the native calls underneath.

# documentation
This project is mainly a rust crate, so the most up-to-date and extended documentation will be found on docs.rs.
 - Check [docs.rs/codemp](https://docs.rs/codemp) for our full documentation!

# architecture
CodeMP is built from scratch to guarantee impeccable performance and accuracy.
The following architectural choices are driven by this very strict requirement.

## interop: FFI
The first challenge of developing such a system is adoption: getting all your colleagues to switch to your editor is not going to happen. Supporting a multitude of plugins in different languages and possibly different architectures however is a daunting task even for larger teams.

Our solution is a single common native library, developed in safe and performant Rust, which can be used by any plugin with a thin layer of glue code to provide native bindings.

This allows us to maintain a single client codebase and multiple plugins, rather than multiple clients and plugins, with the cost of FFI complexity.

We took a gamble which paid off: our team was capable enough to handle cross compiling and multiple bindings, and can now focus on first-class integration in each editor API.

## synchronization: CRDT
Our investigations in the field of text synchronization for multi agent editing showed that there are mostly two approached to solve the problem: Operational Transforms (older, more used) and Conflict-free Replicated Data Structures (CRDTs, a newer technology)

While initial prototypes used OT to achieve syncrhonization, we quickly found issues. The editor is not under our plugin's control, and could always apply new insertions/deletions while processing remote changes. This was a huge issue with OTs, as it would require control over the integration process.

We introduced CRDTs first with a hand-crafted naive approach, and were very impressed by the results. Because of the nature of CRDTs, we have an internal state which is always kept in sync with the server (and all other peers), and this state can then be finely synchronized with the effective editor state. Edits coming while integrating just branch more, and our inner CRDT merges those seemlessly.

We recently swapped our internal library for a production-grade solution: [diamond-types](https://github.com/josephg/diamond-types), with even more impressive results: we jumped from processing ~2 thousand operations per second to an astonishing **~8 million**, a `1000x` improvement!

## layout: star (client/server)
Network layout posed a challenging decision: a distributed system could provide lower latency but a centralized arbiter could dramatically reduce necessary resources for each peer.

We want codemp to be a viable solution on low power devices in unreliable networks, so opted to a centralized approach.

While for small work groups the benefits are negligible, bigger sessions dramatically benefit from having a central server which handles reduntant merging and skips irrelevant operations, while masking IPs and removing the problem of punching through NATs.

We hope to provide a solution capable of scaling to hundreds or thousands of concurrent users, in order to open new uses in conferences, competitions, teaching and live entertainment.

## protocol: streams (grpc)
The underlying network structure is really important to achieve good performance. We need a binary stream to quickly beam back and forth operations.

GRPC provides this, encapsulating is convenient to use primitives, while also providing request/response procedures.

We plan to experiment with laminar and capnproto for the fast cursor and operation streams, but we will probably retain an http-based approach for workspace management and authentication.

# contributing
> [!NOTE]
> This project is maintained by [hexedtech](https://hexed.technology).

If you find bugs or would like to see new features implemented, be sure to open an issue on this repository.

In case you wished to contribute code, that's great! We love external contributions, but we require you to **sign our CLA first** (which is not yet ready, TODO!)
