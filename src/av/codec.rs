use super::*;

av_wrapper!(CodecContext, AVCodecContext, avcodec_free_context);

impl CodecContext {
	pub fn new(codec: Ptr<AVCodec>) -> Self {
		/* Safety: FFI call */
		let ptr = alloc_with(|| unsafe { avcodec_alloc_context3(codec.as_ptr()) });

		Self(ptr)
	}

	pub fn open(&mut self) -> Result<()> {
		/* Safety: FFI call */
		let result = unsafe {
			avcodec_open2(
				self.0.as_mut_ptr(),
				Ptr::null().as_ptr(),
				MutPtr::null().as_mut_ptr()
			)
		};

		result_from_av(result)?;

		Ok(())
	}

	pub unsafe fn send_packet(&mut self, packet: &AVPacket) -> Result<()> {
		/* Safety: FFI call */
		let result = unsafe { avcodec_send_packet(self.0.as_mut_ptr(), &**packet) };

		result_from_av(result)?;

		Ok(())
	}

	pub unsafe fn send_frame(&mut self, frame: &AVFrame) -> Result<()> {
		/* Safety: FFI call */
		let result = unsafe { avcodec_send_frame(self.0.as_mut_ptr(), &**frame) };

		result_from_av(result)?;

		Ok(())
	}

	pub unsafe fn receive_packet(&mut self, packet: &mut AVPacket) -> Result<bool> {
		/* Safety: FFI call */
		let ret = unsafe { avcodec_receive_packet(self.0.as_mut_ptr(), &mut **packet) };

		result_from_av_maybe_none(ret)
	}

	pub unsafe fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		/* Safety: FFI call */
		let ret = unsafe { avcodec_receive_frame(self.0.as_mut_ptr(), &mut **frame) };

		result_from_av_maybe_none(ret)
	}

	pub unsafe fn drain(&mut self) -> Result<()> {
		/* Safety: FFI call */
		let ret = unsafe { avcodec_send_packet(self.0.as_mut_ptr(), MutPtr::null().as_mut_ptr()) };

		result_from_av(ret)?;

		Ok(())
	}

	pub unsafe fn flush(&mut self) {
		/* Safety: FFI call */
		unsafe { avcodec_flush_buffers(self.0.as_mut_ptr()) };
	}
}
