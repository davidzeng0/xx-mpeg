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

#[asynchronous(impl(ref, mut, box))]
pub trait ResourceImpl {
	async fn create_stream(&self) -> Result<Stream>;
}

pub type Stream = Box<dyn StreamImpl + Send + Sync>;
pub type Resource = Box<dyn ResourceImpl + Send + Sync>;
