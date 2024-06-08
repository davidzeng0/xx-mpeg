use super::*;

av_wrapper!(
	AVFrame,
	ffmpeg_sys_next::AVFrame,
	av_frame_free,
	av_frame_alloc
);

impl AVFrame {
	pub fn unref(&mut self) {
		ffi!(av_frame_unref, self.as_mut_ptr());
	}

	pub fn replace(&mut self, frame: &Self) -> Result<()> {
		ffi!(av_frame_replace, self.as_mut_ptr(), frame.0.as_ptr())?;

		Ok(())
	}

	#[allow(dead_code)]
	pub fn move_ref(&mut self, other: &mut Self) {
		ffi!(av_frame_move_ref, other.as_mut_ptr(), self.as_mut_ptr());
	}
}
