use super::*;

pub struct Codecs;

impl Codecs {
	pub fn find_encoder(id: AVCodecID) -> Option<NonNull<AVCodec>> {
		NonNull::new(ffi!(avcodec_find_encoder, id).into())
	}

	pub fn find_decoder(id: AVCodecID) -> Option<NonNull<AVCodec>> {
		NonNull::new(ffi!(avcodec_find_decoder, id).into())
	}

	pub fn find_encoder_by_name(name: &str) -> Option<NonNull<AVCodec>> {
		Self::find_encoder_by_name_c(&into_cstr(name))
	}

	pub fn find_encoder_by_name_c(name: &CStr) -> Option<NonNull<AVCodec>> {
		NonNull::new(ffi!(avcodec_find_encoder_by_name, name.as_ptr()).into())
	}

	pub fn find_decoder_by_name(name: &str) -> Option<NonNull<AVCodec>> {
		Self::find_decoder_by_name_c(&into_cstr(name))
	}

	pub fn find_decoder_by_name_c(name: &CStr) -> Option<NonNull<AVCodec>> {
		NonNull::new(ffi!(avcodec_find_decoder_by_name, name.as_ptr()).into())
	}
}

av_wrapper!(CodecContext, AVCodecContext, avcodec_free_context);

impl CodecContext {
	pub fn new(codec: NonNull<AVCodec>) -> Self {
		let ptr = alloc_with(|| ffi!(avcodec_alloc_context3, codec.as_ptr()));

		Self(ptr)
	}

	pub fn open(&mut self) -> Result<()> {
		ffi!(
			avcodec_open2,
			self.as_mut_ptr(),
			Ptr::null().as_ptr(),
			MutPtr::null().as_mut_ptr()
		)?;

		Ok(())
	}

	/// # Safety
	/// packet contains raw pointers and must be valid
	pub unsafe fn send_packet(&mut self, packet: &AVPacket) -> Result<()> {
		ffi!(avcodec_send_packet, self.as_mut_ptr(), &**packet)?;

		Ok(())
	}

	/// # Safety
	/// frame contains raw pointers and must be valid
	pub unsafe fn send_frame(&mut self, frame: &AVFrame) -> Result<()> {
		ffi!(avcodec_send_frame, self.as_mut_ptr(), &**frame)?;

		Ok(())
	}

	/// # Safety
	/// packet contains raw pointers and must be valid
	pub unsafe fn receive_packet(&mut self, packet: &mut AVPacket) -> Result<bool> {
		ffi_optional!(avcodec_receive_packet, self.as_mut_ptr(), &mut **packet)
	}

	/// # Safety
	/// frame contains raw pointers and must be valid
	pub unsafe fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		ffi_optional!(avcodec_receive_frame, self.as_mut_ptr(), &mut **frame)
	}

	pub fn drain(&mut self) -> Result<()> {
		/* Safety: valid ptr */
		let is_decoder = unsafe { av_codec_is_decoder(self.codec) } != 0;

		if is_decoder {
			ffi!(
				avcodec_send_packet,
				self.as_mut_ptr(),
				MutPtr::null().as_mut_ptr()
			)?;
		} else {
			ffi!(
				avcodec_send_frame,
				self.as_mut_ptr(),
				MutPtr::null().as_mut_ptr()
			)?;
		}

		Ok(())
	}

	pub fn flush(&mut self) {
		ffi!(avcodec_flush_buffers, self.as_mut_ptr());
	}
}
