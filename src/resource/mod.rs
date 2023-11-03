use xx_core::{async_std::io::*, error::*, read_wrapper, seek_wrapper};
use xx_pulse::*;

mod http;
pub use http::*;

use crate::reader::DEFAULT_SEEK_THRESHOLD;

pub trait StreamTrait: Read + Seek {
	fn suggested_seek_threshold(&self) -> u64 {
		DEFAULT_SEEK_THRESHOLD
	}
}

#[async_trait]
pub trait ResourceTrait {
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

pub type Stream = Box<dyn StreamTrait>;
pub type Resource = Box<dyn ResourceTrait>;
