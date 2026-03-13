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
local NilPromise = {}

--- block until promise is ready
function NilPromise:await() end

--- cancel promise execution
function NilPromise:cancel() end

---@param cb fun() callback to invoke
---invoke callback asynchronously as soon as promise is ready
function NilPromise:and_then(cb) end


---@class (exact) StringPromise : Promise
local StringPromise = {}

--- block until promise is ready and return value
--- @return string
function StringPromise:await() end

--- cancel promise execution
function StringPromise:cancel() end

---@param cb fun(x: string) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function StringPromise:and_then(cb) end


---@class (exact) StringArrayPromise : Promise
local StringArrayPromise = {}
--- block until promise is ready and return value
--- @return string[]
function StringArrayPromise:await() end
--- cancel promise execution
function StringArrayPromise:cancel() end
---@param cb fun(x: string[]) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function StringArrayPromise:and_then(cb) end


---@class (exact) ClientPromise : Promise
local ClientPromise = {}
--- block until promise is ready and return value
--- @return Client
function ClientPromise:await() end
--- cancel promise execution
function ClientPromise:cancel() end
---@param cb fun(x: Client) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function ClientPromise:and_then(cb) end


---@class (exact) WorkspacePromise : Promise
local WorkspacePromise = {}
--- block until promise is ready and return value
--- @return Workspace
function WorkspacePromise:await() end
--- cancel promise execution
function WorkspacePromise:cancel() end
---@param cb fun(x: Workspace) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function WorkspacePromise:and_then(cb) end


---@class (exact) WorkspaceEventPromise : Promise
local WorkspaceEventPromise = {}
--- block until promise is ready and return value
--- @return WorkspaceEvent
function WorkspaceEventPromise:await() end
--- cancel promise execution
function WorkspaceEventPromise:cancel() end
---@param cb fun(x: WorkspaceEvent) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function WorkspaceEventPromise:and_then(cb) end


---@class (exact) MaybeWorkspaceEventPromise : Promise
local MaybeWorkspaceEventPromise = {}
--- block until promise is ready and return value
--- @return WorkspaceEvent | nil
function MaybeWorkspaceEventPromise:await() end
---@param cb fun(x: WorkspaceEvent | nil) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function MaybeWorkspaceEventPromise:and_then(cb) end


---@class (exact) BufferControllerPromise : Promise
local BufferControllerPromise = {}
--- block until promise is ready and return value
--- @return BufferController
function BufferControllerPromise:await() end
--- cancel promise execution
function BufferControllerPromise:cancel() end
---@param cb fun(x: BufferController) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function BufferControllerPromise:and_then(cb) end


---@class (exact) CursorEventPromise : Promise
local CursorEventPromise = {}
--- block until promise is ready and return value
--- @return CursorEvent
function CursorEventPromise:await() end
--- cancel promise execution
function CursorEventPromise:cancel() end
---@param cb fun(x: CursorEvent) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function CursorEventPromise:and_then(cb) end


---@class (exact) MaybeCursorEventPromise : Promise
local MaybeCursorEventPromise = {}
--- block until promise is ready and return value
--- @return CursorEvent | nil
function MaybeCursorEventPromise:await() end
--- cancel promise execution
function MaybeCursorEventPromise:cancel() end
---@param cb fun(x: CursorEvent | nil) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function MaybeCursorEventPromise:and_then(cb) end


---@class (exact) BufferUpdatePromise : Promise
local BufferUpdatePromise = {}
--- block until promise is ready and return value
--- @return BufferUpdate
function BufferUpdatePromise:await() end
--- cancel promise execution
function BufferUpdatePromise:cancel() end
---@param cb fun(x: BufferUpdate) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function BufferUpdatePromise:and_then(cb) end


---@class (exact) MaybeBufferUpdatePromise : Promise
local MaybeBufferUpdatePromise = {}
--- block until promise is ready and return value
--- @return BufferUpdate | nil
function MaybeBufferUpdatePromise:await() end
--- cancel promise execution
function MaybeBufferUpdatePromise:cancel() end
---@param cb fun(x: BufferUpdate | nil) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function MaybeBufferUpdatePromise:and_then(cb) end

---@class (exact) UserInfoListPromise : Promise
local UserInfoListPromise = {}
--- block until promise is ready and return value
--- @return UserInfo[]
function UserInfoListPromise:await() end
--- cancel promise execution
function UserInfoListPromise:cancel() end
---@param cb fun(x: UserInfo[]) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function UserInfoListPromise:and_then(cb) end

---@class (exact) UserInfoPromise : Promise
local UserInfoPromise = {}
--- block until promise is ready and return value
--- @return UserInfo
function UserInfoPromise:await() end
--- cancel promise execution
function UserInfoPromise:cancel() end
---@param cb fun(x: UserInfo) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function UserInfoPromise:and_then(cb) end

---@class (exact) WorkspaceIdentifierListPromise : Promise
local WorkspaceIdentifierListPromise = {}
--- block until promise is ready and return value
--- @return WorkspaceIdentifier[]
function WorkspaceIdentifierListPromise:await() end
--- cancel promise execution
function WorkspaceIdentifierListPromise:cancel() end
---@param cb fun(x: WorkspaceIdentifier[]) callback to invoke
---invoke callback asynchronously as soon as promise is ready
function WorkspaceIdentifierListPromise:and_then(cb) end

-- [[ END ASYNC STUFF ]]


---@class (exact) Client
---the effective local client, handling connecting to codemp server
local Client = {}

---@return UserInfo
---current logged in user for this client
function Client:current_user() end

---@return string[]
---array of all currently active workspace names
function Client:active_workspaces() end

---@return NilPromise
---@async
---@nodiscard
---refresh current user token if possible
function Client:refresh() end

---@param user string workspace owning user
---@param ws string workspace id to connect to
---@return WorkspacePromise
---@async
---@nodiscard
---join requested workspace if possible and subscribe to event bus
function Client:attach_workspace(user, ws) end

---@param ws string workspace id to create
---@return NilPromise
---@async
---@nodiscard
---create a new workspace with given id
function Client:create_workspace(ws) end

---@param user string workspace owning user
---@param ws string workspace id to leave
---leave workspace with given id, detaching and disconnecting
function Client:leave_workspace(user, ws) end

---@param ws string workspace id to delete
---@return NilPromise
---@async
---@nodiscard
---delete workspace with given id
function Client:delete_workspace(ws) end

---@param user string user owning the workspace to quit
---@param workspace string workspace to quit
---@return NilPromise
---@async
---@nodiscard
---quit a joined workspace, by user + workspace name
function Client:quit_workspace(user, workspace) end

---@param user string user inviting us
---@param workspace string workspace being invited to
---@return NilPromise
---@async
---@nodiscard
---accept an invite to a new workspace
function Client:accept_invite(user, workspace) end

---@param user string user inviting us
---@param workspace string workspace being invited to
---@return NilPromise
---@async
---@nodiscard
---reject an invite to a new workspace
function Client:reject_invite(user, workspace) end

---@param ws string workspace id to delete
---@param user string user name to invite to given workspace
---@return NilPromise
---@async
---@nodiscard
---grant user acccess to workspace
function Client:invite_to_workspace(ws, user) end

---@return WorkspaceIdentifierListPromise
---@async
---@nodiscard
---fetch and list owned workspaces
function Client:fetch_owned_workspaces() end

---@return WorkspaceIdentifierListPromise
---@async
---@nodiscard
---fetch and list joined workspaces
function Client:fetch_joined_workspaces() end

---@param ws string workspace id to get
---@return Workspace?
---get an active workspace by name
function Client:get_workspace(ws) end

---@param user string username to lookup
---@return UserInfoPromise
---@async
---@nodiscard
---get full user info for given username from server
function Client:get_user_info(user) end

---@return NilPromise
---@async
---@nodiscard
---block until next session event without returning it
function Client:poll() end

---clears any previously registered session callback
function Client:clear_callback() end

---@param cb fun(w: Client) callback to invoke on each workspace event received
---register a new callback to be called on session events (replaces any previously registered one)
function Client:callback(cb) end



---@class UserInfo
---represents a service user and contains all its relevant info
---@field name string user unique, immutable name
---@field display_name string|nil user display name, mutable and not guaranteed to be unique
---@field description string|nil user description, maybe containing contact info
---@field avatar any|nil user avatar image, as bytes 

---@class WorkspaceIdentifier
---uniquely identifies a workspace, by its owner and workspace name
---@field user string username of workspace owner
---@field workspace string workspace name




---@class (exact) Workspace
---a joined codemp workspace
local Workspace = {}

---@return WorkspaceIdentifier
---workspace id
function Workspace:id() end

---@return string[]
---array of all currently active buffer names
function Workspace:active_buffers() end

---@return CursorController
---reference to workspace's CursorController
function Workspace:cursor() end

---@param path string relative path ("name") of new buffer
---@param ephemeral boolean wether this buffer is ephemeral (auto deletes)
---@return NilPromise
---@async
---@nodiscard
---create a new empty buffer
function Workspace:create_buffer(path, ephemeral) end

---@param path string relative path ("name") of buffer to delete
---@return NilPromise
---@async
---@nodiscard
---delete buffer from workspace
function Workspace:delete_buffer(path) end

---@param path string relative path ("name") of buffer to pin
---@return NilPromise
---@async
---@nodiscard
---pin a buffer, meaning it will persist even if no users are attached
function Workspace:pin_buffer(path) end

---@param path string relative path ("name") of buffer to un-pin
---@return NilPromise
---@async
---@nodiscard
---un-pin a buffer, meaning it will get deleted once all users leave
function Workspace:un_pin_buffer(path) end

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
---@return boolean success
---detach from an active buffer, closing all streams. returns false if there are still dangling references
function Workspace:detach_buffer(path) end

---@param filter? string apply a filter to the return elements
---@return string[]
---return the list of available buffers in this workspace, as relative paths from workspace root
function Workspace:search_buffers(filter) end

---@return UserInfo[]
---return all names of users currently in this workspace
function Workspace:user_list() end

---@param path string path of buffer queried for attached users
---@return UserInfo[]
---return all names of users currently attached to given buffer (by path)
function Workspace:buffer_user_list(path) end

---@return NilPromise
---@async
---@nodiscard
---force refresh buffer list from workspace
function Workspace:fetch_buffers() end

---@return NilPromise
---@async
---@nodiscard
---force refresh users list from workspace
function Workspace:fetch_users(path) end

---@param path string the buffer to look in
---@return NilPromise
---@async
---@nodiscard
---fetch the list of users in the given buffer
function Workspace:fetch_buffer_users(path) end

---@class (exact) WorkspaceEvent
---@field type string can be "UserJoin", "UserLeave" or "FileTreeUpdated"
---@field name? string present for "UserJoin" and "UserLeave"
---@field path? string present for "FileTreeUpdated"

---@return MaybeWorkspaceEventPromise
---@async
---@nodiscard
---try to receive workspace events, returning nil if none is available
function Workspace:try_recv() end

---@return WorkspaceEventPromise
---@async
---@nodiscard
---block until next workspace event and return it
function Workspace:recv() end

---@return NilPromise
---@async
---@nodiscard
---block until next workspace event without returning it
function Workspace:poll() end

---clears any previously registered workspace callback
function Workspace:clear_callback() end

---@param cb fun(w: Workspace) callback to invoke on each workspace event received
---register a new callback to be called on workspace events (replaces any previously registered one)
function Workspace:callback(cb) end





---@class (exact) BufferController
---handle to a remote buffer, for async send/recv operations
local BufferController = {}

---@class TextChange
---@field content string text content of change
---@field start_idx integer start index of change
---@field end_idx integer end index of change
local TextChange = {}

---@class (exact) BufferUpdate
---@field change TextChange text change for this delta
---@field version table<integer> CRDT version after this change
---@field hash integer? optional hash of text buffer after this change, for sync checks
local BufferUpdate = {}

---@param other string text to apply change to
---apply this text change to a string, returning the result
function TextChange:apply(other) end

---@return WorkspaceIdentifier
---returns the workspace id this buffer belongs to
function BufferController:workspace_id() end

---@return string
---returns the path this buffer belongs to
function BufferController:path() end

---@param change TextChange text change to broadcast
---update buffer with a text change; note that to delete content should be empty but not span, while to insert span should be empty but not content (can insert and delete at the same time)
function BufferController:send(change) end

---@return MaybeBufferUpdatePromise
---@async
---@nodiscard
---try to receive text changes, returning nil if none is available
function BufferController:try_recv() end

---@return BufferUpdatePromise
---@async
---@nodiscard
---block until next text change and return it
function BufferController:recv() end

---@return NilPromise
---@async
---@nodiscard
---block until next text change without returning it
function BufferController:poll() end

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

---@param version [integer] version to ack
---notify controller that this version's change has been correctly applied
function BufferController:ack(version) end




---@class (exact) CursorController
---handle to a workspace's cursor channel, allowing send/recv operations
local CursorController = {}

---a cursor selection span
---@class Selection
---@field start_row integer cursor position starting row in buffer
---@field start_col integer cursor position starting column in buffer
---@field end_row integer cursor position final row in buffer
---@field end_col integer cursor position final column in buffer

---a cursor instantaneous state
---@class Cursor
---@field buffer string path of buffer this cursor is on
---@field sel Selection[] the updated cursor selection(s)

---an event that occurred about a user's cursor
---@class CursorEvent
---@field user string user who sent this cursor
---@field cursor Cursor cursor position data

---@return WorkspaceIdentifier
---returns the workspace id this cursor controller belongs to
function CursorController:workspace_id() end

---@return CursorEvent[]
---@async
---@nodiscard
---gets the current state of all user cursors
function CursorController:list() end

---@param cursor Cursor cursor position to broadcast
---update cursor position by sending a cursor event to server
function CursorController:send(cursor) end

---@return MaybeCursorEventPromise
---@async
---@nodiscard
---try to receive cursor events, returning nil if none is available
function CursorController:try_recv() end

---@return CursorEventPromise
---@async
---@nodiscard
---block until next cursor event and return it
function CursorController:recv() end

---@return NilPromise
---@async
---@nodiscard
---block until next cursor event without returning it
function CursorController:poll() end

---clears any previously registered cursor callback
function CursorController:clear_callback() end

---@param cb fun(c: CursorController) callback to invoke on each cursor event from server
---register a new callback to be called on cursor events (replaces any previously registered one)
function CursorController:callback(cb) end




---@class Config
---@field username string user identifier used to register, possibly your email
---@field password string user password chosen upon registration
---@field host string | nil address of server to connect to, default api.code.mp
---@field port integer | nil port to connect to, default 50053
---@field tls boolean | nil enable or disable tls, default true

---@class Codemp
---the codemp shared library
local Codemp = {}

---@param config Config configuration for
---@return ClientPromise
---@async
---@nodiscard
---connect to codemp server, authenticate and return client
function Codemp.connect(config) end

---@return function, any | nil
---@nodiscard
---check if codemp thread sent a callback to be run on main thread
function Codemp.poll_callback() end

---@param data string
---@return integer
---use xxh3 hash, returns an i64 from any string
function Codemp.hash(data) end

---@return string
---get current library version as string, in semver format
function Codemp.version() end

---@class (exact) RuntimeDriver
local RuntimeDriver = {}

---@return boolean
---stops the runtime thread, returns false if driver was already stopped
function RuntimeDriver:stop() end

---@param block? boolean block current thread if true, otherwise spawn a background thread
---@return RuntimeDriver | nil
---spawns a background thread and uses it to run the codemp runtime
---returns the driver handle only if another thread has been spawned (block=true)
function Codemp.setup_driver(block) end

---@param printer? string | fun(string) | nil log sink used for printing, if string will go to file, otherwise use given function
---@param debug? boolean show more verbose debug logs, default false
---@return boolean success if logger was setup correctly, false otherwise
---setup a global logger for codemp, note that can only be done once
function Codemp.setup_tracing(printer, debug) end

