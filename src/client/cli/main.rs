use clap::Parser;
use library::proto::{buffer_client::BufferClient, BufferPayload};
use tokio_stream::StreamExt;

#[derive(Parser, Debug)]
struct CliArgs {
	/// path of buffer to create
	path: String,

	/// initial content for buffer
	#[arg(short, long)]
	content: Option<String>,

	/// attach instead of creating a new buffer
	#[arg(long, default_value_t = false)]
	attach: bool,

	/// host to connect to
	#[arg(long, default_value = "http://[::1]:50051")]
	host: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = CliArgs::parse();

	let mut client = BufferClient::connect(args.host).await?;

	let request = BufferPayload {
		path: args.path,
		content: args.content,
	};

	if !args.attach {
		client.create(request.clone()).await.unwrap();
	}

	let mut stream = client.attach(request).await.unwrap().into_inner();

	while let Some(item) = stream.next().await {
		println!("> {:?}", item);
	}

	Ok(())
}

