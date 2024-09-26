import mp.code.*;
import mp.code.data.Config;
import mp.code.data.Cursor;
import mp.code.data.TextChange;
import mp.code.data.User;
import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;
import mp.code.exceptions.ControllerException;
import org.junit.jupiter.api.Test;

import java.util.Objects;
import java.util.Optional;
import java.util.OptionalLong;
import java.util.UUID;

import static mp.code.data.DetachResult.DETACHING;

public class CodeMPTest {
	private final Client client;
	private final Client otherClient;

	// client connection init
	public CodeMPTest() throws ConnectionException {
		System.out.println("aaaa");
		//new Thread(() -> Extensions.drive(true)); // drive thread so callback works
		Extensions.setupTracing(null, true);

		this.client = Client.connect(new Config(
			Objects.requireNonNull(System.getenv("CODEMP_TEST_USERNAME_1")),
			Objects.requireNonNull(System.getenv("CODEMP_TEST_PASSWORD_1")),
			"api.codemp.dev",
			50053,
			false
		));

		this.otherClient = Client.connect(new Config(
			Objects.requireNonNull(System.getenv("CODEMP_TEST_USERNAME_2")),
			Objects.requireNonNull(System.getenv("CODEMP_TEST_PASSWORD_2")),
			"api.code.mp",
			50053,
			false
		));
	}

	@Test
	void testGetUser() {
		User u = this.client.getUser();
		System.out.println("User name:" + u.name);
		System.out.println("User ID: " + u.id);
	}

	@Test
	void testWorkspaceInteraction() throws ConnectionException {
		String randomName = UUID.randomUUID().toString();

		int oldOwned = this.client.listWorkspaces(true, false).length;
		int oldInvited = this.client.listWorkspaces(false, true).length;
		this.client.createWorkspace(randomName);
		assert (oldOwned + 1) == this.client.listWorkspaces(true, false).length;
		assert oldInvited == this.client.listWorkspaces(false, true).length;

		int activeWorkspaces = this.client.activeWorkspaces().length;
		this.client.joinWorkspace(randomName);
		assert (activeWorkspaces + 1) == this.client.activeWorkspaces().length;

		Optional<Workspace> ws = this.client.getWorkspace(randomName);
		assert ws.isPresent();
		assert ws.get().getWorkspaceId().equals(randomName);
		ws.get().fetchBuffers();
		ws.get().fetchUsers();

		this.client.inviteToWorkspace(randomName, this.otherClient.getUser().name);
		assert this.client.leaveWorkspace(randomName);
		assert this.otherClient.leaveWorkspace(randomName);

		this.client.deleteWorkspace(randomName);
	}

	@Test
	void testRefresh() throws ConnectionRemoteException {
		this.client.refresh();
	}

	@Test
	void testBufferInteractions() throws ConnectionException, ControllerException, InterruptedException {
		String randomWorkspace = UUID.randomUUID().toString();
		String randomBuffer = UUID.randomUUID().toString();
		this.client.createWorkspace(randomWorkspace);
		Workspace ws = this.client.joinWorkspace(randomWorkspace);

		this.client.inviteToWorkspace(ws.getWorkspaceId(), this.otherClient.getUser().name);

		int oldFileTree = ws.getFileTree(Optional.empty(), true).length;
		ws.createBuffer(randomBuffer);
		assert (oldFileTree + 1) == ws.getFileTree(Optional.empty(), true).length;

		assert ws.getFileTree(Optional.of(randomBuffer.substring(0, 10)), true).length == 0;
		assert ws.getFileTree(Optional.of(randomBuffer.substring(0, 10)), false).length == 1;

		ws.deleteBuffer(randomBuffer);
		assert oldFileTree == ws.getFileTree(Optional.empty(), true).length;

		ws.createBuffer(randomBuffer);

		int oldActive = ws.activeBuffers().length;
		ws.attachToBuffer(randomBuffer);
		assert (oldActive + 1) == ws.activeBuffers().length;

		Optional<BufferController> buffer = ws.getBuffer(randomBuffer);
		assert buffer.isPresent();

		buffer.get().callback(bufferController -> {
			assert true;
		});

		Thread t = new Thread(() -> parallelBufferThreadTask(randomWorkspace, randomBuffer));
		t.start();

		// wait for other thread to attach
		while(ws.listBufferUsers(randomBuffer).length == 1) {
			wait(50);
		}

		buffer.get().poll();
		buffer.get().clearCallback();

		buffer.get().recv();

		buffer.get().poll();
		assert buffer.get().tryRecv().isPresent();

		assert buffer.get().tryRecv().isEmpty();

		buffer.get().send(new TextChange(0, 0, "1", OptionalLong.empty()));

		assert ws.detachFromBuffer(randomBuffer) == DETACHING;
		ws.deleteBuffer(randomBuffer);

		assert ws.event().getChangedBuffer().isPresent();
		t.join(1000);

		this.client.leaveWorkspace(randomWorkspace);
		this.client.deleteWorkspace(randomWorkspace);
	}

	private void parallelBufferThreadTask(String workspace, String buffer) {
		try {
			Workspace w = this.otherClient.joinWorkspace(workspace);
			BufferController controller = w.attachToBuffer(buffer);
			for(int i = 0; i < 3; i++) {
				try {
					wait(200);
					controller.send(new TextChange(
						0, 0, "1", OptionalLong.empty()
					));
				} catch(InterruptedException e) {
					break;
				}
			}
			w.detachFromBuffer(buffer);

			String anotherRandomBuffer = UUID.randomUUID().toString();
			w.createBuffer(anotherRandomBuffer);
			w.deleteBuffer(anotherRandomBuffer);
		} catch(ConnectionException | ControllerException e) {
			throw new RuntimeException(e);
		}
	}

	@Test
	void testCursorInteractions() throws ConnectionException, InterruptedException, ControllerException {
		String randomWorkspace = UUID.randomUUID().toString();
		String randomBuffer = UUID.randomUUID().toString();

		// prepare first client
		this.client.createWorkspace(randomWorkspace);
		Workspace ws = this.client.joinWorkspace(randomWorkspace);
		this.client.inviteToWorkspace(ws.getWorkspaceId(), this.otherClient.getUser().name);
		ws.createBuffer(randomBuffer);
		ws.attachToBuffer(randomBuffer);
		CursorController cursor = ws.getCursor();

		// prepare second client (ignore initial cursor for convenience)
		this.otherClient.joinWorkspace(randomWorkspace).attachToBuffer(randomBuffer);

		cursor.callback(bufferController -> {
			assert true;
		});

		Thread t = new Thread(() -> parallelCursorThreadTask(randomWorkspace, randomBuffer));
		t.start();

		// wait for other thread to attach
		while(ws.listBufferUsers(randomBuffer).length == 1) {
			wait(50);
		}

		cursor.poll();
		cursor.clearCallback();

		cursor.recv();

		cursor.poll();
		assert cursor.tryRecv().isPresent();

		assert cursor.tryRecv().isEmpty();

		cursor.send(new Cursor(0, 0, 0, 0, randomBuffer, this.client.getUser().name));

		assert ws.detachFromBuffer(randomBuffer) == DETACHING;
		ws.deleteBuffer(randomBuffer);

		t.join(1000);

		this.client.leaveWorkspace(randomWorkspace);
		this.client.deleteWorkspace(randomWorkspace);

	}

	private void parallelCursorThreadTask(String workspace, String buffer) {
		try {
			@SuppressWarnings("OptionalGetWithoutIsPresent")
			Workspace w = this.otherClient.getWorkspace(workspace).get();
			for(int i = 0; i < 3; i++) {
				try {
					wait(200);
					w.getCursor().send(new Cursor(
						0, 0, 0, 0, buffer, this.otherClient.getUser().name
					));
				} catch(InterruptedException e) {
					break;
				}
			}
			w.detachFromBuffer(buffer);
		} catch(ControllerException e) {
			throw new RuntimeException(e);
		}
	}
}
