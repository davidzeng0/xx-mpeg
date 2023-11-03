use std::{
	mem::{transmute, zeroed},
	ops::{Deref, DerefMut},
	ptr::{null, null_mut}
};

use enumflags2::BitFlags;
pub use ffmpeg_sys_next::*;
use xx_core::{error::*, os::error::ErrorCodes, pointer::MutPtr};
pub use AVCodecID::*;

use super::*;

fn result_from_av(code: i32) -> Result<i32> {
	if code >= 0 {
		return Ok(code);
	}

	Err(match code {
		AVERROR_BSF_NOT_FOUND => Error::new(ErrorKind::NotFound, "Bitstream filter not found"),
		AVERROR_BUG | AVERROR_BUG2 => Error::new(ErrorKind::Other, "Internal bug"),
		AVERROR_BUFFER_TOO_SMALL => Error::new(ErrorKind::InvalidInput, "Buffer too small"),
		AVERROR_DECODER_NOT_FOUND => Error::new(ErrorKind::NotFound, "Decoder not found"),
		AVERROR_DEMUXER_NOT_FOUND => Error::new(ErrorKind::NotFound, "Demuxer not found"),
		AVERROR_ENCODER_NOT_FOUND => Error::new(ErrorKind::NotFound, "Encoder not found"),
		AVERROR_EOF => Error::new(ErrorKind::UnexpectedEof, "Eof"),
		AVERROR_EXIT => Error::new(ErrorKind::Other, "Exit requested"),
		AVERROR_EXTERNAL => Error::new(ErrorKind::Other, "Error in external library"),
		AVERROR_FILTER_NOT_FOUND => Error::new(ErrorKind::NotFound, "Filter not found"),
		AVERROR_INVALIDDATA => Error::Simple(ErrorKind::InvalidData),
		AVERROR_MUXER_NOT_FOUND => Error::new(ErrorKind::NotFound, "Muxer not found"),
		AVERROR_OPTION_NOT_FOUND => Error::new(ErrorKind::NotFound, "Option not found"),
		AVERROR_PATCHWELCOME => Error::new(ErrorKind::Unsupported, "Not implemented"),
		AVERROR_PROTOCOL_NOT_FOUND => Error::new(ErrorKind::NotFound, "Protocol not found"),
		AVERROR_STREAM_NOT_FOUND => Error::new(ErrorKind::NotFound, "Stream not found"),
		AVERROR_UNKNOWN => Error::new(
			ErrorKind::Other,
			"Unknown error or error in external library"
		),
		AVERROR_EXPERIMENTAL => Error::new(ErrorKind::Other, "Feature is experimental"),
		AVERROR_INPUT_CHANGED => Error::new(ErrorKind::Other, "Input changed"),
		AVERROR_OUTPUT_CHANGED => Error::new(ErrorKind::Other, "Output changed"),
		AVERROR_HTTP_BAD_REQUEST => Error::new(ErrorKind::Other, "HTTP bad request"),
		AVERROR_HTTP_UNAUTHORIZED => Error::new(ErrorKind::Other, "HTTP unauthorized"),
		AVERROR_HTTP_FORBIDDEN => Error::new(ErrorKind::Other, "HTTP forbidden"),
		AVERROR_HTTP_NOT_FOUND => Error::new(ErrorKind::Other, "HTTP not found"),
		AVERROR_HTTP_OTHER_4XX => Error::new(ErrorKind::Other, "HTTP 4xx"),
		AVERROR_HTTP_SERVER_ERROR => Error::new(ErrorKind::Other, "HTTP server error"),
		code => Error::from_raw_os_error(AVUNERROR(code))
	})
}

struct CodecContext {
	ptr: MutPtr<AVCodecContext>
}

impl CodecContext {
	fn new(codec: *const ffmpeg_sys_next::AVCodec) -> Self {
		Self {
			ptr: unsafe { avcodec_alloc_context3(codec) }.into()
		}
	}
}

impl CodecContext {
	fn open(&mut self) -> Result<()> {
		let result = unsafe { avcodec_open2(self.ptr.as_ptr_mut(), null(), null_mut()) };

		result_from_av(result)?;

		Ok(())
	}

	unsafe fn send_packet(&mut self, packet: &AVPacket) -> Result<()> {
		let result = avcodec_send_packet(self.ptr.as_ptr_mut(), packet);

		result_from_av(result)?;

		Ok(())
	}

	unsafe fn send_frame(&mut self, frame: &AVFrame) -> Result<()> {
		let result = avcodec_send_frame(self.ptr.as_ptr_mut(), frame);

		result_from_av(result)?;

		Ok(())
	}

	fn result_from_av_maybe_none(err: i32) -> Result<bool> {
		const AGAIN: i32 = AVERROR(ErrorCodes::Again as i32);

		match err {
			AVERROR_EOF | AGAIN => Ok(false),
			err => {
				result_from_av(err)?;

				Ok(true)
			}
		}
	}

	unsafe fn receive_packet(&mut self, packet: &mut AVPacket) -> Result<bool> {
		let received =
			Self::result_from_av_maybe_none(avcodec_receive_packet(self.ptr.as_ptr_mut(), packet))?;

		Ok(received)
	}

	unsafe fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		let received =
			Self::result_from_av_maybe_none(avcodec_receive_frame(self.ptr.as_ptr_mut(), frame))?;

		Ok(received)
	}
}

impl Drop for CodecContext {
	fn drop(&mut self) {
		let mut ptr = self.ptr.as_ptr_mut();

		unsafe { avcodec_free_context(&mut ptr) }
	}
}

impl Deref for CodecContext {
	type Target = MutPtr<AVCodecContext>;

	fn deref(&self) -> &Self::Target {
		&self.ptr
	}
}

impl DerefMut for CodecContext {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.ptr
	}
}

pub struct AVCodec {
	id: CodecId,
	context: CodecContext
}

pub fn make_av_rational(rational: &Rational) -> AVRational {
	let (num, den) = rational.parts();

	AVRational { num: num as i32, den: den as i32 }
}

impl AVCodec {
	pub fn new(
		id: CodecId, codec: *const ffmpeg_sys_next::AVCodec, params: &mut CodecParams, mode: Mode
	) -> Result<Self> {
		let mut context = CodecContext::new(codec);

		params
			.config
			.reserve_exact(AV_INPUT_BUFFER_PADDING_SIZE as usize);

		for spare in params.config.spare_capacity_mut() {
			spare.write(0);
		}

		context.time_base = make_av_rational(&params.time_base);
		context.pkt_timebase = make_av_rational(&params.packet_time_base);
		context.bit_rate = params.bit_rate as i64;
		context.bits_per_coded_sample = params.bit_depth as i32;
		context.compression_level = params.compression_level as i32;
		context.delay = params.delay as i32;
		context.seek_preroll = params.seek_preroll as i32;

		context.sample_rate = params.sample_rate as i32;
		context.channels = params.channels as i32;
		context.channel_layout = params.channel_layout;
		context.frame_size = params.frame_size as i32;
		context.sample_fmt = unsafe { transmute(params.sample_format) };
		context.request_sample_fmt = context.sample_fmt;

		if mode == Mode::Decode {
			context.extradata = params.config.as_mut_ptr();
			context.extradata_size = params.config.len() as i32;
		}

		let result = context.open();

		if mode == Mode::Decode {
			context.extradata = null_mut();
			context.extradata_size = 0;
		}

		result?;

		params.time_base =
			Rational::new(context.time_base.num as u32, context.time_base.den as u32);
		params.bit_rate = context.bit_rate as u32;
		params.bit_depth = context.bits_per_coded_sample as u16;
		params.compression_level = context.compression_level as u16;
		params.delay = context.delay as u32;
		params.seek_preroll = context.seek_preroll as u32;

		params.sample_rate = context.sample_rate as u32;
		params.channels = context.channels as u16;
		params.channel_layout = context.channel_layout;
		params.frame_size = context.frame_size as u32;
		params.sample_format = unsafe { transmute(context.sample_fmt) };

		Ok(Self { id, context })
	}
}

impl CodecTrait for AVCodec {
	fn id(&self) -> CodecId {
		self.id
	}

	fn packet_padding(&self) -> usize {
		AV_INPUT_BUFFER_PADDING_SIZE as usize
	}

	fn send_packet(&mut self, packet: &Packet) -> Result<()> {
		unsafe {
			let mut av_packet: AVPacket = zeroed();

			assert!(packet.zero_padding >= self.packet_padding());

			av_packet.time_base = make_av_rational(&packet.time_base);
			av_packet.buf = packet.get_buffer().get_ref() as *const _ as *mut _;
			av_packet.data = packet.data().as_ptr() as *mut _;
			av_packet.size = packet.data().len() as i32;
			av_packet.dts = packet.timestamp as i64;
			av_packet.pts = packet.timestamp as i64 - self.context.delay as i64;
			av_packet.duration = packet.duration as i64;
			av_packet.flags = packet.flags.bits() as i32;

			self.context.send_packet(&av_packet)
		}
	}

	fn send_frame(&mut self, frame: &Frame) -> Result<()> {
		unsafe {
			let mut av_frame: AVFrame = zeroed();

			if self.context.codec_type == AVMediaType::AVMEDIA_TYPE_AUDIO {
				assert_eq!(frame.format, self.context.sample_fmt as i32);
			} else if self.context.codec_type == AVMediaType::AVMEDIA_TYPE_VIDEO {
				assert_eq!(frame.format, self.context.pix_fmt as i32);
			}

			av_frame.nb_samples = frame.samples as i32;
			av_frame.format = frame.format;
			av_frame.pts = frame.timestamp as i64;
			av_frame.pkt_dts = frame.timestamp as i64;
			av_frame.time_base = make_av_rational(&frame.time_base);
			av_frame.sample_rate = frame.sample_rate as i32;
			av_frame.channels = frame.channels as i32;
			av_frame.channel_layout = frame.channel_layout;
			av_frame.flags = frame.flags.bits() as i32;
			av_frame.duration = frame.duration as i64;

			av_frame.extended_buf = frame.extended_bufs().cast();
			av_frame.nb_extended_buf = frame.extended_bufs_size() as i32;

			if frame.extended_data().is_null() {
				av_frame.extended_data = av_frame.data.as_mut_ptr();
			} else {
				av_frame.extended_data = frame.extended_data();
			}

			av_frame.data.copy_from_slice(frame.data());
			av_frame.linesize.copy_from_slice(frame.line_size());
			av_frame.buf.copy_from_slice(transmute(&frame.bufs()[..]));

			self.context.send_frame(&av_frame)
		}
	}

	fn receive_packet(&mut self) -> Result<Option<Packet>> {
		unsafe {
			let mut av_packet: AVPacket = zeroed();

			match self.context.receive_packet(&mut av_packet)? {
				true => (),
				false => return Ok(None)
			}

			let mut packet = Packet::new();
			let buffer = BufferRef::from_buffer_ref(*av_packet.buf);

			packet.set_buffer(buffer, av_packet.data, av_packet.size as usize);
			packet.timestamp = av_packet.dts;
			packet.duration = av_packet.duration as u64;
			packet.flags = BitFlags::from_bits_unchecked(av_packet.flags as u32);
			packet.zero_padding = self.packet_padding();

			av_packet_unref(&mut av_packet);

			Ok(Some(packet))
		}
	}

	fn receive_frame(&mut self) -> Result<Option<Frame>> {
		unsafe {
			let mut av_frame: AVFrame = zeroed();

			match self.context.receive_frame(&mut av_frame)? {
				true => (),
				false => return Ok(None)
			}

			let mut frame = Frame::new();

			frame.samples = av_frame.nb_samples as u32;
			frame.format = av_frame.format;
			frame.timestamp = av_frame.pts;
			frame.time_base =
				Rational::new(av_frame.time_base.num as u32, av_frame.time_base.den as u32);
			frame.sample_rate = av_frame.sample_rate as u32;
			frame.channels = av_frame.channels as u16;
			frame.channel_layout = av_frame.channel_layout;
			frame.flags = BitFlags::from_bits_unchecked(av_frame.flags as u32);
			frame.duration = av_frame.pkt_duration as u64;

			frame.set_extended_bufs(av_frame.extended_buf.cast());
			frame.set_extended_bufs_size(av_frame.nb_extended_buf as usize);

			if av_frame.extended_data != av_frame.data.as_mut_ptr() {
				frame.set_extended_data(av_frame.extended_data);
				av_frame.extended_data = av_frame.data.as_mut_ptr();
			} else {
				frame.data_mut().copy_from_slice(&av_frame.data);
			}

			frame
				.bufs_mut()
				.copy_from_slice(transmute(&av_frame.buf[..]));
			frame.line_size_mut().copy_from_slice(&av_frame.linesize);

			av_frame.extended_buf = null_mut();
			av_frame.nb_extended_buf = 0;
			av_frame.buf = zeroed();

			av_frame_unref(&mut av_frame);

			Ok(Some(frame))
		}
	}

	fn drain(&mut self) -> Result<()> {
		result_from_av(unsafe {
			avcodec_receive_packet(self.context.ptr.as_ptr_mut(), null_mut())
		})?;

		Ok(())
	}

	fn flush(&mut self) -> Result<()> {
		unsafe {
			avcodec_flush_buffers(self.context.ptr.as_ptr_mut());
		}

		Ok(())
	}
}

macro_rules! codec_pair {
	($codec_id: expr, $codec_name: expr, $av_codec: expr, $name: ident) => {
		paste::paste! {
			pub struct [<$name Encoder>];

			impl [<$name Encoder>] {
				pub fn new(params: &mut CodecParams) -> Result<Box<dyn CodecTrait>> {
					let codec_name: Option<&'static str> = $codec_name;
					let mut codec = if let Some(name) = codec_name {
						let name = std::ffi::CString::new(name).unwrap();

						unsafe { avcodec_find_encoder_by_name(name.as_ptr()) }
					} else {
						std::ptr::null()
					};

					if codec.is_null() {
						codec = unsafe { avcodec_find_encoder($av_codec) };
					}

					if !codec.is_null() {
						Ok(Box::new(AVCodec::new($codec_id, codec, params, Mode::Encode)?))
					} else {
						Err(Error::new(ErrorKind::NotFound, "Encoder not found"))
					}
				}
			}

			pub struct [<$name Decoder>];

			impl [<$name Decoder>] {
				pub fn new(params: &mut CodecParams) -> Result<Box<dyn CodecTrait>> {
					let codec_name: Option<&'static str> = $codec_name;
					let mut codec = if let Some(name) = codec_name {
						let name = std::ffi::CString::new(name).unwrap();

						unsafe { avcodec_find_decoder_by_name(name.as_ptr()) }
					} else {
						std::ptr::null()
					};

					if codec.is_null() {
						codec = unsafe { avcodec_find_decoder($av_codec) };
					}

					if !codec.is_null() {
						Ok(Box::new(AVCodec::new($codec_id, codec, params, Mode::Decode)?))
					} else {
						Err(Error::new(ErrorKind::NotFound, "Decoder not found"))
					}
				}
			}
		}
	};
}

pub(super) use codec_pair;
