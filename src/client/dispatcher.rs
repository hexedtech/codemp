pub mod proto {
	tonic::include_proto!("session");
	tonic::include_proto!("workspace");
	tonic::include_proto!("buffer");
}
use std::sync::Arc;
use tracing::error;

use tokio::sync::{mpsc, Mutex};
use tokio_stream::StreamExt;
use tokio_stream::wrappers::ReceiverStream;

use proto::{
	workspace_client::WorkspaceClient,
	session_client::SessionClient,
	buffer_client::BufferClient,
	WorkspaceBuilderRequest, JoinRequest, SessionResponse, CursorUpdate
};
use tonic::{transport::Channel, Status, Request, Response};

#[derive(Clone)]
pub struct Dispatcher {
	name: String,
	dp: Arc<Mutex<DispatcherWorker>>, // TODO use channels and don't lock
}

struct DispatcherWorker {
	// TODO do I need all three? Did I design the server badly?
	session: SessionClient<Channel>,
	workspace: WorkspaceClient<Channel>,
	_buffers: BufferClient<Channel>,
}

impl Dispatcher {
	pub async fn connect(addr:String) -> Result<Dispatcher, tonic::transport::Error> {
		let (s, w, b) = tokio::join!(
			SessionClient::connect(addr.clone()),
			WorkspaceClient::connect(addr.clone()),
			BufferClient::connect(addr.clone()),
		);
		Ok(
			Dispatcher { 
				name: format!("User#{}", rand::random::<u16>()),
				dp: Arc::new(
					Mutex::new(
						DispatcherWorker { session: s?, workspace: w?, _buffers: b? }
					)
				)
			}
		)
	}

	pub async fn create_workspace(&self, name:String) -> Result<Response<SessionResponse>, Status> {
		self.dp.lock().await.session.create_workspace(
			Request::new(WorkspaceBuilderRequest { name })
		).await
	}

	pub async fn join_workspace(&self, session_id:String) -> Result<(), Status> {
		let mut req = Request::new(JoinRequest { name: self.name.clone() });
		req.metadata_mut().append("workspace", session_id.parse().unwrap());
		let mut stream = self.dp.lock().await.workspace.join(req).await?.into_inner();

		let _worker = tokio::spawn(async move {
			while let Some(pkt) = stream.next().await {
				match pkt {
					Ok(_event) => {
						// TODO do something with events when they will mean something!
					},
					Err(e) => error!("Error receiving event | {}", e),
				}
			}
		});

		Ok(())
	}

	pub async fn start_cursor_worker(&self, session_id:String, feed:mpsc::Receiver<CursorUpdate>) -> Result<mpsc::Receiver<CursorUpdate>, Status> {
		let mut in_stream = Request::new(ReceiverStream::new(feed));
		in_stream.metadata_mut().append("workspace", session_id.parse().unwrap());
		
		let mut stream = self.dp.lock().await.workspace.subscribe(in_stream).await?.into_inner();
		let (tx, rx) = mpsc::channel(50);

		let _worker = tokio::spawn(async move {
			while let Some(pkt) = stream.next().await {
				match pkt {
					Ok(update) => tx.send(update).await.unwrap(), // TODO how to handle an error here?
					Err(e) => error!("Error receiving cursor update | {}", e),
				}
			}
		});

		Ok(rx)
	}
}
