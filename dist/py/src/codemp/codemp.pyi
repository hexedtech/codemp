from typing import Tuple, Optional, Callable

class Driver:
	"""
	this is akin to a big red button with a white "STOP" on top of it.
	it is used to stop the runtime.
	"""
	def stop(self) -> None: ...

class User:
	"""
	A remote user, with uuid and username
	"""
	id: str
	name: str

class Config:
	"""
	Configuration data structure for codemp clients
	"""
	username: str
	password: str
	host: Optional[str]
	port: Optional[int]
	tls: Optional[bool]

	def __new__(cls, username: str, password: str, **kwargs) -> Config: ...

def init() -> Driver: ...
def set_logger(logger_cb: Callable[[str], None], debug: bool) -> bool: ...
def connect(config: Config) -> Promise[Client]: ...

class Promise[T]:
	"""
	This is a class akin to a future, which wraps a join handle from a spawned
	task on the rust side. you may call .pyawait() on this promise to block
	until we have a result, or return immediately if we already have one.
	This only goes one way rust -> python.

	It can either be used directly or you can wrap it inside a future python side.
	"""
	def wait(self)      -> T: ...
	def is_done(self)   -> bool: ...

class Client:
	"""
	Handle to the actual client that manages the session. It manages the connection
	to a server and joining/creating new workspaces
	"""
	def attach_workspace(self, workspace: str)  -> Promise[Workspace]: ...
	def create_workspace(self, workspace: str)  -> Promise[None]: ...
	def delete_workspace(self, workspace: str)  -> Promise[None]: ...
	def invite_to_workspace(self, workspace: str, username: str) -> Promise[None]: ...
	def fetch_owned_workspaces(self)            -> Promise[list[str]]: ...
	def fetch_joined_workspaces(self)           -> Promise[list[str]]: ...
	def leave_workspace(self, workspace: str)   -> bool: ...
	def get_workspace(self, id: str)            -> Workspace: ...
	def active_workspaces(self)                 -> list[str]: ...
	def current_user(self)                      -> User: ...
	def refresh(self)                           -> Promise[None]: ...

class Event:
	pass

class Workspace:
	"""
	Handle to a workspace inside codemp. It manages buffers.
	A cursor is tied to the single workspace.
	"""
	def create_buffer(self, path: str)          -> Promise[None]: ...
	def attach_buffer(self, path: str)          -> Promise[BufferController]: ...
	def detach_buffer(self, path: str)          -> bool: ...
	def fetch_buffers(self)                     -> Promise[list[str]]: ...
	def fetch_users(self)                       -> Promise[list[User]]: ...
	def fetch_buffer_users(self, path: str)     -> Promise[list[User]]: ...
	def delete_buffer(self, path: str)          -> Promise[None]: ...
	def id(self)                                -> str: ...
	def cursor(self)                            -> CursorController: ...
	def get_buffer(self, path: str)             -> Optional[BufferController]: ...
	def user_list(self)                         -> list[User]: ...
	def active_buffers(self)                    -> list[str]: ...
	def search_buffers(self, filter: Optional[str]) -> list[str]: ...
	def recv(self)                              -> Promise[Event]: ...
	def try_recv(self)                          -> Promise[Optional[Event]]: ...
	def poll(self)                              -> Promise[None]: ...
	def clear_callback(self)                    -> None: ...
	def callback(self, cb: Callable[[Workspace], None]) -> None: ...

class TextChange:
	"""
	Editor agnostic representation of a text change, it translate between internal
	codemp text operations and editor operations
	"""
	start: int
	end: int
	content: str

	def is_delete(self)         -> bool: ...
	def is_insert(self)         -> bool: ...
	def is_empty(self)          -> bool: ...
	def apply(self, txt: str)   -> str: ...

class BufferUpdate:
	"""
	A single editor delta event, wrapping a TextChange and the new version
	"""
	change: TextChange
	hash: Optional[int]
	version: list[int]


class BufferController:
	"""
	Handle to the controller for a specific buffer, which manages the back and forth
	of operations to and from other peers.
	"""
	def path(self)                              -> str: ...
	def content(self)                           -> Promise[str]: ...
	def ack(self, v: list[int])                 -> None: ...
	def send(self, op: TextChange)              -> None: ...
	def try_recv(self)                          -> Promise[Optional[TextChange]]: ...
	def recv(self)                              -> Promise[TextChange]: ...
	def poll(self)                              -> Promise[None]: ...
	def callback(self,
		cb: Callable[[BufferController], None]) -> None: ...
	def clear_callback(self)                    -> None: ...



class Selection:
	"""
	An Editor agnostic cursor position representation
	"""
	start: Tuple[int, int]
	end: Tuple[int, int]
	buffer: str

class Cursor:
	"""
	A remote cursor event
	"""
	user: str
	sel: Selection


class CursorController:
	"""
	Handle to the controller for a workspace, which manages the back and forth of
	cursor movements to and from other peers
	"""
	def send(self, pos: Selection)              -> None: ...
	def try_recv(self)                          -> Promise[Optional[Cursor]]: ...
	def recv(self)                              -> Promise[Cursor]: ...
	def poll(self)                              -> Promise[None]: ...
	def callback(self,
		cb: Callable[[CursorController], None]) -> None: ...
	def clear_callback(self)                    -> None: ...

