use std::ops::{Deref, DerefMut};

use enumflags2::*;
use ffmpeg_sys_next::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use xx_core::error::*;

use super::{codecs::*, *};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
#[repr(i32)]
pub enum SampleFormat {
	#[default]
	None = AVSampleFormat::AV_SAMPLE_FMT_NONE as i32,

	/* packed types */
	U8   = AVSampleFormat::AV_SAMPLE_FMT_U8 as i32,
	I16  = AVSampleFormat::AV_SAMPLE_FMT_S16 as i32,
	I32  = AVSampleFormat::AV_SAMPLE_FMT_S32 as i32,
	F32  = AVSampleFormat::AV_SAMPLE_FMT_FLT as i32,
	F64  = AVSampleFormat::AV_SAMPLE_FMT_DBL as i32,

	/* planar types */
	U8P  = AVSampleFormat::AV_SAMPLE_FMT_U8P as i32,
	I16P = AVSampleFormat::AV_SAMPLE_FMT_S16P as i32,
	I32P = AVSampleFormat::AV_SAMPLE_FMT_S32P as i32,
	F32P = AVSampleFormat::AV_SAMPLE_FMT_FLTP as i32,
	F64P = AVSampleFormat::AV_SAMPLE_FMT_DBLP as i32,

	I64  = AVSampleFormat::AV_SAMPLE_FMT_S64 as i32,
	I64P = AVSampleFormat::AV_SAMPLE_FMT_S64P as i32
}

impl SampleFormat {
	pub fn from(format: i32) -> Self {
		Self::from_i32(format).unwrap_or(Self::None)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, FromPrimitive)]
#[repr(u64)]
pub enum Channel {
	Unknown             = 0,
	FrontLeft           = AV_CH_FRONT_LEFT,
	FrontRight          = AV_CH_FRONT_RIGHT,
	FrontCenter         = AV_CH_FRONT_CENTER,
	LowFrequency        = AV_CH_LOW_FREQUENCY,
	BackLeft            = AV_CH_BACK_LEFT,
	BackRight           = AV_CH_BACK_RIGHT,
	FrontLeftOfCenter   = AV_CH_FRONT_LEFT_OF_CENTER,
	FrontRightOfCenter  = AV_CH_FRONT_RIGHT_OF_CENTER,
	BackCenter          = AV_CH_BACK_CENTER,
	SideLeft            = AV_CH_SIDE_LEFT,
	SideRight           = AV_CH_SIDE_RIGHT,
	TopCenter           = AV_CH_TOP_CENTER,
	TopFrontLeft        = AV_CH_TOP_FRONT_LEFT,
	TopFrontCenter      = AV_CH_TOP_FRONT_CENTER,
	TopFrontRight       = AV_CH_TOP_FRONT_RIGHT,
	TopBackLeft         = AV_CH_TOP_BACK_LEFT,
	TopBackCenter       = AV_CH_TOP_BACK_CENTER,
	TopBackRight        = AV_CH_TOP_BACK_RIGHT,
	StereoLeft          = AV_CH_STEREO_LEFT,
	StereoRight         = AV_CH_STEREO_RIGHT,
	WideLeft            = AV_CH_WIDE_LEFT,
	WideRight           = AV_CH_WIDE_RIGHT,
	SurroundDirectLeft  = AV_CH_SURROUND_DIRECT_LEFT,
	SurroundDirectRight = AV_CH_SURROUND_DIRECT_RIGHT,
	LowFrequency2       = AV_CH_LOW_FREQUENCY_2,
	TopSideLeft         = AV_CH_TOP_SIDE_LEFT,
	TopSideRight        = AV_CH_TOP_SIDE_RIGHT,
	BottomFrontCenter   = AV_CH_BOTTOM_FRONT_CENTER,
	BottomFrontLeft     = AV_CH_BOTTOM_FRONT_LEFT,
	BottomFrontRight    = AV_CH_BOTTOM_FRONT_RIGHT
}

impl Channel {
	pub fn from(channel: u64) -> Self {
		Self::from_u64(channel).unwrap_or(Self::Unknown)
	}
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CodecParse {
	#[default]
	None,
	Header,
	Full
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CodecId {
	#[default]
	Unknown,
	Aac,
	Opus,
	Flac,
	Vorbis,
	Mp3
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
	Decode,
	Encode
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[bitflags]
pub enum PacketFlag {
	Keyframe   = AV_PKT_FLAG_KEY as u32,
	Corrupt    = AV_PKT_FLAG_CORRUPT as u32,
	Discard    = AV_PKT_FLAG_DISCARD as u32,
	Trusted    = AV_PKT_FLAG_TRUSTED as u32,
	Disposable = AV_PKT_FLAG_DISPOSABLE as u32
}

#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[bitflags]
pub enum FrameFlag {
	Corrupt = AV_FRAME_FLAG_CORRUPT as u32,
	Discard = AV_FRAME_FLAG_DISCARD as u32
}

#[derive(Default)]
pub struct CodecParams {
	pub id: CodecId,
	pub config: Vec<u8>,

	pub ty: MediaType,
	pub time_base: Rational,
	pub packet_time_base: Rational,
	pub bit_rate: u32,
	pub bit_depth: u16,
	pub compression_level: u16,
	pub delay: u32,
	pub seek_preroll: u32,

	pub sample_rate: u32,
	pub channels: u16,
	pub channel_layout: u64,
	pub frame_size: u32,
	pub sample_format: SampleFormat
}

impl CodecParams {
	pub fn change_time_base(&mut self, time_base: Rational) {
		self.delay = time_base.rescale(self.delay as u64, self.time_base) as u32;
		self.seek_preroll = time_base.rescale(self.seek_preroll as u64, self.time_base) as u32;
		self.time_base = time_base;
	}
}

pub trait CodecTrait {
	fn id(&self) -> CodecId;
	fn packet_padding(&self) -> usize;

	fn send_packet(&mut self, packet: &Packet) -> Result<()>;
	fn send_frame(&mut self, frame: &Frame) -> Result<()>;

	fn receive_packet(&mut self) -> Result<Option<Packet>>;
	fn receive_frame(&mut self) -> Result<Option<Frame>>;

	fn drain(&mut self) -> Result<()>;
	fn flush(&mut self) -> Result<()>;
}

pub struct Codec {
	inner: Box<dyn CodecTrait>
}

impl Deref for Codec {
	type Target = Box<dyn CodecTrait>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for Codec {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}

impl Codec {
	pub fn new(params: &mut CodecParams, mode: Mode) -> Result<Self> {
		macro_rules! pick_coder {
			($name: ident) => {
				paste::paste! {
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
			CodecId::Mp3 => pick_coder!(Mp3),
			_ => return Err(Error::new(ErrorKind::Unsupported, "Codec unsupported"))
		};

		Ok(Self { inner: codec })
	}
}

pub trait CodecParserTrait {
	fn id(&self) -> CodecId;

	fn parse(&self, packet: &mut Packet) -> Result<()>;
}

pub struct CodecParser {
	inner: Box<dyn CodecParserTrait>
}

impl CodecParser {
	pub fn new(params: &mut CodecParams) -> Result<Self> {
		let codec = match params.id {
			CodecId::Aac => AacParser::new(params)?,
			CodecId::Opus => OpusParser::new(params)?,
			CodecId::Flac => FlacParser::new(params)?,
			CodecId::Vorbis => VorbisParser::new(params)?,
			CodecId::Mp3 => Mp3Parser::new(params)?,
			_ => return Err(Error::new(ErrorKind::Unsupported, "Codec unsupported"))
		};

		Ok(Self { inner: codec })
	}
}

impl Deref for CodecParser {
	type Target = Box<dyn CodecParserTrait>;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}

impl DerefMut for CodecParser {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.inner
	}
}
