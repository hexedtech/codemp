use std::{error::Error, future::Future};

// TODO create a BufferFixture too

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

pub struct ClientFixture {
	name: String,
	username: Option<String>,
	password: Option<String>,
}
impl ClientFixture {
	pub fn of(name: &str) -> Self {
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

pub struct WorkspaceFixture {
	user: String,
	invite: Option<String>,
	workspace: String,
}
impl WorkspaceFixture {
	pub fn of(user: &str, invite: &str, workspace: &str) -> Self {
		Self {
			user: user.to_string(),
			invite: Some(invite.to_string()),
			workspace: workspace.to_string(),
		}
	}

	pub fn one(user: &str, ws: &str) -> Self {
		Self {
			user: user.to_string(),
			invite: None,
			workspace: format!("{ws}-{}", uuid::Uuid::new_v4()),
		}
	}

	pub fn two(user: &str, invite: &str, ws: &str) -> Self {
		Self {
			user: user.to_string(),
			invite: Some(invite.to_string()),
			workspace: format!("{ws}-{}", uuid::Uuid::new_v4()),
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
