use super::*;

pub struct Codecs;

impl Codecs {
	pub fn find_encoder(id: AVCodecID) -> Option<Ptr<AVCodec>> {
		find_with(|| ffi!(avcodec_find_encoder, id))
	}

	pub fn find_decoder(id: AVCodecID) -> Option<Ptr<AVCodec>> {
		find_with(|| ffi!(avcodec_find_decoder, id))
	}

	pub fn find_encoder_by_name(name: &str) -> Option<Ptr<AVCodec>> {
		Self::find_encoder_by_name_c(&into_cstr(name))
	}

	pub fn find_encoder_by_name_c(name: &CStr) -> Option<Ptr<AVCodec>> {
		find_with(|| ffi!(avcodec_find_encoder_by_name, name.as_ptr()))
	}

	pub fn find_decoder_by_name(name: &str) -> Option<Ptr<AVCodec>> {
		Self::find_decoder_by_name_c(&into_cstr(name))
	}

	pub fn find_decoder_by_name_c(name: &CStr) -> Option<Ptr<AVCodec>> {
		find_with(|| ffi!(avcodec_find_decoder_by_name, name.as_ptr()))
	}
}

av_wrapper!(CodecContext, AVCodecContext, avcodec_free_context);

impl CodecContext {
	pub fn new(codec: Ptr<AVCodec>) -> Self {
		let ptr = alloc_with(|| ffi!(avcodec_alloc_context3, codec.as_ptr()));

		Self(ptr)
	}

	pub fn open(&mut self) -> Result<()> {
		ffi!(
			avcodec_open2,
			self.0.as_mut_ptr(),
			Ptr::null().as_ptr(),
			MutPtr::null().as_mut_ptr()
		)?;

		Ok(())
	}

	pub unsafe fn send_packet(&mut self, packet: &AVPacket) -> Result<()> {
		ffi!(avcodec_send_packet, self.0.as_mut_ptr(), &**packet)?;

		Ok(())
	}

	pub unsafe fn send_frame(&mut self, frame: &AVFrame) -> Result<()> {
		ffi!(avcodec_send_frame, self.0.as_mut_ptr(), &**frame)?;

		Ok(())
	}

	pub unsafe fn receive_packet(&mut self, packet: &mut AVPacket) -> Result<bool> {
		ffi_optional!(avcodec_receive_packet, self.0.as_mut_ptr(), &mut **packet)
	}

	pub unsafe fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		ffi_optional!(avcodec_receive_frame, self.0.as_mut_ptr(), &mut **frame)
	}

	pub fn drain(&mut self) -> Result<()> {
		ffi!(
			avcodec_send_packet,
			self.0.as_mut_ptr(),
			MutPtr::null().as_mut_ptr()
		)?;

		Ok(())
	}

	pub fn flush(&mut self) {
		ffi!(avcodec_flush_buffers, self.0.as_mut_ptr());
	}
}
