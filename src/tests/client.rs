use super::{
	assert_or_err,
	fixtures::{ClientFixture, ScopedFixture, WorkspaceFixture},
};
use crate::api::{AsyncReceiver, AsyncSender};

#[tokio::test]
async fn test_workspace_creation_and_lookup() {
	ClientFixture::of("alice")
		.with(|client| {
			let client = client.clone();

			async move {
				let workspace_name = uuid::Uuid::new_v4().to_string();
				let wrong_name = uuid::Uuid::new_v4().to_string();

				client.create_workspace(&workspace_name).await?;
				let ws = client.get_workspace(&workspace_name);
				assert_or_err!(
					ws.is_some(),
					"Failed to retrieve the workspace just created."
				);
				assert_or_err!(
					ws.unwrap().id() == workspace_name,
					"The retreived workspace has a different name than the one created."
				);

				assert_or_err!(
					client.get_workspace(&wrong_name).is_none(),
					"Looking up a non existent workspace returned something."
				);

				let wslist = client.fetch_owned_workspaces().await?;
				assert_or_err!(wslist.len() == 1, "the amount of owned workspaces is not 1");
				assert_or_err!(
					wslist.contains(&workspace_name),
					"the newly create workspace is not present in the owned list."
				);
				Ok(())
			}
		})
		.await
}

#[tokio::test]
async fn test_cant_create_same_workspace_more_than_once() {
	ClientFixture::of("alice")
		.with(|client| {
			let client = client.clone();

			async move {
				let workspace_name = uuid::Uuid::new_v4().to_string();

				client.create_workspace(&workspace_name).await?;
				assert_or_err!(
					client.create_workspace(workspace_name).await.is_err(),
					"a workspace was created twice without error."
				);
				Ok(())
			}
		})
		.await
}

#[tokio::test]
async fn test_workspace_attach_and_active_workspaces() {
	ClientFixture::of("alice")
		.with(|client| {
			let client = client.clone();

			async move {
				let workspace_name = uuid::Uuid::new_v4().to_string();

				client.create_workspace(&workspace_name).await?;
				let ws = client.attach_workspace(&workspace_name).await?;

				assert_or_err!(
					ws.id() == workspace_name,
					"we attached to a workspace that has a different name than the one requested."
				);
				assert_or_err!(
					client.active_workspaces().contains(&workspace_name),
					"the workspace is not present in the active workspaces list after attaching."
				);
				Ok(())
			}
		})
		.await
}

#[tokio::test]
async fn test_attaching_to_non_existing_is_error() {
	ClientFixture::of("alice")
		.with(|client| {
			let client = client.clone();

			async move {
				let workspace_name = uuid::Uuid::new_v4().to_string();

				// we don't create any workspace.
				// client.create_workspace(workspace_name).await?;
				assert_or_err!(
					client.attach_workspace(&workspace_name).await.is_err(),
					"we attached to a non existing workspace."
				);
				Ok(())
			}
		})
		.await
}

#[tokio::test]
async fn test_attach_and_leave_workspace_interactions() {
	ClientFixture::of("alice")
		.with(|client| {
			let client = client.clone();

			async move {
				let workspace_name = uuid::Uuid::new_v4().to_string();

				client.create_workspace(&workspace_name).await?;
				// leaving a workspace you are not attached to, returns true
				assert_or_err!(
				client.leave_workspace(&workspace_name),
				"leaving a workspace you are not attached to returned false, should return true."
			);

				// leaving a workspace you are attached to, returns true
				// when there is only one reference to it.
				client.attach_workspace(&workspace_name).await?;
				assert_or_err!(
					client.leave_workspace(&workspace_name),
					"leaving a workspace with a single reference returned false."
				);

				// we are also implicitly testing repeated leaving and attaching
				// to the same workspace.
				let res = client.attach_workspace(&workspace_name).await;
				assert_or_err!(res.is_ok(), "could not attach to the same workspace immediately after successfully leaving it.");

				// we should have an extra reference here, which should make the
				// consume return false.
				let _ws = res.unwrap();
				assert_or_err!(
					!client.leave_workspace(&workspace_name),
					"leaving a workspace while some reference still exist returned true."
				);

				Ok(())
			}
		})
		.await
}

#[tokio::test]
async fn test_delete_empty_workspace() {
	ClientFixture::of("alice")
		.with(|client| {
			let client = client.clone();

			async move {
				let workspace_name = uuid::Uuid::new_v4().to_string();

				client.create_workspace(&workspace_name).await?;
				client.delete_workspace(&workspace_name).await?;

				assert_or_err!(
					client.get_workspace(&workspace_name).is_none(),
					"a deleted workspaces was still available for lookup."
				);
				assert_or_err!(
					client.fetch_owned_workspaces().await?.is_empty(),
					"a deleted workspace was still present in the owned workspaces list."
				);

				Ok(())
			}
		})
		.await
}

#[tokio::test]
async fn test_deleting_twice_or_non_existing_is_an_error() {
	ClientFixture::of("alice")
		.with(|client| {
			let client = client.clone();

			async move {
				let workspace_name = uuid::Uuid::new_v4().to_string();

				client.create_workspace(&workspace_name).await?;
				client.delete_workspace(&workspace_name).await?;

				assert_or_err!(
					client.delete_workspace(&workspace_name).await.is_err(),
					"we could delete a workspace twice, or a non existing one."
				);
				Ok(())
			}
		})
		.await
}

// Now we can begin using WorkspaceFixtures for with a single user.

// #[tokio::test]
// async fn test_delete_workspace_with_users_attached() {
// 	WorkspaceFixture::one("bob", "to-be-deleted")
// 		.with(|(client, workspace): &mut (crate::Client, crate::Workspace)| {

// 			async move {
// 				client.delete_workspace(workspace.id()).await?;

// 				// TODO: I Don't know what should happen here.
// 				Ok(())
// 			}
// 		})
// 		.await
// }

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
						.is_err(),
						"we invited a user to a non existing workspace.");

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
					assert_or_err!(user_list.len() == 2,
						"after inviting alice there should be only two users in a workspace.");
					assert_or_err!(user_list
						.iter()
						.any(|u| u.name == client_alice.current_user().name),
						"alice was invited but is not present as one of the users.");
					assert_or_err!(user_list
						.iter()
						.any(|u| u.name == client_bob.current_user().name),
						"bob owns the workspace but is not present in the workspace.");

					let alice_owned_workspaces = client_alice.fetch_owned_workspaces().await?;
					let alice_invited_workspaces = client_alice.fetch_joined_workspaces().await?;

					assert_or_err!(alice_owned_workspaces.is_empty(),
						"The workspace alice was invided to is listed as owned for her.");
					assert_or_err!(alice_invited_workspaces.contains(&workspace_bob.id()),
						"The list of workspaces to which alice was invited to does not contain the one bob invited her to.");
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
