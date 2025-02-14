#![allow(unreachable_pub)]

use xx_core::error::*;
use xx_core::pointer::*;

use super::*;
pub use crate::av::AVCodecID::*;
use crate::av::*;

pub struct AVCodec {
	id: CodecId,
	context: CodecContext,
	packet: AVPacket,
	frame: AVFrame
}

#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
fn open_codec(
	codec: NonNull<ffmpeg_sys_next::AVCodec>, params: &mut CodecParams, mode: Mode
) -> Result<CodecContext> {
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
	context.ch_layout = (&params.ch_layout).into();
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
	params.ch_layout = context.ch_layout.into();
	params.frame_size = context.frame_size.try_into().unwrap();
	params.sample_format = context.sample_fmt.into();

	params.width = context.width.try_into().unwrap();
	params.height = context.height.try_into().unwrap();
	params.sample_aspect_ratio = context.sample_aspect_ratio.into();
	params.framerate = context.framerate.into();
	params.pixel_format = context.pix_fmt.into();

	Ok(context)
}

impl AVCodec {
	pub fn new(
		id: CodecId, codec: NonNull<ffmpeg_sys_next::AVCodec>, params: &mut CodecParams, mode: Mode
	) -> Result<Self> {
		Ok(Self {
			id,
			context: open_codec(codec, params, mode)?,
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

		frame.copy_fields_to(&mut self.frame);

		/* Safety: all data is valid */
		unsafe { self.context.send_frame(&self.frame)? };

		self.frame.unref();

		Ok(())
	}

	fn receive_packet(&mut self, packet: &mut Packet) -> Result<bool> {
		/* Safety: packet is valid */
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
		/* Safety: frame is valid */
		if !unsafe { self.context.receive_frame(&mut frame.data)? } {
			return Ok(false);
		}

		frame.get_fields_from_inner(Some(self.context.codec_type.into()));

		Ok(true)
	}

	fn drain(&mut self) -> Result<()> {
		self.context.drain()?;

		Ok(())
	}

	fn flush(&mut self) -> Result<()> {
		self.context.flush();

		Ok(())
	}
}

pub struct AVCodecParser {
	id: CodecId,
	codec: CodecContext,
	parser: ParserContext
}

impl AVCodecParser {
	pub fn new(
		id: CodecId, codec: NonNull<ffmpeg_sys_next::AVCodec>, mut parser: ParserContext,
		parse: CodecParse, params: &mut CodecParams
	) -> Result<Self> {
		let codec = open_codec(codec, params, Mode::Decode)?;

		if parse == CodecParse::Header {
			parser.flags |= PARSER_FLAG_COMPLETE_FRAMES;
		}

		Ok(Self { id, codec, parser })
	}
}

impl CodecParserImpl for AVCodecParser {
	fn id(&self) -> CodecId {
		self.id
	}

	fn parse(&mut self, packet: &mut Packet) -> Result<()> {
		let mut duration = packet.duration;

		if !self.parser.parse(
			&mut self.codec,
			&packet.data,
			packet.timestamp,
			packet.timestamp,
			-1,
			&mut duration
		) {
			return Ok(());
		}

		packet.duration = duration;

		Ok(())
	}
}
