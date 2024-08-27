---@meta annotations
-- type annotations for codemp native lua library

-- [[ ASYNC STUFF ]]

-- TODO lua-language-server doesn't seem to support generic classes
--      https://github.com/LuaLS/lua-language-server/issues/1532
--      so we need to expand every possible promise type...
-- 
--      do you have a better idea? send a PR our way!

---@class (exact) Promise
---@field ready boolean true if promise completed

---@class (exact) NilPromise : Promise
---@field await fun(self: NilPromise): nil block until promise is ready

---@class (exact) StringPromise : Promise
---@field await fun(self: StringPromise): string block until promise is ready and return value

---@class (exact) StringArrayPromise : Promise
---@field await fun(self: StringArrayPromise): string[] block until promise is ready and return value

---@class (exact) ClientPromise : Promise
---@field await fun(self: ClientPromise): Client block until promise is ready and return value

---@class (exact) WorkspacePromise : Promise
---@field await fun(self: WorkspacePromise): Workspace block until promise is ready and return value

---@class (exact) WorkspaceEventPromise : Promise
---@field await fun(self: WorkspaceEventPromise): WorkspaceEvent block until promise is ready and return value

---@class (exact) BufferControllerPromise : Promise
---@field await fun(self: BufferControllerPromise): BufferController block until promise is ready and return value

---@class (exact) CursorPromise : Promise
---@field await fun(self: CursorPromise): Cursor block until promise is ready and return value

---@class (exact) MaybeCursorPromise : Promise
---@field await fun(self: MaybeCursorPromise): Cursor? block until promise is ready and return value

---@class (exact) TextChangePromise : Promise
---@field await fun(self: TextChangePromise): TextChange block until promise is ready and return value

---@class (exact) MaybeTextChangePromise : Promise
---@field await fun(self: MaybeTextChangePromise): TextChange? block until promise is ready and return value

-- [[ END ASYNC STUFF ]]


---@class (exact) Client
---@field id string uuid of local user
---@field username string name of local user
---@field active_workspaces string[] array of all currently active workspace names
---the effective local client, handling connecting to codemp server
local Client = {}

---@return NilPromise
---@async
---@nodiscard
---refresh current user token if possible
function Client:refresh() end

---@param ws string workspace id to connect to
---@return WorkspacePromise
---@async
---@nodiscard
---join requested workspace if possible and subscribe to event bus
function Client:join_workspace(ws) end

---@param ws string workspace id to create
---@return NilPromise
---@async
---@nodiscard
---create a new workspace with given id
function Client:create_workspace(ws) end

---@param ws string workspace id to leave
---leave workspace with given id, detaching and disconnecting
function Client:leave_workspace(ws) end

---@param ws string workspace id to delete
---@return NilPromise
---@async
---@nodiscard
---delete workspace with given id
function Client:delete_workspace(ws) end

---@param ws string workspace id to delete
---@param user string user name to invite to given workspace
---@return NilPromise
---@async
---@nodiscard
---grant user acccess to workspace
function Client:invite_to_workspace(ws, user) end

---@param owned boolean? list owned workspaces, default true
---@param invited boolean? list invited workspaces, default true
---@return StringArrayPromise
---@async
---@nodiscard
---grant user acccess to workspace
function Client:list_workspaces(owned, invited) end

---@param ws string workspace id to get
---@return Workspace?
---get an active workspace by name
function Client:get_workspace(ws) end



---@class (exact) Workspace
---@field name string workspace name
---@field cursor CursorController workspace cursor controller
---@field active_buffers string[] array of all currently active buffer names
---a joined codemp workspace
local Workspace = {}

---@param path string relative path ("name") of new buffer
---@return NilPromise
---@async
---@nodiscard
---create a new empty buffer
function Workspace:create_buffer(path) end

---@param path string relative path ("name") of buffer to delete
---@return NilPromise
---@async
---@nodiscard
---delete buffer from workspace
function Workspace:delete_buffer(path) end

---@param path string relative path ("name") of buffer to get
---@return BufferController?
---get an active buffer controller by name
function Workspace:get_buffer(path) end

---@param path string relative path ("name") of buffer to attach to
---@return BufferControllerPromise
---@async
---@nodiscard
---attach to a remote buffer, synching content and changes and returning its controller
function Workspace:attach_buffer(path) end

---@param path string relative path ("name") of buffer to detach from
---@return boolean
---detach from an active buffer, closing all streams. returns false if buffer was no longer active
function Workspace:detach_buffer(path) end

---@param filter? string only return elements starting with given filter
---@return string[]
---return the list of available buffers in this workspace, as relative paths from workspace root
function Workspace:filetree(filter) end

---@return NilPromise
---@async
---@nodiscard
---force refresh buffer list from workspace
function Workspace:fetch_buffers(path) end

---@return NilPromise
---@async
---@nodiscard
---force refresh users list from workspace
function Workspace:fetch_users(path) end

---@class (exact) WorkspaceEvent
---@field type string
---@field value string

---@return WorkspaceEventPromise
---@async
---@nodiscard
---get next workspace event
function Workspace:event() end




---@class (exact) BufferController
---handle to a remote buffer, for async send/recv operations
local BufferController = {}

---@class (exact) TextChange
---@field content string text content of change
---@field first integer start index of change
---@field last integer end index of change
---@field hash integer? optional hash of text buffer after this change, for sync checks
---@field apply fun(self: TextChange, other: string): string apply this text change to a string

---@param first integer change start index
---@param last integer change end index
---@param content string change content
---@return NilPromise
---@async
---@nodiscard
---update buffer with a text change; note that to delete content should be empty but not span, while to insert span should be empty but not content (can insert and delete at the same time)
function BufferController:send(first, last, content) end

---@return MaybeTextChangePromise
---@async
---@nodiscard
---try to receive text changes, returning nil if none is available
function BufferController:try_recv() end

---@return TextChangePromise
---@async
---@nodiscard
---block until next text change and return it
function BufferController:recv() end

---@return NilPromise
---@async
---@nodiscard
---block until next text change without returning it
function BufferController:poll() end

---@return boolean
---stop buffer worker and disconnect, returns false if was already stopped
function BufferController:stop() end

---clears any previously registered buffer callback
function BufferController:clear_callback() end

---@param cb fun(c: BufferController) callback to invoke on each text change from server
---register a new callback to be called on remote text changes (replaces any previously registered one)
function BufferController:callback(cb) end

---@return StringPromise
---@async
---@nodiscard
---get current content of buffer controller, marking all pending changes as seen
function BufferController:content() end




---@class (exact) CursorController
---handle to a workspace's cursor channel, allowing send/recv operations
local CursorController = {}

---@class (exact) RowCol
---@field row integer row number
---@field col integer column number
---row and column tuple

---@class (exact) Cursor
---@field user string? id of user owning this cursor
---@field buffer string relative path ("name") of buffer on which this cursor is
---@field start RowCol cursor start position
---@field finish RowCol cursor end position
---a cursor position

---@param buffer string buffer relative path ("name") to send this cursor on
---@param start_row integer cursor start row
---@param start_col integer cursor start col
---@param end_row integer cursor end row
---@param end_col integer cursor end col
---@return NilPromise
---@async
---@nodiscard
---update cursor position by sending a cursor event to server
function CursorController:send(buffer, start_row, start_col, end_row, end_col) end


---@return MaybeCursorPromise
---@async
---@nodiscard
---try to receive cursor events, returning nil if none is available
function CursorController:try_recv() end

---@return CursorPromise
---@async
---@nodiscard
---block until next cursor event and return it
function CursorController:recv() end

---@return NilPromise
---@async
---@nodiscard
---block until next cursor event without returning it
function CursorController:poll() end

---@return boolean
---stop cursor worker and disconnect, returns false if was already stopped
function CursorController:stop() end

---clears any previously registered cursor callback
function CursorController:clear_callback() end

---@param cb fun(c: CursorController) callback to invoke on each cursor event from server
---register a new callback to be called on cursor events (replaces any previously registered one)
function CursorController:callback(cb) end



---@class (exact) Codemp
---the codemp shared library
local Codemp = {}

---@param host string server host to connect to
---@param username string username used to log in (usually email)
---@param password string password used to log in
---@return ClientPromise
---@async
---@nodiscard
---connect to codemp server, authenticate and return client
function Codemp.connect(host, username, password) end

---@param data string
---@return integer
---use xxh3 hash, returns an i64 from any string
function Codemp.hash(data) end

---@class (exact) RuntimeDriver
---@field stop fun(): boolean stops the runtime thread without deleting the runtime itself, returns false if driver was already stopped

---@return RuntimeDriver
---spawns a background thread and uses it to run the codemp runtime
function Codemp.spawn_runtime_driver() end

---@param printer string | fun(string) log sink used for printing, if string will go to file, otherwise use given function
---@param debug boolean? show more verbose debug logs, default false
---@return boolean true if logger was setup correctly, false otherwise
---setup a global logger for codemp, note that can only be done once
function Codemp.logger(printer, debug) end
