from typing import Callable, Generic, Optional, TypeVar

T = TypeVar("T")


def version() -> str: ...
def init() -> Driver: ...
def set_logger(logger_cb: Callable[[str], None], debug: bool) -> bool: ...
def connect(config: Config) -> Promise[Client]: ...


class Driver:
    """
    This is akin to a big red button with a white "STOP" on top of it.
    It is used to stop the runtime.
    """

    def stop(self) -> None: ...


class Promise(Generic[T]):
    """
    Future-like object for async operations started in Rust.
    Call `wait()` to block until completion.
    """

    def wait(self) -> T: ...
    def done(self) -> bool: ...


class WorkspaceIdentifier:
    """
    Workspace identifier made of owner username and workspace name.
    """

    user: str
    workspace: str


class UserInfo:
    """
    A remote user profile.
    """

    @property
    def name(self) -> str: ...

    @property
    def display_name(self) -> str: ...

    @display_name.setter
    def display_name(self, value: str) -> None: ...

    @property
    def description(self) -> str: ...

    @description.setter
    def description(self, value: str) -> None: ...


class Config:
    """
    Configuration data structure for codemp clients.
    """

    username: str
    password: str
    host: Optional[str]
    port: Optional[int]
    tls: Optional[bool]

    def __new__(
        cls,
        *,
        username: str,
        password: str,
        host: Optional[str] = ...,
        port: Optional[int] = ...,
        tls: Optional[bool] = ...,
    ) -> Config: ...


class Client:
    """
    Handle to the session client. Manages workspaces and account operations.
    """

    def refresh(self) -> Promise[None]: ...
    def create_workspace(self, workspace: str) -> Promise[None]: ...
    def delete_workspace(self, workspace: str) -> Promise[None]: ...
    def quit_workspace(self, user: str, workspace: str) -> Promise[None]: ...
    def invite_to_workspace(self, workspace: str, user: str) -> Promise[None]: ...
    def accept_invite(self, user: str, workspace: str) -> Promise[None]: ...
    def reject_invite(self, user: str, workspace: str) -> Promise[None]: ...
    def fetch_owned_workspaces(self) -> Promise[list[WorkspaceIdentifier]]: ...
    def fetch_joined_workspaces(self) -> Promise[list[WorkspaceIdentifier]]: ...
    def get_workspace(self, user: str, workspace: str) -> Optional[Workspace]: ...
    def active_workspaces(self) -> list[WorkspaceIdentifier]: ...
    def get_user_info(self, user: str) -> Promise[UserInfo]: ...
    def current_user(self) -> UserInfo: ...
    def attach_workspace(self, user: str, workspace: str) -> Promise[Workspace]: ...
    def leave_workspace(self, user: str, workspace: str) -> bool: ...


class FileTreeUpdated:
    """
    Fired when the file tree changes.
    Contains the modified buffer path (deleted, created or renamed).
    """

    path: str


class UserJoin:
    """
    Fired when a user joins the workspace.
    """

    name: str


class UserLeave:
    """
    Fired when a user leaves the workspace.
    """

    name: str


class UserJoinBuffer:
    """
    Fired when a user joins a specific buffer.
    """

    name: str
    buffer: str


class UserLeaveBuffer:
    """
    Fired when a user leaves a specific buffer.
    """

    name: str
    buffer: str


class Event:
    """
    Workspace events to notify users of changes happening in the workspace.
    """

    FileTreeUpdated: FileTreeUpdated
    UserJoin: UserJoin
    UserLeave: UserLeave
    UserJoinBuffer: UserJoinBuffer
    UserLeaveBuffer: UserLeaveBuffer


class Workspace:
    """
    Handle to a workspace. It manages buffers and workspace events.
    """

    def create_buffer(self, path: str, ephemeral: bool) -> Promise[None]: ...
    def pin_buffer(self, path: str) -> Promise[None]: ...
    def un_pin_buffer(self, path: str) -> Promise[None]: ...
    def attach_buffer(self, path: str) -> Promise[BufferController]: ...
    def detach_buffer(self, path: str) -> bool: ...
    def fetch_buffers(self) -> Promise[None]: ...
    def fetch_users(self) -> Promise[None]: ...
    def fetch_buffer_users(self, path: str) -> Promise[None]: ...
    def delete_buffer(self, path: str) -> Promise[None]: ...
    def id(self) -> WorkspaceIdentifier: ...
    def cursor(self) -> CursorController: ...
    def get_buffer(self, path: str) -> Optional[BufferController]: ...
    def active_buffers(self) -> list[str]: ...
    def search_buffers(self, filter: Optional[str] = None) -> list[str]: ...
    def user_list(self) -> list[UserInfo]: ...
    def recv(self) -> Promise[Event]: ...
    def try_recv(self) -> Promise[Optional[Event]]: ...
    def poll(self) -> Promise[None]: ...
    def clear_callback(self) -> None: ...
    def callback(self, cb: Callable[[Workspace], None]) -> None: ...


class TextChange:
    """
    Editor-agnostic representation of a text change.
    """

    start_idx: int
    end_idx: int
    content: str

    def __new__(
        cls,
        *,
        start: int,
        end: int,
        content: str,
    ) -> TextChange: ...

    def is_delete(self) -> bool: ...
    def is_insert(self) -> bool: ...
    def is_empty(self) -> bool: ...
    def apply(self, txt: str) -> str: ...


class BufferUpdate:
    """
    A single buffer delta event with version and optional post-change hash.
    """

    hash: Optional[int]
    version: list[int]
    change: TextChange


class BufferController:
    """
    Handle to a specific buffer controller.
    """

    def path(self) -> str: ...
    def content(self) -> Promise[str]: ...
    def ack(self, v: list[int]) -> None: ...
    def send(self, op: TextChange) -> None: ...
    def try_recv(self) -> Promise[Optional[BufferUpdate]]: ...
    def recv(self) -> Promise[BufferUpdate]: ...
    def poll(self) -> Promise[None]: ...
    def callback(self, cb: Callable[[BufferController], None]) -> None: ...
    def clear_callback(self) -> None: ...


class Selection:
    """
    Editor-agnostic cursor selection representation.
    """

    start_row: int
    start_col: int
    end_row: int
    end_col: int

    def __new__(
        cls,
        *,
        start_row: int,
        start_col: int,
        end_row: int,
        end_col: int,
    ) -> Selection: ...


class Cursor:
    """
    Cursor payload with flattened getters exposed by FFI.
    """

    start: list[tuple[int, int]]
    end: list[tuple[int, int]]
    buffer: str


class CursorEvent:
    """
    Cursor event with sending user and cursor payload.
    """

    user: str
    cursor: Cursor


class CursorController:
    """
    Handle to the workspace cursor controller.
    """

    def send(self, pos: Cursor) -> None: ...
    def try_recv(self) -> Promise[Optional[CursorEvent]]: ...
    def recv(self) -> Promise[CursorEvent]: ...
    def poll(self) -> Promise[None]: ...
    def callback(self, cb: Callable[[CursorController], None]) -> None: ...
    def clear_callback(self) -> None: ...
