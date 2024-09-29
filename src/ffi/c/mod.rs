use std::ffi::{c_char, CString};

use crate::{api::Config, Client, Workspace};

pub(crate) fn tokio() -> &'static tokio::runtime::Runtime {
	use std::sync::OnceLock;
	static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
	RT.get_or_init(||
		tokio::runtime::Builder::new_current_thread()
			.enable_all()
			.build()
			.expect("could not create tokio runtime")
	)
}


#[no_mangle] // TODO config
pub extern "C" fn Codemp_Client_connect() -> *mut Client {
	match tokio()
		.block_on(Client::connect(Config::new("", "")))
	{
		Ok(c) => Box::into_raw(Box::new(c)),
		Err(e) => {
			tracing::error!("failed connecting to remote: {e}");
			std::ptr::null_mut()
		},
	}
}

#[no_mangle]
pub unsafe extern "C" fn Codemp_Client_join_workspace(client: *mut Client, workspace: *mut c_char) -> *mut Workspace {
	let client = unsafe { Box::leak(Box::from_raw(client)) };
	let workspace = unsafe { CString::from_raw(workspace) }.to_string_lossy().to_string();

	match tokio()
		.block_on(client.join_workspace(workspace))
	{
		Ok(ws) => Box::into_raw(Box::new(ws)),
		Err(e) => {
			tracing::error!("failed joining workspace: {e}");
			std::ptr::null_mut()
		},
	}

}
