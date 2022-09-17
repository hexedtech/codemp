# codemp
This project is heavily inspired by Microsoft Live Share plugin for Visual Studio (Code).
While the functionality is incredibly good, I often find issues or glitches which slow me down, and being locked to only use Visual Studio products is limiting.
I decided to write my own solution, and to make it open source, so that any editor can integrate it with a plugin.

# Design
## Client/Server
While initially a P2P system seemed interesting, I don't think it would easily scale with many users (due to having to integrate many changes on each client).
I decided to build a client/server architecture, with a central "Workspace" managed by the server application and multiple clients connecting to it.
Each client will only have to care about keeping itself in sync with the server (remembering local-only changes and acknowledged changes), leaving the task of keeping track of differences to the server.

## Plugins
This software will probably be distribuited as a standalone binary that editors can use to connect to a "Workspace". A dynamic library object might also be a choice.
Each editor plugin must be responsible of mapping codemp functionality to actual editor capabilities, bridging codemp client to the editor itself. The client should be able to handle a session autonomously.

## Text Synchronization
A non destructive way to sync changes across clients is necessary.
I initially explored CRDTs, but implementation seemed complex with little extra benefits from "more traditional" approaches (Operational Transforms).
This has to be investigated more.

# Roadmap
* [x] Initial design choices
* [x] Simple GRPC server with tonic
* [x] Simple neovim client with RPC/msgpack
* [ ] Implementing core protocol routes
* [ ] Simple neovim client capable of displaying other person cursor
* [ ] Implement OTs / CRTDs for sharing file deltas
* [ ] More clients (VSCode? JetBrains IDEs?)
* [ ] LSP functionality bridged to guests from host?
* [ ] Full remote development suite by keeping the project repo on a server?
