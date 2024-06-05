use super::*;

av_wrapper!(
	AVPacket,
	ffmpeg_sys_next::AVPacket,
	av_packet_free,
	av_packet_alloc
);

impl AVPacket {
	pub fn unref(&mut self) {
		ffi!(av_packet_unref, self.0.as_mut_ptr());
	}

	pub fn data(&self) -> Ptr<[u8]> {
		#[allow(clippy::unwrap_used)]
		Ptr::slice_from_raw_parts(self.data.cast_const().into(), self.size.try_into().unwrap())
	}
}
