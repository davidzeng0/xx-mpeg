#[allow(unused_imports)]
pub use ffmpeg_sys_next::{
	PARSER_FLAG_COMPLETE_FRAMES, PARSER_FLAG_FETCHED_OFFSET, PARSER_FLAG_ONCE,
	PARSER_FLAG_USE_CODEC_TS
};

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

	/// # Panics
	/// if the packet is too large
	#[allow(clippy::needless_pass_by_ref_mut, clippy::unwrap_used)]
	pub fn parse(
		&mut self, codec: &mut CodecContext, mut packet: &[u8], mut pts: i64, mut dts: i64,
		mut pos: i64, duration: &mut u64
	) -> bool {
		let mut data = MutPtr::null().as_mut_ptr();
		let mut size = 0;
		let mut got_packet = false;
		let dur = *duration;

		*duration = 0;

		while !packet.is_empty() {
			#[allow(clippy::cast_sign_loss)]
			/* Safety: FFI */
			let used = unsafe {
				av_parser_parse2(
					self.as_mut_ptr(),
					codec.as_mut_ptr(),
					&mut data,
					&mut size,
					packet.as_ptr(),
					packet.len().try_into().unwrap(),
					pts,
					dts,
					pos
				)
			} as usize;

			pts = UNKNOWN_TIMESTAMP;
			dts = UNKNOWN_TIMESTAMP;
			pos = -1;
			packet = &packet[used..];

			if size == 0 {
				continue;
			}

			if codec.codec_type == AVMediaType::AVMEDIA_TYPE_AUDIO {
				*duration = duration
					.checked_add(self.duration.try_into().unwrap())
					.unwrap();
			} else if self.flags & PARSER_FLAG_COMPLETE_FRAMES != 0 {
				*duration = duration.checked_add(dur).unwrap();
			}

			if got_packet {
				continue;
			}

			got_packet = true;
		}

		got_packet
	}
}
