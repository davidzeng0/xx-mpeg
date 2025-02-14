use xx_core::macros::{paste, wrapper_functions};

use super::codecs::*;
use super::*;

pub type MediaType = av::MediaType;
pub type Discard = av::Discard;

pub type PacketFlag = av::PacketFlag;
pub type FrameFlag = av::FrameFlag;

pub type SampleFormat = av::SampleFormat;
pub type Channel = av::Channel;
pub type ChannelLayout = av::ChannelLayout;

pub type PictureType = av::PictureType;
pub type PixelFormat = av::PixelFormat;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CodecParse {
	#[default]
	None,
	Header,
	Full
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CodecId {
	#[default]
	Unknown,
	Aac,
	Opus,
	Flac,
	Vorbis,
	Mp2,
	Mp3
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
	Decode,
	Encode
}

#[derive(Default)]
pub struct CodecParams {
	pub id: CodecId,
	pub ty: MediaType,

	pub config: Vec<u8>,

	pub time_base: Rational,
	pub packet_time_base: Rational,
	pub bit_rate: u32,
	pub bit_depth: u16,
	pub compression_level: u16,
	pub delay: u32,
	pub seek_preroll: u32,

	pub sample_rate: u32,
	pub ch_layout: ChannelLayout,
	pub frame_size: u32,
	pub sample_format: SampleFormat,

	pub width: u32,
	pub height: u32,
	pub sample_aspect_ratio: Rational,
	pub framerate: Rational,
	pub pixel_format: PixelFormat
}

impl CodecParams {
	#[allow(clippy::cast_possible_truncation)]
	pub fn change_time_base(&mut self, time_base: Rational) {
		self.delay = time_base.rescale(self.delay as u64, self.time_base) as u32;
		self.seek_preroll = time_base.rescale(self.seek_preroll as u64, self.time_base) as u32;
		self.time_base = time_base;
	}
}

pub trait CodecImpl {
	fn id(&self) -> CodecId;

	fn send_packet(&mut self, packet: &Packet) -> Result<()>;
	fn send_frame(&mut self, frame: &Frame) -> Result<()>;

	fn receive_packet(&mut self, packet: &mut Packet) -> Result<bool>;
	fn receive_frame(&mut self, frame: &mut Frame) -> Result<bool>;

	fn drain(&mut self) -> Result<()>;
	fn flush(&mut self) -> Result<()>;
}

pub struct Codec(Box<dyn CodecImpl + Send + Sync>);

impl Codec {
	wrapper_functions! {
		inner = self.0;

		pub fn id(&self) -> CodecId;

		pub fn send_packet(&mut self, packet: &Packet) -> Result<()>;
		pub fn send_frame(&mut self, frame: &Frame) -> Result<()>;

		pub fn drain(&mut self) -> Result<()>;
		pub fn flush(&mut self) -> Result<()>;
	}

	pub fn new(params: &mut CodecParams, mode: Mode) -> Result<Self> {
		macro_rules! pick_coder {
			($name:ident) => {
				paste! {
					match mode {
						Mode::Decode => {
							[<$name Decoder>]::new(params)?
						}

						Mode::Encode => {
							[<$name Encoder>]::new(params)?
						}
					}
				}
			};
		}

		let codec = match params.id {
			CodecId::Aac => pick_coder!(Aac),
			CodecId::Opus => pick_coder!(Opus),
			CodecId::Flac => pick_coder!(Flac),
			CodecId::Vorbis => pick_coder!(Vorbis),
			CodecId::Mp2 => pick_coder!(Mp2),
			CodecId::Mp3 => pick_coder!(Mp3),
			_ => return Err(FormatError::CodecNotFound.into())
		};

		Ok(Self(codec))
	}

	pub fn receive_packet(&mut self) -> Result<Option<Packet>> {
		let mut packet = Packet::new();

		Ok(self.0.receive_packet(&mut packet)?.then_some(packet))
	}

	pub fn receive_frame(&mut self) -> Result<Option<Frame>> {
		let mut frame = Frame::new();

		Ok(self.0.receive_frame(&mut frame)?.then_some(frame))
	}
}

pub trait CodecParserImpl {
	fn id(&self) -> CodecId;

	fn parse(&mut self, packet: &mut Packet) -> Result<()>;
}

pub struct CodecParser(Box<dyn CodecParserImpl + Send + Sync>);

impl CodecParser {
	wrapper_functions! {
		inner = self.0;

		pub fn id(&self) -> CodecId;
		pub fn parse(&mut self, packet: &mut Packet) -> Result<()>;
	}

	pub fn new(parse: CodecParse, params: &mut CodecParams) -> Result<Self> {
		let codec = match params.id {
			CodecId::Aac => AacParser::new(parse, params)?,
			CodecId::Opus => OpusParser::new(parse, params)?,
			CodecId::Flac => FlacParser::new(parse, params)?,
			CodecId::Vorbis => VorbisParser::new(parse, params)?,
			CodecId::Mp2 => Mp2Parser::new(parse, params)?,
			CodecId::Mp3 => Mp3Parser::new(parse, params)?,
			_ => return Err(FormatError::CodecNotFound.into())
		};

		Ok(Self(codec))
	}
}
