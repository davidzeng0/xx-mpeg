#![allow(unreachable_pub)]

use ffmpeg_sys_next::AVMediaType;
use xx_core::{error::*, pointer::*};

use super::*;
pub use crate::av::{AVCodecID::*, AVFrame, AVPacket, CodecContext, INPUT_BUFFER_PADDING};

pub struct AVCodec {
	id: CodecId,
	context: CodecContext,
	packet: AVPacket,
	frame: AVFrame
}

impl AVCodec {
	#[allow(clippy::unwrap_used)]
	pub fn new(
		id: CodecId, codec: Ptr<ffmpeg_sys_next::AVCodec>, params: &mut CodecParams, mode: Mode
	) -> Result<Self> {
		let mut context = CodecContext::new(codec);

		params.config.reserve_exact(INPUT_BUFFER_PADDING);

		for spare in params
			.config
			.spare_capacity_mut()
			.iter_mut()
			.take(INPUT_BUFFER_PADDING)
		{
			spare.write(0);
		}

		context.time_base = params.time_base.into();
		context.pkt_timebase = params.packet_time_base.into();
		context.bit_rate = params.bit_rate as i64;
		context.bits_per_raw_sample = params.bit_depth as i32;
		context.compression_level = params.compression_level as i32;
		context.delay = params.delay.try_into().unwrap();
		context.seek_preroll = params.seek_preroll.try_into().unwrap();

		context.sample_rate = params.sample_rate.try_into().unwrap();
		context.channels = params.channels as i32;
		context.channel_layout = params.channel_layout;
		context.frame_size = params.frame_size.try_into().unwrap();
		context.sample_fmt = params.sample_format.into();
		context.request_sample_fmt = context.sample_fmt;

		context.width = params.width.try_into().unwrap();
		context.height = params.height.try_into().unwrap();
		context.sample_aspect_ratio = params.sample_aspect_ratio.into();
		context.frame_num = params.frame_size.into();
		context.pix_fmt = params.pixel_format.into();

		if mode == Mode::Decode {
			context.extradata_size = params.config.len().try_into().unwrap();
			context.extradata = params.config.as_mut_ptr();
		}

		let result = context.open();

		if mode == Mode::Decode {
			context.extradata = MutPtr::null().as_mut_ptr();
			context.extradata_size = 0;
		}

		result?;

		params.time_base = context.time_base.into();
		params.bit_rate = context.bit_rate.try_into().unwrap();
		params.bit_depth = context.bits_per_raw_sample.try_into().unwrap();
		params.compression_level = context.compression_level.try_into().unwrap();
		params.delay = context.delay.try_into().unwrap();
		params.seek_preroll = context.seek_preroll.try_into().unwrap();

		params.sample_rate = context.sample_rate.try_into().unwrap();
		params.channels = context.channels.try_into().unwrap();
		params.channel_layout = context.channel_layout;
		params.frame_size = context.frame_size.try_into().unwrap();
		params.sample_format = context.sample_fmt.into();

		params.width = context.width.try_into().unwrap();
		params.height = context.height.try_into().unwrap();
		params.sample_aspect_ratio = context.sample_aspect_ratio.into();
		params.framerate = context.framerate.into();
		params.pixel_format = context.pix_fmt.into();

		Ok(Self {
			id,
			context,
			packet: AVPacket::new(),
			frame: AVFrame::new()
		})
	}
}

#[allow(clippy::unwrap_used, clippy::cast_sign_loss)]
impl CodecImpl for AVCodec {
	fn id(&self) -> CodecId {
		self.id
	}

	#[allow(clippy::arithmetic_side_effects, clippy::cast_possible_wrap)]
	fn send_packet(&mut self, packet: &Packet) -> Result<()> {
		self.packet.time_base = packet.time_base.into();
		self.packet.data = packet.data.as_ptr().cast_mut().cast();
		self.packet.size = packet.data.len().try_into().unwrap();
		self.packet.dts = packet.timestamp;
		self.packet.pts = packet.timestamp - self.context.delay as i64;
		self.packet.duration = packet.duration.try_into().unwrap();
		self.packet.flags = packet.flags.bits() as i32;

		/* Safety: all data is valid */
		unsafe { self.context.send_packet(&self.packet) }
	}

	fn send_frame(&mut self, frame: &Frame) -> Result<()> {
		self.frame.replace(&frame.data)?;

		self.frame.time_base = frame.time_base.into();
		self.frame.pkt_dts = frame.decode_timestamp;
		self.frame.pts = frame.presentation_timestamp;
		self.frame.duration = frame.duration.try_into().unwrap();

		self.frame.sample_rate = frame.sample_rate.try_into().unwrap();
		self.frame.channel_layout = frame.channel_layout;

		self.frame.pict_type = frame.picture_type.into();
		self.frame.sample_aspect_ratio = frame.sample_aspect_ratio.into();
		self.frame.repeat_pict = frame.repeat_picture;

		/* Safety: all data is valid */
		unsafe { self.context.send_frame(&self.frame)? };

		self.frame.unref();

		Ok(())
	}

	fn receive_packet(&mut self, packet: &mut Packet) -> Result<bool> {
		/* Safety: FFI call */
		if !unsafe { self.context.receive_packet(&mut self.packet)? } {
			return Ok(false);
		}

		/* Safety: non-null */
		packet.data = unsafe { self.packet.data().as_ref().to_vec() };
		packet.timestamp = self.packet.dts;
		packet.duration = self.packet.duration.try_into().unwrap();
		packet.flags = BitFlags::from_bits_truncate(self.packet.flags as u32);

		self.packet.unref();

		Ok(true)
	}

	fn receive_frame(&mut self, frame: &mut Frame) -> Result<bool> {
		/* Safety: FFI call */
		if !unsafe { self.context.receive_frame(&mut frame.data)? } {
			return Ok(false);
		}

		frame.time_base = frame.data.time_base.into();
		frame.decode_timestamp = frame.data.pkt_dts;
		frame.presentation_timestamp = frame.data.pts;
		frame.duration = frame.data.duration.try_into().unwrap();
		frame.flags = BitFlags::from_bits_truncate(frame.data.flags as u32);

		frame.samples = frame.data.nb_samples.try_into().unwrap();
		frame.sample_rate = frame.data.sample_rate.try_into().unwrap();
		frame.channels = frame.data.channels.try_into().unwrap();
		frame.channel_layout = frame.data.channel_layout;
		frame.sample_format = SampleFormat::None;

		frame.picture_type = frame.data.pict_type.into();
		frame.sample_aspect_ratio = frame.data.sample_aspect_ratio.into();
		frame.width = frame.data.width.try_into().unwrap();
		frame.height = frame.data.height.try_into().unwrap();
		frame.repeat_picture = frame.data.repeat_pict;

		match self.context.codec_type {
			AVMediaType::AVMEDIA_TYPE_AUDIO => frame.sample_format = frame.data.format.into(),
			AVMediaType::AVMEDIA_TYPE_VIDEO => frame.pixel_format = frame.data.format.into(),
			_ => ()
		}

		Ok(true)
	}

	fn drain(&mut self) -> Result<()> {
		/* Safety: FFI call */
		unsafe { self.context.drain() }
	}

	fn flush(&mut self) -> Result<()> {
		/* Safety: FFI call */
		unsafe { self.context.flush() };

		Ok(())
	}
}
