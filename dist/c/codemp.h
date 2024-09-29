#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <ostream>
#include <new>

/// A `codemp` client handle.
///
/// It generates a new UUID and stores user credentials upon connecting.
///
/// A new [`Client`] can be obtained with [`Client::connect`].
struct Client;

/// A currently active shared development environment
///
/// Workspaces encapsulate a working environment: cursor positions, filetree, user list
/// and more. Each holds a [cursor::Controller] and a map of [buffer::Controller]s.
/// Using a workspace handle, it's possible to receive events (user join/leave, filetree updates)
/// and create/delete/attach to new buffers.
struct Workspace;

extern "C" {

Client *Codemp_Client_connect();

Workspace *Codemp_Client_join_workspace(Client *client, char *workspace);

} // extern "C"
