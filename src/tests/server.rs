use super::{
	assert_or_err,
	fixtures::{ClientFixture, ScopedFixture, WorkspaceFixture},
};

#[tokio::test]
async fn test_buffer_create() {
	WorkspaceFixture::one("alice", "test-buffer-create")
		.with(|(_, workspace_alice)| {
			let buffer_name = uuid::Uuid::new_v4().to_string();
			let workspace_alice = workspace_alice.clone();

			async move {
				workspace_alice.create_buffer(&buffer_name).await?;
				assert_or_err!(vec![buffer_name.clone()] == workspace_alice.fetch_buffers().await?);
				workspace_alice.delete_buffer(&buffer_name).await?;

				Ok(())
			}
		})
		.await;
}

#[tokio::test]
async fn test_cant_create_buffer_twice() {
	WorkspaceFixture::one("alice", "test-cant-create-buffer-twice")
		.with(|(_, ws)| {
			let ws = ws.clone();
			async move {
				ws.create_buffer("cacca").await?;
				assert!(
					ws.create_buffer("cacca").await.is_err(),
					"alice could create again the same buffer"
				);
				Ok(())
			}
		})
		.await;
}

#[tokio::test]
#[ignore] // TODO reference server has no concept of buffer ownership yet!
async fn cannot_delete_others_buffers() {
	WorkspaceFixture::two("alice", "bob", "test-cannot-delete-others-buffers")
		.with(|(_, workspace_alice, _, workspace_bob)| {
			let buffer_name = uuid::Uuid::new_v4().to_string();
			let workspace_alice = workspace_alice.clone();
			let workspace_bob = workspace_bob.clone();

			async move {
				workspace_alice.create_buffer(&buffer_name).await?;
				assert_or_err!(workspace_bob.delete_buffer(&buffer_name).await.is_err());
				Ok(())
			}
		})
		.await;
}

#[tokio::test] // TODO split down this test in smaller checks
async fn test_workspace_interactions() {
	if let Err(e) = async {
		let client_alice = ClientFixture::of("alice").setup().await?;
		let client_bob = ClientFixture::of("bob").setup().await?;
		let workspace_name = format!("test-workspace-interactions-{}", uuid::Uuid::new_v4());

		client_alice.create_workspace(&workspace_name).await?;
		let owned_workspaces = client_alice.fetch_owned_workspaces().await?;
		assert_or_err!(owned_workspaces.contains(&workspace_name));
		client_alice.attach_workspace(&workspace_name).await?;
		assert_or_err!(vec![workspace_name.clone()] == client_alice.active_workspaces());

		client_alice
			.invite_to_workspace(&workspace_name, &client_bob.current_user().name)
			.await?;
		client_bob.attach_workspace(&workspace_name).await?;
		assert_or_err!(client_bob
			.fetch_joined_workspaces()
			.await?
			.contains(&workspace_name));

		assert_or_err!(client_bob.leave_workspace(&workspace_name));
		assert_or_err!(client_alice.leave_workspace(&workspace_name));

		client_alice.delete_workspace(&workspace_name).await?;

		Ok::<(), Box<dyn std::error::Error>>(())
	}
	.await
	{
		panic!("{e}");
	}
}
