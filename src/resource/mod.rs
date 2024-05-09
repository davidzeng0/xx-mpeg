#![allow(clippy::module_name_repetitions)]

use xx_core::async_std::io::*;

use super::*;

pub mod http;
pub use http::*;

pub const DEFAULT_SEEK_THRESHOLD: u64 = 512 * 1024;

pub trait StreamImpl: Read + Seek {
	fn suggested_seek_threshold(&self) -> u64 {
		DEFAULT_SEEK_THRESHOLD
	}

	/// Returns `true` if the stream may be seekable
	fn seekable(&self) -> bool {
		false
	}
}

#[asynchronous]
pub trait ResourceImpl {
	async fn create_stream(&self) -> Result<Stream>;
}

impl Read for Stream {
	read_wrapper! {
		inner = as_ref();
		mut inner = as_mut();
	}
}

impl Seek for Stream {
	seek_wrapper! {
		inner = as_ref();
		mut inner = as_mut();
	}
}

pub type Stream = Box<dyn StreamImpl>;
pub type Resource = Box<dyn ResourceImpl>;
