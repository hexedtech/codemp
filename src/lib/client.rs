use std::{future::Future, sync::Arc, pin::Pin};

use operational_transform::OperationSeq;
use tokio::sync::mpsc;
use tonic::{transport::{Channel, Error}, Status};
use tracing::error;

use crate::{proto::{buffer_client::BufferClient, BufferPayload, OperationRequest, RawOp, CursorMov}, opfactory::AsyncFactory};

pub trait EditorDriver : Clone {
	fn id(&self) -> String;
}

#[derive(Clone)]
pub struct CodempClient<T : EditorDriver> {
	client: BufferClient<Channel>,
	driver: T,
}

impl<T : EditorDriver> CodempClient<T> { // TODO wrap tonic 'connect' to allow multiple types
	pub async fn new(addr: &str, driver: T) -> Result<Self, Error> {
		let client = BufferClient::connect(addr.to_string()).await?;

		Ok(CodempClient { client, driver })
	}

	pub async fn create_buffer(&mut self, path: String, content: Option<String>) -> Result<bool, Status> {
		let req = BufferPayload {
			path, content,
			user: self.driver.id(),
		};

		let res = self.client.create(req).await?;

		Ok(res.into_inner().accepted)
	}

	pub async fn attach_buffer(&mut self, path: String) -> Result<mpsc::Sender<OperationSeq>, Status> {
		let req = BufferPayload {
			path, content: None,
			user: self.driver.id(),
		};

		let content = self.client.sync(req.clone())
			.await?
			.into_inner()
			.content;

		let mut stream = self.client.attach(req).await?.into_inner();

		let factory = Arc::new(AsyncFactory::new(content));

		let (tx, mut rx) = mpsc::channel(64);

		tokio::spawn(async move {
			loop {
				match stream.message().await {
					Err(e)      => break error!("error receiving update: {}", e),
					Ok(None)    => break,
					Ok(Some(x)) => match serde_json::from_str::<OperationSeq>(&x.opseq) {
						Err(e) => break error!("error deserializing opseq: {}", e),
						Ok(v) => match factory.process(v).await {
							Err(e) => break error!("could not apply operation from server: {}", e),
							Ok(txt) => { // send back txt
							}
						}
					},
				}
			}
		});

		tokio::spawn(async move {
			while let Some(op) = rx.recv().await {

			}
		});

		Ok(tx)
	}
}
