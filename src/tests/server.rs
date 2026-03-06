use super::{
	assert_or_err,
	fixtures::{ClientFixture, ScopedFixture, WorkspaceFixture},
};

#[tokio::test]
async fn test_buffer_create() {
	WorkspaceFixture::one("alice")
		.with(|(_, workspace_alice)| {
			let buffer_name = uuid::Uuid::new_v4().to_string();
			let workspace_alice = workspace_alice.clone();

			async move {
				workspace_alice.create_buffer(buffer_name.clone(), false).await?;
				workspace_alice.fetch_buffers().await?;
				assert_or_err!(vec![buffer_name.clone()] == workspace_alice.search_buffers(None));
				workspace_alice.delete_buffer(buffer_name).await?;

				Ok(())
			}
		})
		.await;
}

#[tokio::test]
async fn test_cant_create_buffer_twice() {
	WorkspaceFixture::one("alice")
		.with(|(_, ws)| {
			let ws = ws.clone();
			async move {
				ws.create_buffer("cacca".to_string(), false).await?;
				assert!(
					ws.create_buffer("cacca".to_string(), false).await.is_err(),
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
	WorkspaceFixture::two("alice", "bob")
		.with(|(_, workspace_alice, _, workspace_bob)| {
			let buffer_name = uuid::Uuid::new_v4().to_string();
			let workspace_alice = workspace_alice.clone();
			let workspace_bob = workspace_bob.clone();

			async move {
				workspace_alice.create_buffer(buffer_name.clone(), false).await?;
				assert_or_err!(workspace_bob.delete_buffer(buffer_name).await.is_err());
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
		let wsid = crate::api::WorkspaceIdentifier { user: client_alice.current_user().name.clone(), workspace: workspace_name.clone() };

		client_alice.create_workspace(workspace_name.clone()).await?;
		let owned_workspaces = client_alice.fetch_owned_workspaces().await?;
		assert_or_err!(owned_workspaces.contains(&wsid));
		client_alice.attach_workspace(wsid.clone()).await?;
		assert_or_err!(vec![wsid.clone()] == client_alice.active_workspaces());

		client_alice
			.invite_to_workspace(workspace_name.clone(), client_bob.current_user().name.clone())
			.await?;
		client_bob.attach_workspace(wsid.clone()).await?;
		assert_or_err!(
			client_bob
				.fetch_joined_workspaces()
				.await?
				.contains(&wsid)
		);

		assert_or_err!(client_bob.leave_workspace(&wsid));
		assert_or_err!(client_alice.leave_workspace(&wsid));

		client_alice.delete_workspace(workspace_name.clone()).await?;

		Ok::<(), Box<dyn std::error::Error>>(())
	}
	.await
	{
		panic!("{e}");
	}
}
