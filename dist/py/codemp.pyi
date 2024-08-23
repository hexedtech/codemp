from typing import Tuple, Optional, Callable

class Driver:
	"""
	this is akin to a big red button with a white "STOP" on top of it.
	it is used to stop the runtime.
	"""
	def stop(self) -> None: ...


def init(logger_cb: Callable[[str], None], debug: bool) -> Driver: ...

class Promise[T]:
	"""
	This is a class akin to a future, which wraps a join handle from a spawned
	task on the rust side. you may call .pyawait() on this promise to block
	until we have a result, or return immediately if we already have one.
	This only goes one way rust -> python.

	It can either be used directly or you can wrap it inside a future python side.
	"""
	def wait(self) 		-> T: ...
	def is_done(self) 	-> bool: ...

class TextChange:
	"""
	Editor agnostic representation of a text change, it translate between internal
	codemp text operations and editor operations
	"""
	start: int
	end: int
	content: str

	def is_delete(self) 		-> bool: ...
	def is_insert(self) 		-> bool: ...
	def is_empty(self) 			-> bool: ...
	def apply(self, txt: str) 	-> str: ...


class BufferController:
	"""
	Handle to the controller for a specific buffer, which manages the back and forth
	of operations to and from other peers.
	"""
	def name(self)								-> str: ...
	def content(self) 							-> Promise[str]: ...
	def send(self,
		start: int,
		end: int,
		txt: str) 								-> Promise[None]: ...
	def try_recv(self) 							-> Promise[Optional[TextChange]]: ...
	def recv(self) 								-> Promise[TextChange]: ...
	def poll(self) 								-> Promise[None]: ...
	def callback(self,
		cb: Callable[[BufferController], None]) -> None: ...
	def clear_callback(self) 					-> None: ...
	def stop(self) 								-> bool: ...



class Cursor:
	"""
	An Editor agnostic cursor position representation
	"""
	start: Tuple[int, int]
	end: Tuple[int, int]
	buffer: str
	user: Optional[str] # can be an empty string


class CursorController:
	"""
	Handle to the controller for a workspace, which manages the back and forth of
	cursor movements to and from other peers
	"""
	def send(self,
		path: str,
		start: Tuple[int, int],
		end: Tuple[int, int]) 					-> Promise[None]: ...
	def try_recv(self) 							-> Promise[Optional[Cursor]]: ...
	def recv(self) 								-> Promise[Cursor]: ...
	def poll(self) 								-> Promise[None]: ...
	def callback(self,
		cb: Callable[[CursorController], None]) -> None: ...
	def clear_callback(self) 					-> None: ...
	def stop(self) 								-> bool: ...


class Workspace:
	"""
	Handle to a workspace inside codemp. It manages buffers.
	A cursor is tied to the single workspace.
	"""
	def create(self, path: str) 				-> Promise[None]: ...
	def attach(self, path: str) 				-> Promise[BufferController]: ...
	def detach(self, path: str) 				-> bool: ...
	def fetch_buffers(self) 					-> Promise[None]: ...
	def fetch_users(self) 						-> Promise[None]: ...
	def list_buffer_users(self, path: str) 		-> Promise[list[str]]: ...
	def delete(self, path: str) 				-> Promise[None]: ...
	def id(self) 								-> str: ...
	def cursor(self) 							-> CursorController: ...
	def buffer_by_name(self, path: str) 		-> Optional[BufferController]: ...
	def buffer_list(self) 						-> list[str]: ...
	def filetree(self, filter: Optional[str]) 	-> list[str]: ...


class Client:
	"""
	Handle to the actual client that manages the session. It manages the connection
	to a server and joining/creating new workspaces
	"""
	def __new__(cls,
		host: str,
		username: str, password: str) 			-> Client: ...
	def join_workspace(self, workspace: str) 	-> Promise[Workspace]: ...
	def leave_workspace(self, workspace: str) 	-> bool: ...
	def get_workspace(self, id: str) 			-> Workspace: ...
	def active_workspaces(self) 				-> list[str]: ...
	def user_id(self) 							-> str: ...
