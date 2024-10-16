import mp.code.*;
import mp.code.data.Config;
import mp.code.data.Cursor;
import mp.code.data.TextChange;
import mp.code.data.User;
import mp.code.exceptions.ConnectionException;
import mp.code.exceptions.ConnectionRemoteException;
import mp.code.exceptions.ControllerException;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;

import java.util.Objects;
import java.util.Optional;
import java.util.OptionalLong;
import java.util.UUID;

import static mp.code.data.DetachResult.DETACHING;

@SuppressWarnings({"StatementWithEmptyBody", "OptionalGetWithoutIsPresent"})
public class CodeMPTest {
	private final Client client;
	private final Client otherClient;

	// client connection init
	public CodeMPTest() throws ConnectionException {
		new Thread(() -> Extensions.drive(true)); // drive thread so callback works
		//Extensions.setupTracing(null, true);

		this.client = Client.connect(new Config(
			Objects.requireNonNull(System.getenv("CODEMP_TEST_USERNAME_1")),
			Objects.requireNonNull(System.getenv("CODEMP_TEST_PASSWORD_1")),
			"api.codemp.dev",
			50053,
			false
		));

		// failed tests may have cluttered the list, clean it first
		for(String ws : this.client.listWorkspaces(true, false)) {
			this.client.deleteWorkspace(ws);
		}

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
		//assert u.name.equals(System.getenv("CODEMP_TEST_USERNAME_1"));
		//assert u.id.toString().equals(System.getenv("CODEMP_TEST_ID_1"));
	}

	@Test
	void testWorkspaceInteractions() throws ConnectionException {
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
		assert !this.otherClient.leaveWorkspace(randomName);

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

		// prepare first client
		this.client.createWorkspace(randomWorkspace);
		Workspace ws = this.client.joinWorkspace(randomWorkspace);

		// test buffer creation and verify that the buffer list has changed
		int oldFileTree = ws.getFileTree(Optional.empty(), true).length;
		ws.createBuffer(randomBuffer);
		assert (oldFileTree + 1) == ws.getFileTree(Optional.empty(), true).length;

		// test buffer filters
		assert ws.getFileTree(Optional.of(randomBuffer.substring(0, 10)), true).length == 0;
		assert ws.getFileTree(Optional.of(randomBuffer.substring(0, 10)), false).length == 1;

		int oldActive = ws.activeBuffers().length;
		ws.attachToBuffer(randomBuffer);
		assert (oldActive + 1) == ws.activeBuffers().length;

		BufferController buffer = ws.getBuffer(randomBuffer).get();

		// prepare second client and clean queue
		this.client.inviteToWorkspace(ws.getWorkspaceId(), this.otherClient.getUser().name);
		BufferController otherBuffer = this.otherClient.joinWorkspace(randomWorkspace).attachToBuffer(randomBuffer);
		while(buffer.tryRecv().isPresent()) {}

		TextChange textChange = new TextChange(0, 0, "", OptionalLong.empty());

		/* Testing callback */
		buffer.callback(bufferController -> new Thread(() -> {
			try {
				assert bufferController.recv().equals(textChange);
			} catch(ControllerException e) {
				throw new RuntimeException(e);
			}
		}).start());

		otherBuffer.send(textChange);
		buffer.poll();
		buffer.clearCallback();

		otherBuffer.send(textChange);
		buffer.recv();

		otherBuffer.send(textChange);
		buffer.poll();
		assert buffer.tryRecv().isPresent();

		assert buffer.tryRecv().isEmpty();

		assert ws.detachFromBuffer(randomBuffer) == DETACHING;

		this.otherClient.getWorkspace(randomWorkspace).get().createBuffer(UUID.randomUUID().toString());
		assert ws.event().getChangedBuffer().isPresent();

		ws.deleteBuffer(randomBuffer);
		Assertions.assertEquals(oldFileTree, ws.getFileTree(Optional.empty(), true).length);

		this.client.leaveWorkspace(randomWorkspace);
		this.client.deleteWorkspace(randomWorkspace);
	}

	@Test
	void testWorkspaceEvents() throws ConnectionException, ControllerException {
		String randomWorkspace = UUID.randomUUID().toString();

		// prepare first client
		this.client.createWorkspace(randomWorkspace);
		Workspace ws = this.client.joinWorkspace(randomWorkspace);
		this.client.inviteToWorkspace(randomWorkspace, this.otherClient.getUser().name);

		// prepare second client
		this.otherClient.joinWorkspace(randomWorkspace).createBuffer(UUID.randomUUID().toString());

		// block until event is received
		assert ws.event().getChangedBuffer().isPresent();

		// cleanup
		this.otherClient.leaveWorkspace(randomWorkspace);
		this.client.deleteWorkspace(randomWorkspace);
	}

	@Test
	void testCursorInteractions() throws ConnectionException, ControllerException, InterruptedException {
		String randomWorkspace = UUID.randomUUID().toString();
		String randomBuffer = UUID.randomUUID().toString();

		// prepare first client
		this.client.createWorkspace(randomWorkspace);
		Workspace ws = this.client.joinWorkspace(randomWorkspace);
		ws.createBuffer(randomBuffer);
		ws.attachToBuffer(randomBuffer);
		CursorController cursor = ws.getCursor();

		// prepare second client and clean queue
		this.client.inviteToWorkspace(ws.getWorkspaceId(), this.otherClient.getUser().name);
		CursorController otherCursor = this.otherClient.joinWorkspace(randomWorkspace).getCursor();
		while(cursor.tryRecv().isPresent()) {}

		Cursor someCursor = new Cursor(
			0, 0, 0, 0, randomBuffer, this.otherClient.getUser().name
		);

		/* Testing callback */
		cursor.callback(cursorController -> new Thread(() -> {
			try {
				assert cursorController.recv().equals(someCursor);
			} catch(ControllerException e) {
				throw new RuntimeException(e);
			}
		}).start());

		otherCursor.send(someCursor);
		cursor.poll(); // wait for other thread to send
		cursor.clearCallback(); // should have now received the first callback, clear it

		/* Testing recv and tryRecv */
		otherCursor.send(someCursor);
		cursor.recv(); // block until receive

		// send flat cursor
		otherCursor.send(new Cursor(
			0, 0, 0, 0, randomBuffer, this.otherClient.getUser().name
		));
		cursor.poll();
		assert cursor.tryRecv().isPresent(); // expect something (and consume)
		assert cursor.tryRecv().isEmpty(); // expect nothing

		// cleanup
		this.otherClient.leaveWorkspace(randomWorkspace);
		this.client.leaveWorkspace(randomWorkspace);
		this.client.deleteWorkspace(randomWorkspace);
	}
}
