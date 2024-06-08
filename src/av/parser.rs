#![allow(clippy::module_name_repetitions)]

use super::*;

pub struct ParserContext(MutNonNull<AVCodecParserContext>);

impl Drop for ParserContext {
	fn drop(&mut self) {
		ffi!(av_parser_close, self.as_mut_ptr());
	}
}

ptr_deref!(ParserContext, AVCodecParserContext);

impl ParserContext {
	#[allow(dead_code)]
	pub fn new(codec: AVCodecID) -> Self {
		let ptr = alloc_with(|| ffi!(av_parser_init, codec as i32));

		Self(ptr)
	}

	pub fn try_new(codec: AVCodecID) -> Option<Self> {
		MutNonNull::new(ffi!(av_parser_init, codec as i32).into()).map(Self)
	}

	#[allow(clippy::needless_pass_by_ref_mut)]
	pub fn parse(&mut self, _: &[u8]) -> Result<()> {
		todo!();
	}
}
