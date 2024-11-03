use super::{
	assert_or_err,
	fixtures::{ClientFixture, ScopedFixture, WorkspaceFixture},
};
use crate::api::{AsyncReceiver, AsyncSender};
use super::{assert_or_err, fixtures::{ScopedFixture, WorkspaceFixture}};

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
				assert_or_err!(ws.is_some());
				assert_or_err!(ws.unwrap().id() == workspace_name);

				assert_or_err!(client.get_workspace(&wrong_name).is_none());

				let wslist = client.fetch_owned_workspaces().await?;
				assert_or_err!(wslist.len() == 1);
				assert_or_err!(wslist.contains(&workspace_name));
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
				assert_or_err!(client.create_workspace(workspace_name).await.is_err());
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

				assert_or_err!(ws.id() == workspace_name);
				assert_or_err!(client.active_workspaces().contains(&workspace_name));
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
				assert_or_err!(client.attach_workspace(&workspace_name).await.is_err());
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
				let ws = res.unwrap();
				assert_or_err!(
					!client.leave_workspace(&workspace_name),
					"leaving a workspace while some reference still exist returned true."
				);

				Ok(())
			}
		})
		.await
}
				Ok(())
			}
		})
		.await;
}

#[tokio::test]
async fn test_send_operation() {
	WorkspaceFixture::two("alice", "bob", "test-send-operation")
		.with(|((_, workspace_alice), (_, workspace_bob))| {
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
		.with(|((_, workspace_alice), (_, workspace_bob))| {
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
