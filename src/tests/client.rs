use super::{
	assert_or_err,
	fixtures::{ClientFixture, ScopedFixture, WorkspaceFixture},
};
use crate::api::{AsyncReceiver, AsyncSender};

#[tokio::test]
async fn test_workspace_creation_and_lookup() {
	super::fixture! {
		ClientFixture::of("alice") => |client| {
			let workspace_name = uuid::Uuid::new_v4().to_string();
			let wrong_name = uuid::Uuid::new_v4().to_string();

			client.create_workspace(&workspace_name).await?;
			let wslist = client.fetch_owned_workspaces().await.unwrap_or_default();
			let ws = client.get_workspace(&workspace_name);
			let ws_exists = ws.is_some();
			let ws_name_matches = ws.map_or(false, |x| x.id() == workspace_name);
			let ws_wrong_name_doesnt_exist = client.get_workspace(&wrong_name).is_none();

			let res = client.delete_workspace(&workspace_name).await;

			assert_or_err!(ws_exists);
			assert_or_err!(ws_name_matches);
			assert_or_err!(ws_wrong_name_doesnt_exist);
			assert_or_err!(res.is_ok());
			assert_or_err!(client.get_workspace(&workspace_name).is_none());
			assert_or_err!(wslist.len() == 1);
			assert_or_err!(wslist.contains(&workspace_name));

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
async fn test_leave_workspace_with_dangling_ref() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-dangling-ref") => |client, workspace| {
			assert_or_err!(client.leave_workspace(&workspace.id()) == false);
			Ok(())
		}
	}
}

#[tokio::test]
async fn test_attach_after_leave() {
	super::fixture! {
		WorkspaceFixture::one("alice", "test-dangling-ref") => |client, workspace| {
			client.leave_workspace(&workspace.id());
			assert_or_err!(client.attach_workspace(&workspace.id()).await.is_ok());
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
async fn test_invite_user_to_workspace_and_invited_lookup() {
	WorkspaceFixture::one("bob", "workspace-di-bob")
		.with(
			|(client_bob, workspace_bob)| {
				let client_bob = client_bob.clone();
				let workspace_bob = workspace_bob.clone();

				async move {
					let client_alice = ClientFixture::of("alice").setup().await?;

					let wrong_workspace_name = uuid::Uuid::new_v4().to_string();
					// inviting to a non existing workspace is an error
					assert_or_err!(client_bob
						.invite_to_workspace(
							wrong_workspace_name,
							client_alice.current_user().name.clone(),
						)
						.await
						.is_err());

					client_bob
						.invite_to_workspace(
							workspace_bob.id(),
							client_alice.current_user().name.clone(),
						)
						.await?;

					// there are two users now in the workspace of bob
					// alice is one of the users
					// bob is one of the users
					// the workspace appears in the joined workspaces for alice
					// the workspace does not appear in the owned workspaces for alice

					let user_list = workspace_bob.fetch_users().await?;
					assert_or_err!(user_list.len() == 2);
					assert_or_err!(user_list
						.iter()
						.any(|u| u.name == client_alice.current_user().name));
					assert_or_err!(user_list
						.iter()
						.any(|u| u.name == client_bob.current_user().name));

					let alice_owned_workspaces = client_alice.fetch_owned_workspaces().await?;
					let alice_invited_workspaces = client_alice.fetch_joined_workspaces().await?;

					assert_or_err!(alice_owned_workspaces.is_empty());
					assert_or_err!(alice_invited_workspaces.contains(&workspace_bob.id()));
					Ok(())
				}
			},
		)
		.await
}

// Now we can use workspace fixtures with invite.

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
		.with(
			|(_, workspace_alice)| {
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
			},
		)
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
