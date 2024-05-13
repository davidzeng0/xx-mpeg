use super::*;

av_wrapper!(
	AVFrame,
	ffmpeg_sys_next::AVFrame,
	av_frame_free,
	av_frame_alloc
);

impl AVFrame {
	pub fn unref(&mut self) {
		/* Safety: FFI call */
		unsafe { av_frame_unref(self.0.as_mut_ptr()) };
	}

	pub fn replace(&mut self, frame: &Self) -> Result<()> {
		/* Safety: FFI call */
		let ret = unsafe { av_frame_replace(self.0.as_mut_ptr(), frame.0.as_ptr()) };

		result_from_av(ret).map(|_| ())
	}
}
