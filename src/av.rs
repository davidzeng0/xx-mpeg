#![allow(unreachable_pub)]

use std::{
	mem::transmute,
	ops::{Deref, DerefMut}
};

pub use ffmpeg_sys_next::AVCodecID;
use ffmpeg_sys_next::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use xx_core::{error::*, os::error::OsError, paste::paste, pointer::*};

use super::*;

pub const UNKNOWN_TIMESTAMP: i64 = AV_NOPTS_VALUE;
pub const INPUT_BUFFER_PADDING: usize = AV_INPUT_BUFFER_PADDING_SIZE as usize;

#[errors]
pub enum AVError {
	#[error("Bitstream filter not found")]
	BitstreamFilterNotFound,

	#[error("AV internal bug")]
	InternalBug,

	#[error("Buffer too small")]
	BufferTooSmall,

	#[error("Demuxer not found")]
	DemuxerNotFound,

	#[error("End of file")]
	EndOfFile,

	#[error("Exit requested")]
	ExitRequested,

	#[error("Error in external library")]
	ExternalError,

	#[error("Filter not found")]
	FilterNotFound,

	#[error("Invalid data found while processing input")]
	InvalidData,

	#[error("Muxer not found")]
	MuxerNotFound,

	#[error("Option not found")]
	OptionNotFound,

	#[error("Not implemented")]
	NotImplemented,

	#[error("Protocol not found")]
	ProtocolNotFound,

	#[error("Stream not found")]
	StreamNotFound,

	#[error("Unknown error or error in external library")]
	Unknown,

	#[error("Feature is experimental")]
	Experimental,

	#[error("Input changed")]
	InputChanged,

	#[error("Output changed")]
	OutputChanged,

	#[error("HTTP bad request")]
	HttpBadRequest,

	#[error("HTTP unauthorized")]
	HttpUnauthorized,

	#[error("HTTP forbidden")]
	HttpForbidden,

	#[error("HTTP not found")]
	HttpNotFound,

	#[error("HTTP 4xx")]
	HttpOther4xx,

	#[error("HTTP server error")]
	HttpServerError
}

fn result_from_av(code: i32) -> Result<i32> {
	if code >= 0 {
		return Ok(code);
	}

	Err(match code {
		AVERROR_BSF_NOT_FOUND => AVError::BitstreamFilterNotFound.into(),
		AVERROR_BUG | AVERROR_BUG2 => AVError::InternalBug.into(),
		AVERROR_BUFFER_TOO_SMALL => AVError::BufferTooSmall.into(),
		AVERROR_DECODER_NOT_FOUND => FormatError::CodecNotFound.into(),
		AVERROR_DEMUXER_NOT_FOUND => AVError::DemuxerNotFound.into(),
		AVERROR_ENCODER_NOT_FOUND => FormatError::CodecNotFound.into(),
		AVERROR_EOF => AVError::EndOfFile.into(),
		AVERROR_EXIT => AVError::ExitRequested.into(),
		AVERROR_EXTERNAL => AVError::ExternalError.into(),
		AVERROR_FILTER_NOT_FOUND => AVError::FilterNotFound.into(),
		AVERROR_INVALIDDATA => AVError::InvalidData.into(),
		AVERROR_MUXER_NOT_FOUND => AVError::MuxerNotFound.into(),
		AVERROR_OPTION_NOT_FOUND => AVError::OptionNotFound.into(),
		AVERROR_PATCHWELCOME => AVError::NotImplemented.into(),
		AVERROR_PROTOCOL_NOT_FOUND => AVError::ProtocolNotFound.into(),
		AVERROR_STREAM_NOT_FOUND => AVError::StreamNotFound.into(),
		AVERROR_UNKNOWN => AVError::Unknown.into(),
		AVERROR_EXPERIMENTAL => AVError::Experimental.into(),
		AVERROR_INPUT_CHANGED => AVError::InputChanged.into(),
		AVERROR_OUTPUT_CHANGED => AVError::OutputChanged.into(),
		AVERROR_HTTP_BAD_REQUEST => AVError::HttpBadRequest.into(),
		AVERROR_HTTP_UNAUTHORIZED => AVError::HttpUnauthorized.into(),
		AVERROR_HTTP_FORBIDDEN => AVError::HttpForbidden.into(),
		AVERROR_HTTP_NOT_FOUND => AVError::HttpNotFound.into(),
		AVERROR_HTTP_OTHER_4XX => AVError::HttpOther4xx.into(),
		AVERROR_HTTP_SERVER_ERROR => AVError::HttpServerError.into(),
		code => OsError::from_raw(AVUNERROR(code)).into()
	})
}

macro_rules! ptr_deref {
	($struct:ident, $av:path, $free:ident) => {
		impl Drop for $struct {
			fn drop(&mut self) {
				let mut ptr = self.0.as_mut_ptr();

				/* Safety: we own this pointer */
				unsafe { $free(&mut ptr) }
			}
		}

		/// For internal use only. Changing random fields is unsafe
		impl Deref for $struct {
			type Target = $av;

			fn deref(&self) -> &Self::Target {
				/* Safety: the pointer is always valid */
				unsafe { self.0.as_ref() }
			}
		}

		/// For internal use only. Changing random fields is unsafe
		impl DerefMut for $struct {
			fn deref_mut(&mut self) -> &mut Self::Target {
				/* Safety: the pointer is always valid */
				unsafe { self.0.as_mut() }
			}
		}
	};
}

pub struct CodecContext(MutPtr<AVCodecContext>);

impl CodecContext {
	pub fn new(codec: Ptr<AVCodec>) -> Self {
		/* Safety: FFI call */
		let this = Self(unsafe { avcodec_alloc_context3(codec.as_ptr()) }.into());

		assert!(!this.0.is_null(), "Could not allocate memory for codec");

		this
	}
}

impl CodecContext {
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

	pub fn result_from_av_maybe_none(err: i32) -> Result<bool> {
		const AGAIN: i32 = AVERROR(OsError::Again as i32);

		match err {
			AVERROR_EOF | AGAIN => Ok(false),
			err => {
				result_from_av(err)?;

				Ok(true)
			}
		}
	}

	pub unsafe fn receive_packet(&mut self, packet: &mut AVPacket) -> Result<bool> {
		/* Safety: FFI call */
		let ret = unsafe { avcodec_receive_packet(self.0.as_mut_ptr(), &mut **packet) };
		let received = Self::result_from_av_maybe_none(ret)?;

		Ok(received)
	}

	pub unsafe fn receive_frame(&mut self, frame: &mut AVFrame) -> Result<bool> {
		/* Safety: FFI call */
		let ret = unsafe { avcodec_receive_frame(self.0.as_mut_ptr(), &mut **frame) };
		let received = Self::result_from_av_maybe_none(ret)?;

		Ok(received)
	}

	pub unsafe fn drain(&mut self) -> Result<()> {
		/* Safety: FFI call */
		let ret =
			unsafe { avcodec_receive_packet(self.0.as_mut_ptr(), MutPtr::null().as_mut_ptr()) };

		result_from_av(ret)?;

		Ok(())
	}

	pub unsafe fn flush(&mut self) {
		/* Safety: FFI call */
		unsafe { avcodec_flush_buffers(self.0.as_mut_ptr()) };
	}
}

ptr_deref!(CodecContext, AVCodecContext, avcodec_free_context);

pub struct AVPacket(MutPtr<ffmpeg_sys_next::AVPacket>);

impl AVPacket {
	pub fn new() -> Self {
		/* Safety: FFI call */
		let this = Self(unsafe { av_packet_alloc() }.into());

		assert!(!this.0.is_null(), "Failed to allocate packet");

		this
	}

	pub fn unref(&mut self) {
		/* Safety: FFI call */
		unsafe { av_packet_unref(self.0.as_mut_ptr()) }
	}

	pub fn data(&self) -> Ptr<[u8]> {
		#[allow(clippy::unwrap_used)]
		Ptr::slice_from_raw_parts(self.data.cast_const().into(), self.size.try_into().unwrap())
	}
}

ptr_deref!(AVPacket, ffmpeg_sys_next::AVPacket, av_packet_free);

pub struct AVFrame(MutPtr<ffmpeg_sys_next::AVFrame>);

impl AVFrame {
	pub fn new() -> Self {
		/* Safety: FFI call */
		let this = Self(unsafe { av_frame_alloc() }.into());

		assert!(!this.0.is_null(), "Failed to allocate frame");

		this
	}

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

ptr_deref!(AVFrame, ffmpeg_sys_next::AVFrame, av_frame_free);

impl From<Rational> for AVRational {
	fn from(value: Rational) -> Self {
		let Rational { num, den } = value;

		#[allow(clippy::unwrap_used)]
		Self {
			num: num.try_into().unwrap(),
			den: den.try_into().unwrap()
		}
	}
}

impl From<AVRational> for Rational {
	fn from(value: AVRational) -> Self {
		let AVRational { num, den } = value;

		#[allow(clippy::unwrap_used)]
		Self {
			num: num.try_into().unwrap(),
			den: den.try_into().unwrap()
		}
	}
}

macro_rules! define_av_alias {
	(
		#[repr($repr:ty)]
		$(#$attrs: tt)*
		$vis: vis
		enum $name:ident
		$($rest: tt)*
	) => {
		#[repr($repr)]
		#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
		$(#$attrs)*
		$vis enum $name $($rest)*
	};
}

macro_rules! define_av_alias_casts {
	(
		#[repr($repr:ty)]
		$(#$attrs: tt)*
		$vis: vis
		enum $name:ident = $av:ident
		$($rest: tt)*
	) => {
		define_av_alias! {
			#[repr($repr)]
			#[derive(Default, FromPrimitive)]
			$(#$attrs)*
			$vis enum $name $($rest)*
		}

		impl From<$repr> for $name {
			fn from(format: $repr) -> Self {
				paste! {
					Self::[< from_ $repr >](format).unwrap_or_default()
				}
			}
		}

		impl From<$name> for $av {
			fn from(value: $name) -> Self {
				/* Safety: same repr */
				unsafe { transmute(value) }
			}
		}

		impl From<$av> for $name {
			fn from(value: $av) -> Self {
				/* shared lib values may be non-exhaustive */
				Self::from(value as $repr)
			}
		}
	};
}

define_av_alias! {
	#[repr(u32)]
	#[bitflags]
	pub enum PacketFlag {
		Keyframe   = AV_PKT_FLAG_KEY as u32,
		Corrupt    = AV_PKT_FLAG_CORRUPT as u32,
		Discard    = AV_PKT_FLAG_DISCARD as u32,
		Trusted    = AV_PKT_FLAG_TRUSTED as u32,
		Disposable = AV_PKT_FLAG_DISPOSABLE as u32
	}
}

define_av_alias! {
	#[repr(u32)]
	#[bitflags]
	pub enum FrameFlag {
		Corrupt    = AV_FRAME_FLAG_CORRUPT as u32,
		Key        = AV_FRAME_FLAG_KEY as u32,
		Discard    = AV_FRAME_FLAG_DISCARD as u32,
		Interlaced = AV_FRAME_FLAG_INTERLACED as u32
	}
}

define_av_alias_casts! {
	#[repr(i32)]
	pub enum SampleFormat = AVSampleFormat {
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
}

define_av_alias! {
	#[repr(u64)]
	#[derive(Default, FromPrimitive)]
	pub enum Channel {
		#[default]
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
}

impl From<u64> for Channel {
	fn from(channel: u64) -> Self {
		Self::from_u64(channel).unwrap_or(Self::Unknown)
	}
}

define_av_alias_casts! {
	#[repr(u32)]
	pub enum PictureType = AVPictureType {
		#[default]
		None               = AVPictureType::AV_PICTURE_TYPE_NONE as u32,
		Intra              = AVPictureType::AV_PICTURE_TYPE_I as u32,
		Predicted          = AVPictureType::AV_PICTURE_TYPE_P as u32,
		BidirPredicted     = AVPictureType::AV_PICTURE_TYPE_B as u32,
		Switching          = AVPictureType::AV_PICTURE_TYPE_S as u32,
		SwitchingIntra     = AVPictureType::AV_PICTURE_TYPE_SI as u32,
		SwitchingPredicted = AVPictureType::AV_PICTURE_TYPE_SP as u32,
		BI                 = AVPictureType::AV_PICTURE_TYPE_BI as u32
	}
}

define_av_alias_casts! {
	#[repr(i32)]
	pub enum MediaType = AVMediaType {
		#[default]
		Unknown    = AVMediaType::AVMEDIA_TYPE_UNKNOWN as i32,
		Video      = AVMediaType::AVMEDIA_TYPE_VIDEO as i32,
		Audio      = AVMediaType::AVMEDIA_TYPE_AUDIO as i32,
		Data       = AVMediaType::AVMEDIA_TYPE_DATA as i32,
		Subtitle   = AVMediaType::AVMEDIA_TYPE_SUBTITLE as i32,
		Attachment = AVMediaType::AVMEDIA_TYPE_ATTACHMENT as i32
	}
}

define_av_alias_casts! {
	#[repr(i32)]
	#[allow(clippy::upper_case_acronyms)]
	pub enum PixelFormat = AVPixelFormat {
		#[default]
		None          = AVPixelFormat::AV_PIX_FMT_NONE as i32,
		YUV420P       = AVPixelFormat::AV_PIX_FMT_YUV420P as i32,
		YUYV422       = AVPixelFormat::AV_PIX_FMT_YUYV422 as i32,
		RGB24         = AVPixelFormat::AV_PIX_FMT_RGB24 as i32,
		BGR24         = AVPixelFormat::AV_PIX_FMT_BGR24 as i32,
		YUV422P       = AVPixelFormat::AV_PIX_FMT_YUV422P as i32,
		YUV444P       = AVPixelFormat::AV_PIX_FMT_YUV444P as i32,
		YUV410P       = AVPixelFormat::AV_PIX_FMT_YUV410P as i32,
		YUV411P       = AVPixelFormat::AV_PIX_FMT_YUV411P as i32,
		GRAY8         = AVPixelFormat::AV_PIX_FMT_GRAY8 as i32,
		MONOWHITE     = AVPixelFormat::AV_PIX_FMT_MONOWHITE as i32,
		MONOBLACK     = AVPixelFormat::AV_PIX_FMT_MONOBLACK as i32,
		PAL8          = AVPixelFormat::AV_PIX_FMT_PAL8 as i32,
		YUVJ420P      = AVPixelFormat::AV_PIX_FMT_YUVJ420P as i32,
		YUVJ422P      = AVPixelFormat::AV_PIX_FMT_YUVJ422P as i32,
		YUVJ444P      = AVPixelFormat::AV_PIX_FMT_YUVJ444P as i32,
		UYVY422       = AVPixelFormat::AV_PIX_FMT_UYVY422 as i32,
		UYYVYY411     = AVPixelFormat::AV_PIX_FMT_UYYVYY411 as i32,
		BGR8          = AVPixelFormat::AV_PIX_FMT_BGR8 as i32,
		BGR4          = AVPixelFormat::AV_PIX_FMT_BGR4 as i32,
		BGR4Byte      = AVPixelFormat::AV_PIX_FMT_BGR4_BYTE as i32,
		RGB8          = AVPixelFormat::AV_PIX_FMT_RGB8 as i32,
		RGB4          = AVPixelFormat::AV_PIX_FMT_RGB4 as i32,
		RGB4Byte      = AVPixelFormat::AV_PIX_FMT_RGB4_BYTE as i32,
		NV12          = AVPixelFormat::AV_PIX_FMT_NV12 as i32,
		NV21          = AVPixelFormat::AV_PIX_FMT_NV21 as i32,
		ARGB          = AVPixelFormat::AV_PIX_FMT_ARGB as i32,
		RGBA          = AVPixelFormat::AV_PIX_FMT_RGBA as i32,
		ABGR          = AVPixelFormat::AV_PIX_FMT_ABGR as i32,
		BGRA          = AVPixelFormat::AV_PIX_FMT_BGRA as i32,
		GRAY16BE      = AVPixelFormat::AV_PIX_FMT_GRAY16BE as i32,
		GRAY16LE      = AVPixelFormat::AV_PIX_FMT_GRAY16LE as i32,
		YUV440P       = AVPixelFormat::AV_PIX_FMT_YUV440P as i32,
		YUVJ440P      = AVPixelFormat::AV_PIX_FMT_YUVJ440P as i32,
		YUVA420P      = AVPixelFormat::AV_PIX_FMT_YUVA420P as i32,
		RGB48BE       = AVPixelFormat::AV_PIX_FMT_RGB48BE as i32,
		RGB48LE       = AVPixelFormat::AV_PIX_FMT_RGB48LE as i32,
		RGB565BE      = AVPixelFormat::AV_PIX_FMT_RGB565BE as i32,
		RGB565LE      = AVPixelFormat::AV_PIX_FMT_RGB565LE as i32,
		RGB555BE      = AVPixelFormat::AV_PIX_FMT_RGB555BE as i32,
		RGB555LE      = AVPixelFormat::AV_PIX_FMT_RGB555LE as i32,
		BGR565BE      = AVPixelFormat::AV_PIX_FMT_BGR565BE as i32,
		BGR565LE      = AVPixelFormat::AV_PIX_FMT_BGR565LE as i32,
		BGR555BE      = AVPixelFormat::AV_PIX_FMT_BGR555BE as i32,
		BGR555LE      = AVPixelFormat::AV_PIX_FMT_BGR555LE as i32,
		VAAPI         = AVPixelFormat::AV_PIX_FMT_VAAPI as i32,
		YUV420P16LE   = AVPixelFormat::AV_PIX_FMT_YUV420P16LE as i32,
		YUV420P16BE   = AVPixelFormat::AV_PIX_FMT_YUV420P16BE as i32,
		YUV422P16LE   = AVPixelFormat::AV_PIX_FMT_YUV422P16LE as i32,
		YUV422P16BE   = AVPixelFormat::AV_PIX_FMT_YUV422P16BE as i32,
		YUV444P16LE   = AVPixelFormat::AV_PIX_FMT_YUV444P16LE as i32,
		YUV444P16BE   = AVPixelFormat::AV_PIX_FMT_YUV444P16BE as i32,
		DXVA2Vld      = AVPixelFormat::AV_PIX_FMT_DXVA2_VLD as i32,
		RGB444LE      = AVPixelFormat::AV_PIX_FMT_RGB444LE as i32,
		RGB444BE      = AVPixelFormat::AV_PIX_FMT_RGB444BE as i32,
		BGR444LE      = AVPixelFormat::AV_PIX_FMT_BGR444LE as i32,
		BGR444BE      = AVPixelFormat::AV_PIX_FMT_BGR444BE as i32,
		YA8           = AVPixelFormat::AV_PIX_FMT_YA8 as i32,
		BGR48BE       = AVPixelFormat::AV_PIX_FMT_BGR48BE as i32,
		BGR48LE       = AVPixelFormat::AV_PIX_FMT_BGR48LE as i32,
		YUV420P9BE    = AVPixelFormat::AV_PIX_FMT_YUV420P9BE as i32,
		YUV420P9LE    = AVPixelFormat::AV_PIX_FMT_YUV420P9LE as i32,
		YUV420P10BE   = AVPixelFormat::AV_PIX_FMT_YUV420P10BE as i32,
		YUV420P10LE   = AVPixelFormat::AV_PIX_FMT_YUV420P10LE as i32,
		YUV422P10BE   = AVPixelFormat::AV_PIX_FMT_YUV422P10BE as i32,
		YUV422P10LE   = AVPixelFormat::AV_PIX_FMT_YUV422P10LE as i32,
		YUV444P9BE    = AVPixelFormat::AV_PIX_FMT_YUV444P9BE as i32,
		YUV444P9LE    = AVPixelFormat::AV_PIX_FMT_YUV444P9LE as i32,
		YUV444P10BE   = AVPixelFormat::AV_PIX_FMT_YUV444P10BE as i32,
		YUV444P10LE   = AVPixelFormat::AV_PIX_FMT_YUV444P10LE as i32,
		YUV422P9BE    = AVPixelFormat::AV_PIX_FMT_YUV422P9BE as i32,
		YUV422P9LE    = AVPixelFormat::AV_PIX_FMT_YUV422P9LE as i32,
		GBRP          = AVPixelFormat::AV_PIX_FMT_GBRP as i32,
		GBRP9BE       = AVPixelFormat::AV_PIX_FMT_GBRP9BE as i32,
		GBRP9LE       = AVPixelFormat::AV_PIX_FMT_GBRP9LE as i32,
		GBRP10BE      = AVPixelFormat::AV_PIX_FMT_GBRP10BE as i32,
		GBRP10LE      = AVPixelFormat::AV_PIX_FMT_GBRP10LE as i32,
		GBRP16BE      = AVPixelFormat::AV_PIX_FMT_GBRP16BE as i32,
		GBRP16LE      = AVPixelFormat::AV_PIX_FMT_GBRP16LE as i32,
		YUVA422P      = AVPixelFormat::AV_PIX_FMT_YUVA422P as i32,
		YUVA444P      = AVPixelFormat::AV_PIX_FMT_YUVA444P as i32,
		YUVA420P9BE   = AVPixelFormat::AV_PIX_FMT_YUVA420P9BE as i32,
		YUVA420P9LE   = AVPixelFormat::AV_PIX_FMT_YUVA420P9LE as i32,
		YUVA422P9BE   = AVPixelFormat::AV_PIX_FMT_YUVA422P9BE as i32,
		YUVA422P9LE   = AVPixelFormat::AV_PIX_FMT_YUVA422P9LE as i32,
		YUVA444P9BE   = AVPixelFormat::AV_PIX_FMT_YUVA444P9BE as i32,
		YUVA444P9LE   = AVPixelFormat::AV_PIX_FMT_YUVA444P9LE as i32,
		YUVA420P10BE  = AVPixelFormat::AV_PIX_FMT_YUVA420P10BE as i32,
		YUVA420P10LE  = AVPixelFormat::AV_PIX_FMT_YUVA420P10LE as i32,
		YUVA422P10BE  = AVPixelFormat::AV_PIX_FMT_YUVA422P10BE as i32,
		YUVA422P10LE  = AVPixelFormat::AV_PIX_FMT_YUVA422P10LE as i32,
		YUVA444P10BE  = AVPixelFormat::AV_PIX_FMT_YUVA444P10BE as i32,
		YUVA444P10LE  = AVPixelFormat::AV_PIX_FMT_YUVA444P10LE as i32,
		YUVA420P16BE  = AVPixelFormat::AV_PIX_FMT_YUVA420P16BE as i32,
		YUVA420P16LE  = AVPixelFormat::AV_PIX_FMT_YUVA420P16LE as i32,
		YUVA422P16BE  = AVPixelFormat::AV_PIX_FMT_YUVA422P16BE as i32,
		YUVA422P16LE  = AVPixelFormat::AV_PIX_FMT_YUVA422P16LE as i32,
		YUVA444P16BE  = AVPixelFormat::AV_PIX_FMT_YUVA444P16BE as i32,
		YUVA444P16LE  = AVPixelFormat::AV_PIX_FMT_YUVA444P16LE as i32,
		VDPAU         = AVPixelFormat::AV_PIX_FMT_VDPAU as i32,
		XYZ12LE       = AVPixelFormat::AV_PIX_FMT_XYZ12LE as i32,
		XYZ12BE       = AVPixelFormat::AV_PIX_FMT_XYZ12BE as i32,
		NV16          = AVPixelFormat::AV_PIX_FMT_NV16 as i32,
		NV20LE        = AVPixelFormat::AV_PIX_FMT_NV20LE as i32,
		NV20BE        = AVPixelFormat::AV_PIX_FMT_NV20BE as i32,
		RGBA64BE      = AVPixelFormat::AV_PIX_FMT_RGBA64BE as i32,
		RGBA64LE      = AVPixelFormat::AV_PIX_FMT_RGBA64LE as i32,
		BGRA64BE      = AVPixelFormat::AV_PIX_FMT_BGRA64BE as i32,
		BGRA64LE      = AVPixelFormat::AV_PIX_FMT_BGRA64LE as i32,
		YVYU422       = AVPixelFormat::AV_PIX_FMT_YVYU422 as i32,
		YA16BE        = AVPixelFormat::AV_PIX_FMT_YA16BE as i32,
		YA16LE        = AVPixelFormat::AV_PIX_FMT_YA16LE as i32,
		GBRAP         = AVPixelFormat::AV_PIX_FMT_GBRAP as i32,
		GBRAP16BE     = AVPixelFormat::AV_PIX_FMT_GBRAP16BE as i32,
		GBRAP16LE     = AVPixelFormat::AV_PIX_FMT_GBRAP16LE as i32,
		QSV           = AVPixelFormat::AV_PIX_FMT_QSV as i32,
		MMAL          = AVPixelFormat::AV_PIX_FMT_MMAL as i32,
		D3D11VAVld    = AVPixelFormat::AV_PIX_FMT_D3D11VA_VLD as i32,
		CUDA          = AVPixelFormat::AV_PIX_FMT_CUDA as i32,
		ZRGB          = AVPixelFormat::AV_PIX_FMT_0RGB as i32,
		RGBZ          = AVPixelFormat::AV_PIX_FMT_RGB0 as i32,
		ZBGR          = AVPixelFormat::AV_PIX_FMT_0BGR as i32,
		BGR0          = AVPixelFormat::AV_PIX_FMT_BGR0 as i32,
		YUV420P12BE   = AVPixelFormat::AV_PIX_FMT_YUV420P12BE as i32,
		YUV420P12LE   = AVPixelFormat::AV_PIX_FMT_YUV420P12LE as i32,
		YUV420P14BE   = AVPixelFormat::AV_PIX_FMT_YUV420P14BE as i32,
		YUV420P14LE   = AVPixelFormat::AV_PIX_FMT_YUV420P14LE as i32,
		YUV422P12BE   = AVPixelFormat::AV_PIX_FMT_YUV422P12BE as i32,
		YUV422P12LE   = AVPixelFormat::AV_PIX_FMT_YUV422P12LE as i32,
		YUV422P14BE   = AVPixelFormat::AV_PIX_FMT_YUV422P14BE as i32,
		YUV422P14LE   = AVPixelFormat::AV_PIX_FMT_YUV422P14LE as i32,
		YUV444P12BE   = AVPixelFormat::AV_PIX_FMT_YUV444P12BE as i32,
		YUV444P12LE   = AVPixelFormat::AV_PIX_FMT_YUV444P12LE as i32,
		YUV444P14BE   = AVPixelFormat::AV_PIX_FMT_YUV444P14BE as i32,
		YUV444P14LE   = AVPixelFormat::AV_PIX_FMT_YUV444P14LE as i32,
		GBRP12BE      = AVPixelFormat::AV_PIX_FMT_GBRP12BE as i32,
		GBRP12LE      = AVPixelFormat::AV_PIX_FMT_GBRP12LE as i32,
		GBRP14BE      = AVPixelFormat::AV_PIX_FMT_GBRP14BE as i32,
		GBRP14LE      = AVPixelFormat::AV_PIX_FMT_GBRP14LE as i32,
		YUVJ411P      = AVPixelFormat::AV_PIX_FMT_YUVJ411P as i32,
		BayerBGGR8    = AVPixelFormat::AV_PIX_FMT_BAYER_BGGR8 as i32,
		BayerRGGB8    = AVPixelFormat::AV_PIX_FMT_BAYER_RGGB8 as i32,
		BayerGBRG8    = AVPixelFormat::AV_PIX_FMT_BAYER_GBRG8 as i32,
		BayerGRBG8    = AVPixelFormat::AV_PIX_FMT_BAYER_GRBG8 as i32,
		BayerBGGR16LE = AVPixelFormat::AV_PIX_FMT_BAYER_BGGR16LE as i32,
		BayerBGGR16BE = AVPixelFormat::AV_PIX_FMT_BAYER_BGGR16BE as i32,
		BayerRGGB16LE = AVPixelFormat::AV_PIX_FMT_BAYER_RGGB16LE as i32,
		BayerRGGB16BE = AVPixelFormat::AV_PIX_FMT_BAYER_RGGB16BE as i32,
		BayerGBRG16LE = AVPixelFormat::AV_PIX_FMT_BAYER_GBRG16LE as i32,
		BayerGBRG16BE = AVPixelFormat::AV_PIX_FMT_BAYER_GBRG16BE as i32,
		BayerGRBG16LE = AVPixelFormat::AV_PIX_FMT_BAYER_GRBG16LE as i32,
		BayerGRBG16BE = AVPixelFormat::AV_PIX_FMT_BAYER_GRBG16BE as i32,
		XVMC          = AVPixelFormat::AV_PIX_FMT_XVMC as i32,
		YUV440P10LE   = AVPixelFormat::AV_PIX_FMT_YUV440P10LE as i32,
		YUV440P10BE   = AVPixelFormat::AV_PIX_FMT_YUV440P10BE as i32,
		YUV440P12LE   = AVPixelFormat::AV_PIX_FMT_YUV440P12LE as i32,
		YUV440P12BE   = AVPixelFormat::AV_PIX_FMT_YUV440P12BE as i32,
		AYUV64LE      = AVPixelFormat::AV_PIX_FMT_AYUV64LE as i32,
		AYUV64BE      = AVPixelFormat::AV_PIX_FMT_AYUV64BE as i32,
		VIDEOTOOLBOX  = AVPixelFormat::AV_PIX_FMT_VIDEOTOOLBOX as i32,
		P010LE        = AVPixelFormat::AV_PIX_FMT_P010LE as i32,
		P010BE        = AVPixelFormat::AV_PIX_FMT_P010BE as i32,
		GBRAP12BE     = AVPixelFormat::AV_PIX_FMT_GBRAP12BE as i32,
		GBRAP12LE     = AVPixelFormat::AV_PIX_FMT_GBRAP12LE as i32,
		GBRAP10BE     = AVPixelFormat::AV_PIX_FMT_GBRAP10BE as i32,
		GBRAP10LE     = AVPixelFormat::AV_PIX_FMT_GBRAP10LE as i32,
		MEDIACODEC    = AVPixelFormat::AV_PIX_FMT_MEDIACODEC as i32,
		GRAY12BE      = AVPixelFormat::AV_PIX_FMT_GRAY12BE as i32,
		GRAY12LE      = AVPixelFormat::AV_PIX_FMT_GRAY12LE as i32,
		GRAY10BE      = AVPixelFormat::AV_PIX_FMT_GRAY10BE as i32,
		GRAY10LE      = AVPixelFormat::AV_PIX_FMT_GRAY10LE as i32,
		P016LE        = AVPixelFormat::AV_PIX_FMT_P016LE as i32,
		P016BE        = AVPixelFormat::AV_PIX_FMT_P016BE as i32,
		D3D11         = AVPixelFormat::AV_PIX_FMT_D3D11 as i32,
		GRAY9BE       = AVPixelFormat::AV_PIX_FMT_GRAY9BE as i32,
		GRAY9LE       = AVPixelFormat::AV_PIX_FMT_GRAY9LE as i32,
		GBRPF32BE     = AVPixelFormat::AV_PIX_FMT_GBRPF32BE as i32,
		GBRPF32LE     = AVPixelFormat::AV_PIX_FMT_GBRPF32LE as i32,
		GBRAPF32BE    = AVPixelFormat::AV_PIX_FMT_GBRAPF32BE as i32,
		GBRAPF32LE    = AVPixelFormat::AV_PIX_FMT_GBRAPF32LE as i32,
		DRMPRIME      = AVPixelFormat::AV_PIX_FMT_DRM_PRIME as i32,
		OPENCL        = AVPixelFormat::AV_PIX_FMT_OPENCL as i32,
		GRAY14BE      = AVPixelFormat::AV_PIX_FMT_GRAY14BE as i32,
		GRAY14LE      = AVPixelFormat::AV_PIX_FMT_GRAY14LE as i32,
		GRAYF32BE     = AVPixelFormat::AV_PIX_FMT_GRAYF32BE as i32,
		GRAYF32LE     = AVPixelFormat::AV_PIX_FMT_GRAYF32LE as i32,
		YUVA422P12BE  = AVPixelFormat::AV_PIX_FMT_YUVA422P12BE as i32,
		YUVA422P12LE  = AVPixelFormat::AV_PIX_FMT_YUVA422P12LE as i32,
		YUVA444P12BE  = AVPixelFormat::AV_PIX_FMT_YUVA444P12BE as i32,
		YUVA444P12LE  = AVPixelFormat::AV_PIX_FMT_YUVA444P12LE as i32,
		NV24          = AVPixelFormat::AV_PIX_FMT_NV24 as i32,
		NV42          = AVPixelFormat::AV_PIX_FMT_NV42 as i32,
		VULKAN        = AVPixelFormat::AV_PIX_FMT_VULKAN as i32,
		Y210BE        = AVPixelFormat::AV_PIX_FMT_Y210BE as i32,
		Y210LE        = AVPixelFormat::AV_PIX_FMT_Y210LE as i32,
		X2RGB10LE     = AVPixelFormat::AV_PIX_FMT_X2RGB10LE as i32,
		X2RGB10BE     = AVPixelFormat::AV_PIX_FMT_X2RGB10BE as i32,
		X2BGR10LE     = AVPixelFormat::AV_PIX_FMT_X2BGR10LE as i32,
		X2BGR10BE     = AVPixelFormat::AV_PIX_FMT_X2BGR10BE as i32,
		P210BE        = AVPixelFormat::AV_PIX_FMT_P210BE as i32,
		P210LE        = AVPixelFormat::AV_PIX_FMT_P210LE as i32,
		P410BE        = AVPixelFormat::AV_PIX_FMT_P410BE as i32,
		P410LE        = AVPixelFormat::AV_PIX_FMT_P410LE as i32,
		P216BE        = AVPixelFormat::AV_PIX_FMT_P216BE as i32,
		P216LE        = AVPixelFormat::AV_PIX_FMT_P216LE as i32,
		P416BE        = AVPixelFormat::AV_PIX_FMT_P416BE as i32,
		P416LE        = AVPixelFormat::AV_PIX_FMT_P416LE as i32,
		VUYA          = AVPixelFormat::AV_PIX_FMT_VUYA as i32,
		RGBAF16BE     = AVPixelFormat::AV_PIX_FMT_RGBAF16BE as i32,
		RGBAF16LE     = AVPixelFormat::AV_PIX_FMT_RGBAF16LE as i32,
		VUYX          = AVPixelFormat::AV_PIX_FMT_VUYX as i32,
		P012LE        = AVPixelFormat::AV_PIX_FMT_P012LE as i32,
		P012BE        = AVPixelFormat::AV_PIX_FMT_P012BE as i32,
		Y212BE        = AVPixelFormat::AV_PIX_FMT_Y212BE as i32,
		Y212LE        = AVPixelFormat::AV_PIX_FMT_Y212LE as i32,
		XV30BE        = AVPixelFormat::AV_PIX_FMT_XV30BE as i32,
		XV30LE        = AVPixelFormat::AV_PIX_FMT_XV30LE as i32,
		XV36BE        = AVPixelFormat::AV_PIX_FMT_XV36BE as i32,
		XV36LE        = AVPixelFormat::AV_PIX_FMT_XV36LE as i32,
		RGBF32BE      = AVPixelFormat::AV_PIX_FMT_RGBF32BE as i32,
		RGBF32LE      = AVPixelFormat::AV_PIX_FMT_RGBF32LE as i32,
		RGBAF32BE     = AVPixelFormat::AV_PIX_FMT_RGBAF32BE as i32,
		RGBAF32LE     = AVPixelFormat::AV_PIX_FMT_RGBAF32LE as i32,
		P212BE        = AVPixelFormat::AV_PIX_FMT_P212BE as i32,
		P212LE        = AVPixelFormat::AV_PIX_FMT_P212LE as i32,
		P412BE        = AVPixelFormat::AV_PIX_FMT_P412BE as i32,
		P412LE        = AVPixelFormat::AV_PIX_FMT_P412LE as i32,
		GBRAP14BE     = AVPixelFormat::AV_PIX_FMT_GBRAP14BE as i32,
		GBRAP14LE     = AVPixelFormat::AV_PIX_FMT_GBRAP14LE as i32
	}
}

define_av_alias_casts! {
	#[repr(i32)]
	pub enum Discard = AVDiscard {
		#[default]
		None     = AVDiscard::AVDISCARD_NONE as i32,
		Default  = AVDiscard::AVDISCARD_DEFAULT as i32,
		NonRef   = AVDiscard::AVDISCARD_NONREF as i32,
		Bidir    = AVDiscard::AVDISCARD_BIDIR as i32,
		NonIntra = AVDiscard::AVDISCARD_NONINTRA as i32,
		NonKey   = AVDiscard::AVDISCARD_NONKEY as i32,
		All      = AVDiscard::AVDISCARD_ALL as i32
	}
}