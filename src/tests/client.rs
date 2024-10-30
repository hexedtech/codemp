use crate::api::{AsyncReceiver, AsyncSender};
use super::{assert_or_err, fixtures::{ScopedFixture, WorkspaceFixture}};

#[tokio::test]
async fn test_buffer_search() {
	WorkspaceFixture::one("alice")
		.with(|(_, workspace_alice): &mut (crate::Client, crate::Workspace)| {
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
	WorkspaceFixture::two("alice", "bob")
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
	WorkspaceFixture::two("alice", "bob")
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

				for i in 0..100 {
					tokio::time::sleep(std::time::Duration::from_millis(50)).await;
					match bob.try_recv().await? {
						Some(change) => bob.ack(change.version),
						None => break,
					}
					eprintln!("bob more to recv at attempt #{i}");
				}

				for i in 0..100 {
					tokio::time::sleep(std::time::Duration::from_millis(50)).await;
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
				assert_or_err!(false);

				Ok(())
			}
		})
		.await;
}
