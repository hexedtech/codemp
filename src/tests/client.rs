use super::{
	assert_or_err,
	fixtures::{ClientFixture, ScopedFixture, WorkspaceFixture},
};
use crate::api::{AsyncReceiver, AsyncSender};

#[tokio::test]
async fn test_workspace_creation_and_deletion() {
	super::fixture! {
		ClientFixture::of("alice") => |client| {
			let workspace_name = uuid::Uuid::new_v4().to_string();

			client.create_workspace(&workspace_name).await?;

			// we can't error, so we return empty vec which will be interpreted as err
			let workspace_list_before = client.fetch_owned_workspaces().await.unwrap_or_default();

			let res = client.delete_workspace(&workspace_name).await;

			// we can and should err here, because empty vec will be counted as success!
			let workspace_list_after = client.fetch_owned_workspaces().await?;

			assert_or_err!(workspace_list_before.contains(&workspace_name));
			res?;
			assert_or_err!(workspace_list_after.contains(&workspace_name) == false);

			Ok(())
		}
	};
}

#[tokio::test]
async fn test_attach_and_leave_workspace() {
	super::fixture! {
		ClientFixture::of("alice") => |client| {
			let workspace_name = uuid::Uuid::new_v4().to_string();

			client.create_workspace(&workspace_name).await?;

			// leaving a workspace you are not attached to, returns true
			let leave_workspace_before = client.leave_workspace(&workspace_name);

			let attach_workspace_that_exists = match client.attach_workspace(&workspace_name).await {
				Ok(_) => true,
				Err(e) => {
					eprintln!("error attaching to workspace: {e}");
					false
				},
			};

			// leaving a workspace you are attached to, returns true
			// when there is only one reference to it.
			let leave_workspace_after = client.leave_workspace(&workspace_name);

			let _ = client.delete_workspace(&workspace_name).await;

			assert_or_err!(leave_workspace_before, "leaving a workspace you are not attached to returned false, should return true.");
			assert_or_err!(attach_workspace_that_exists, "attaching a workspace that exists failed with error");
			assert_or_err!(leave_workspace_after, "leaving a workspace with a single reference returned false.");

			Ok(())
		}
	}
}

#[tokio::test]
async fn test_invite_user_to_workspace() {
	let client_alice = ClientFixture::of("alice")
		.setup()
		.await
		.expect("failed setting up alice's client");
	let client_bob = ClientFixture::of("bob")
		.setup()
		.await
		.expect("failed setting up bob's client");
	let ws_name = uuid::Uuid::new_v4().to_string();

	// after this we can't just fail anymore: we need to cleanup, so store errs
	client_alice
		.create_workspace(&ws_name)
		.await
		.expect("failed creating workspace");
	let could_invite = client_alice
		.invite_to_workspace(&ws_name, &client_bob.current_user().name)
		.await;
	let ws_list = client_bob
		.fetch_joined_workspaces()
		.await
		.unwrap_or_default(); // can't fail, empty is err
	let could_delete = client_alice.delete_workspace(&ws_name).await;

	could_invite.expect("could not invite bob");
	assert!(ws_list.contains(&ws_name));
	could_delete.expect("could not delete workspace");
}

#[tokio::test]
async fn test_workspace_lookup() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-lookup") => |client, workspace| {
			assert_or_err!(client.get_workspace(&workspace.id()).is_some());
			assert_or_err!(client.get_workspace(&uuid::Uuid::new_v4().to_string()).is_none());
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_leave_workspace_with_dangling_ref() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-dangling-ref") => |client, workspace| {
			assert_or_err!(client.leave_workspace(&workspace.id()) == false);
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_lookup_after_leave() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-lookup-after-leave") => |client, workspace| {
			client.leave_workspace(&workspace.id());
			assert_or_err!(client.get_workspace(&workspace.id()).is_none());
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_attach_after_leave() {
	super::fixture! {
		ClientFixture::of("alice") => |client| {
			let ws_name = uuid::Uuid::new_v4().to_string();
			client.create_workspace(&ws_name).await?;

			let could_attach = client.attach_workspace(&ws_name).await.is_ok();
			let clean_leave = client.leave_workspace(&ws_name);
			// TODO this is very server specific! disconnect may be instant or caught with next
			// keepalive, let's arbitrarily say that after 20 seconds we should have been disconnected
			tokio::time::sleep(std::time::Duration::from_secs(20)).await;
			let could_attach_again = client.attach_workspace(&ws_name).await;
			let could_delete = client.delete_workspace(&ws_name).await;

			assert_or_err!(could_attach);
			assert_or_err!(clean_leave);
			could_attach_again?;
			could_delete?;

			Ok(())
		}
	}
}

#[tokio::test]
async fn test_active_workspaces() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-active-workspaces") => |client, workspace| {
			assert_or_err!(client.active_workspaces().contains(&workspace.id()));
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_cant_create_same_workspace_more_than_once() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-create-multiple-times") => |client, workspace| {
			assert_or_err!(client.create_workspace(workspace.id()).await.is_err(), "created same workspace twice");
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_attaching_to_non_existing_is_error() {
	super::fixture! {
		ClientFixture::of("alice") => |client| {
			let workspace_name = uuid::Uuid::new_v4().to_string();

			// we don't create any workspace.
			// client.create_workspace(workspace_name).await?;
			assert_or_err!(client.attach_workspace(&workspace_name).await.is_err());
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_deleting_workspace_twice_is_an_error() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-delete-twice") => |client, workspace| {
			let workspace_name = workspace.id();

			client.delete_workspace(&workspace_name).await?;
			assert_or_err!(client.delete_workspace(&workspace_name).await.is_err());
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_cannot_invite_self() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-invite-self") => |client, workspace| {
			assert_or_err!(client.invite_to_workspace(workspace.id(), &client.current_user().name).await.is_err());
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_cannot_invite_to_nonexisting() {
	super::fixture! {
		WorkspaceFixture::two("alice", "bob", "test-invite-self") => |client, _ws, client_bob, _workspace_bob| {
			assert_or_err!(client.invite_to_workspace(uuid::Uuid::new_v4().to_string(), &client_bob.current_user().name).await.is_err());
			Ok(())
		}
	}
}

#[tokio::test]
async fn cannot_delete_others_workspaces() {
	WorkspaceFixture::two("alice", "bob", "test-cannot-delete-others-workspaces")
		.with(|(_, ws_alice, client_bob, _)| {
			let ws_alice = ws_alice.clone();
			let client_bob = client_bob.clone();
			async move {
				assert_or_err!(
					client_bob.delete_workspace(&ws_alice.id()).await.is_err(),
					"bob was allowed to delete a workspace he didn't own!"
				);
				Ok(())
			}
		})
		.await
}

#[tokio::test]
async fn test_buffer_search() {
	WorkspaceFixture::one("alice", "test-buffer-search")
		.with(|(_, workspace_alice)| {
			let buffer_name = uuid::Uuid::new_v4().to_string();
			let workspace_alice = workspace_alice.clone();

			async move {
				workspace_alice.create_buffer(&buffer_name).await?;
				assert_or_err!(!workspace_alice
					.search_buffers(Some(&buffer_name[0..4]))
					.is_empty());
				assert_or_err!(workspace_alice.search_buffers(Some("_")).is_empty());
				workspace_alice.delete_buffer(&buffer_name).await?;
				Ok(())
			}
		})
		.await;
}

#[tokio::test]
async fn test_send_operation() {
	WorkspaceFixture::two("alice", "bob", "test-send-operation")
		.with(|(_, workspace_alice, _, workspace_bob)| {
			let buffer_name = uuid::Uuid::new_v4().to_string();
			let workspace_alice = workspace_alice.clone();
			let workspace_bob = workspace_bob.clone();

			async move {
				workspace_alice.create_buffer(&buffer_name).await?;
				let alice = workspace_alice.attach_buffer(&buffer_name).await?;
				let bob = workspace_bob.attach_buffer(&buffer_name).await?;

				alice.send(crate::api::TextChange {
					start_idx: 0,
					end_idx: 0,
					content: "hello world".to_string(),
				})?;

				let result = bob.recv().await?;
				assert_or_err!(result.change.start_idx == 0);
				assert_or_err!(result.change.end_idx == 0);
				assert_or_err!(result.change.content == "hello world");

				Ok(())
			}
		})
		.await;
}

#[tokio::test]
async fn test_content_converges() {
	WorkspaceFixture::two("alice", "bob", "test-content-converges")
		.with(|(_, workspace_alice, _, workspace_bob)| {
			let buffer_name = uuid::Uuid::new_v4().to_string();
			let workspace_alice = workspace_alice.clone();
			let workspace_bob = workspace_bob.clone();

			async move {
				workspace_alice.create_buffer(&buffer_name).await?;
				let alice = workspace_alice.attach_buffer(&buffer_name).await?;
				let bob = workspace_bob.attach_buffer(&buffer_name).await?;

				let mut join_set = tokio::task::JoinSet::new();

				let _alice = alice.clone();
				join_set.spawn(async move {
					for i in 0..10 {
						_alice.content().await?;
						_alice.send(crate::api::TextChange {
							start_idx: 7 * i,
							end_idx: 7 * i,
							content: format!("alice{i} "), // TODO generate a random string instead!!
						})?;
						tokio::time::sleep(std::time::Duration::from_millis(100)).await;
					}
					Ok::<(), crate::errors::ControllerError>(())
				});

				let _bob = bob.clone();
				join_set.spawn(async move {
					for i in 0..10 {
						_bob.content().await?;
						_bob.send(crate::api::TextChange {
							start_idx: 5 * i,
							end_idx: 5 * i,
							content: format!("bob{i} "), // TODO generate a random string instead!!
						})?;
						tokio::time::sleep(std::time::Duration::from_millis(100)).await;
					}
					Ok::<(), crate::errors::ControllerError>(())
				});

				while let Some(x) = join_set.join_next().await {
					x??;
				}

				// test runners may be slow, give 1s to catch up, just in case
				tokio::time::sleep(std::time::Duration::from_secs(1)).await;

				// TODO is there a nicer way to make sure we received all changes?
				for i in 0..20 {
					tokio::time::sleep(std::time::Duration::from_millis(200)).await;
					match bob.try_recv().await? {
						Some(change) => bob.ack(change.version),
						None => break,
					}
					eprintln!("bob more to recv at attempt #{i}");
				}

				for i in 0..20 {
					tokio::time::sleep(std::time::Duration::from_millis(200)).await;
					match alice.try_recv().await? {
						Some(change) => alice.ack(change.version),
						None => break,
					}
					eprintln!("alice more to recv at attempt #{i}");
				}

				let alice_content = alice.content().await?;
				let bob_content = bob.content().await?;

				eprintln!("alice: {alice_content}");
				eprintln!("bob  : {bob_content}");

				assert_or_err!(alice_content == bob_content);

				Ok(())
			}
		})
		.await;
}
