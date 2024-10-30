use std::{error::Error, fmt::Display, future::Future};

use crate::api::{AsyncReceiver, AsyncSender};

#[derive(Debug)]
pub struct AssertionError(String);
impl AssertionError {
	fn new(msg: &str) -> Self {
		Self(msg.to_string())
	}
}
impl Display for AssertionError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}", self.0)
	}
}
impl std::error::Error for AssertionError {}

#[allow(async_fn_in_trait)]
pub trait ScopedFixture<T: Sized> {
	async fn setup(&mut self) -> Result<T, Box<dyn Error>>;

	async fn cleanup(&mut self, resource: Option<T>) {
		drop(resource)
	}

	async fn inner_with<F>(mut self, cb: impl FnOnce(&mut T) -> F) -> Result<(), Box<dyn Error>>
	where
		Self: Sized,
		F: Future<Output = Result<(), Box<dyn Error>>>,
	{
		match self.setup().await {
			Ok(mut t) => {
				let res = cb(&mut t).await;
				self.cleanup(Some(t)).await;
				res
			}
			Err(e) => {
				self.cleanup(None).await;
				Err(e)
			}
		}
	}

	async fn with<F>(self, cb: impl FnOnce(&mut T) -> F)
	where
		Self: Sized,
		F: Future<Output = Result<(), Box<dyn std::error::Error>>>,
	{
		if let Err(e) = self.inner_with(cb).await {
			panic!("{e}");
		}
	}
}

macro_rules! assert_or_err {
	($s:expr) => {
		if !$s {
			return Err(AssertionError::new(&format!(
				"assertion failed at line {}: {}",
				std::line!(),
				stringify!($s)
			))
			.into());
		}
	};
	($s:expr, $msg:literal) => {
		if !$s {
			return Err(AssertionError::new($msg).into());
		}
	};
}

struct ClientFixture {
	name: String,
	username: Option<String>,
	password: Option<String>,
}
impl ClientFixture {
	fn of(name: &str) -> Self {
		Self {
			name: name.to_string(),
			username: None,
			password: None,
		}
	}
}

impl ScopedFixture<crate::Client> for ClientFixture {
	async fn setup(&mut self) -> Result<crate::Client, Box<dyn Error>> {
		let upper = self.name.to_uppercase();
		let username = self.username.clone().unwrap_or_else(|| {
			std::env::var(format!("CODEMP_TEST_USERNAME_{upper}")).unwrap_or_default()
		});
		let password = self.password.clone().unwrap_or_else(|| {
			std::env::var(format!("CODEMP_TEST_PASSWORD_{upper}")).unwrap_or_default()
		});
		let client = crate::Client::connect(crate::api::Config {
			username,
			password,
			tls: Some(false),
			..Default::default()
		})
		.await?;

		Ok(client)
	}
}

struct WorkspaceFixture {
	user: String,
	invite: Option<String>,
	workspace: String,
}
impl WorkspaceFixture {
	#[allow(unused)]
	fn of(user: &str, invite: &str, workspace: &str) -> Self {
		Self {
			user: user.to_string(),
			invite: Some(invite.to_string()),
			workspace: workspace.to_string(),
		}
	}

	fn any(user: &str) -> Self {
		Self {
			user: user.to_string(),
			invite: None,
			workspace: uuid::Uuid::new_v4().to_string(),
		}
	}

	fn two(user: &str, invite: &str) -> Self {
		Self {
			user: user.to_string(),
			invite: Some(invite.to_string()),
			workspace: uuid::Uuid::new_v4().to_string(),
		}
	}
}

impl ScopedFixture<(crate::Client, crate::Workspace)> for WorkspaceFixture {
	async fn setup(&mut self) -> Result<(crate::Client, crate::Workspace), Box<dyn Error>> {
		let client = ClientFixture::of(&self.user).setup().await?;
		client.create_workspace(&self.workspace).await?;
		let workspace = client.attach_workspace(&self.workspace).await?;
		Ok((client, workspace))
	}

	async fn cleanup(&mut self, resource: Option<(crate::Client, crate::Workspace)>) {
		if let Some((client, _workspace)) = resource {
			client.leave_workspace(&self.workspace);
			if let Err(e) = client.delete_workspace(&self.workspace).await {
				eprintln!("could not delete workspace: {e}");
			}
		}
	}
}

impl ScopedFixture<((crate::Client, crate::Workspace), (crate::Client, crate::Workspace))> for WorkspaceFixture {
	async fn setup(&mut self) -> Result<((crate::Client, crate::Workspace), (crate::Client, crate::Workspace)), Box<dyn Error>> {
		let client = ClientFixture::of(&self.user).setup().await?;
		let invite_client = ClientFixture::of(
			&self
				.invite
				.clone()
				.unwrap_or(uuid::Uuid::new_v4().to_string()),
		)
		.setup()
		.await?;
		client.create_workspace(&self.workspace).await?;
		client
			.invite_to_workspace(&self.workspace, invite_client.current_user().name.clone())
			.await?;
		let workspace = client.attach_workspace(&self.workspace).await?;
		let invite = invite_client.attach_workspace(&self.workspace).await?;
		Ok(((client, workspace), (invite_client, invite)))
	}

	async fn cleanup(&mut self, resource: Option<((crate::Client, crate::Workspace), (crate::Client, crate::Workspace))>) {
		if let Some(((client, _workspace), (_, _))) = resource {
			client.leave_workspace(&self.workspace);
			if let Err(e) = client.delete_workspace(&self.workspace).await {
				eprintln!("could not delete workspace: {e}");
			}
		}
	}
}

#[tokio::test]
async fn test_workspace_interactions() {
	if let Err(e) = async {
		let client_alice = ClientFixture::of("alice").setup().await?;
		let client_bob = ClientFixture::of("bob").setup().await?;
		let workspace_name = uuid::Uuid::new_v4().to_string();

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

// =======

#[tokio::test]
async fn cannot_delete_others_workspaces() {
	WorkspaceFixture::two("alice", "bob")
		.with(|((_, ws_alice), (client_bob, _))| {
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
async fn test_cant_create_buffer_twice() {
	WorkspaceFixture::any("alice")
		.with(|(_, ws): &mut (crate::Client, crate::Workspace)| {
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
async fn test_buffer_search() {
	WorkspaceFixture::two("alice", "bob")
		.with(|((_, workspace_alice), (_, workspace_bob))| {
			let buffer_name = uuid::Uuid::new_v4().to_string();
			let workspace_alice = workspace_alice.clone();
			let workspace_bob = workspace_bob.clone();

			async move {
				workspace_alice.create_buffer(&buffer_name).await?;
				assert_or_err!(vec![buffer_name.clone()] == workspace_alice.fetch_buffers().await?);
				assert_or_err!(vec![buffer_name.clone()] == workspace_bob.fetch_buffers().await?);

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
async fn cannot_delete_others_buffers() {
	WorkspaceFixture::two("alice", "bob")
		.with(|((_, workspace_alice), (_, workspace_bob))| {
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

// =====

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
