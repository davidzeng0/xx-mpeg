use super::*;

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
		Corrupt       = AV_FRAME_FLAG_CORRUPT as u32,
		Key           = AV_FRAME_FLAG_KEY as u32,
		Discard       = AV_FRAME_FLAG_DISCARD as u32,
		Interlaced    = AV_FRAME_FLAG_INTERLACED as u32,
		TopFieldFirst = AV_FRAME_FLAG_TOP_FIELD_FIRST as u32,
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

define_av_alias_casts! {
	#[repr(i32)]
	pub enum Channel = AVChannel {
		#[default]
		None                = AVChannel::AV_CHAN_NONE as i32,
		FrontLeft           = AVChannel::AV_CHAN_FRONT_LEFT as i32,
		FrontRight          = AVChannel::AV_CHAN_FRONT_RIGHT as i32,
		FrontCenter         = AVChannel::AV_CHAN_FRONT_CENTER as i32,
		LowFrequency        = AVChannel::AV_CHAN_LOW_FREQUENCY as i32,
		BackLeft            = AVChannel::AV_CHAN_BACK_LEFT as i32,
		BackRight           = AVChannel::AV_CHAN_BACK_RIGHT as i32,
		FrontLeftOfCenter   = AVChannel::AV_CHAN_FRONT_LEFT_OF_CENTER as i32,
		FrontRightOfCenter  = AVChannel::AV_CHAN_FRONT_RIGHT_OF_CENTER as i32,
		BackCenter          = AVChannel::AV_CHAN_BACK_CENTER as i32,
		SideLeft            = AVChannel::AV_CHAN_SIDE_LEFT as i32,
		SideRight           = AVChannel::AV_CHAN_SIDE_RIGHT as i32,
		TopCenter           = AVChannel::AV_CHAN_TOP_CENTER as i32,
		TopFrontLeft        = AVChannel::AV_CHAN_TOP_FRONT_LEFT as i32,
		TopFrontCenter      = AVChannel::AV_CHAN_TOP_FRONT_CENTER as i32,
		TopFrontRight       = AVChannel::AV_CHAN_TOP_FRONT_RIGHT as i32,
		TopBackLeft         = AVChannel::AV_CHAN_TOP_BACK_LEFT as i32,
		TopBackCenter       = AVChannel::AV_CHAN_TOP_BACK_CENTER as i32,
		TopBackRight        = AVChannel::AV_CHAN_TOP_BACK_RIGHT as i32,
		StereoLeft          = AVChannel::AV_CHAN_STEREO_LEFT as i32,
		StereoRight         = AVChannel::AV_CHAN_STEREO_RIGHT as i32,
		WideLeft            = AVChannel::AV_CHAN_WIDE_LEFT as i32,
		WideRight           = AVChannel::AV_CHAN_WIDE_RIGHT as i32,
		SurroundDirectLeft  = AVChannel::AV_CHAN_SURROUND_DIRECT_LEFT as i32,
		SurroundDirectRight = AVChannel::AV_CHAN_SURROUND_DIRECT_RIGHT as i32,
		LowFrequency2       = AVChannel::AV_CHAN_LOW_FREQUENCY_2 as i32,
		TopSideLeft         = AVChannel::AV_CHAN_TOP_SIDE_LEFT as i32,
		TopSideRight        = AVChannel::AV_CHAN_TOP_SIDE_RIGHT as i32,
		BottomFrontCenter   = AVChannel::AV_CHAN_BOTTOM_FRONT_CENTER as i32,
		BottomFrontLeft     = AVChannel::AV_CHAN_BOTTOM_FRONT_LEFT as i32,
		BottomFrontRight    = AVChannel::AV_CHAN_BOTTOM_FRONT_RIGHT as i32,
		Unused              = AVChannel::AV_CHAN_UNUSED as i32,
		Unknown             = AVChannel::AV_CHAN_UNKNOWN as i32,
		AmbisonicBase       = AVChannel::AV_CHAN_AMBISONIC_BASE as i32,
		AmbisonicEnd        = AVChannel::AV_CHAN_AMBISONIC_END as i32
	}
}

define_av_alias! {
	#[repr(u64)]
	#[derive(Default, FromPrimitive)]
	pub enum ChannelBit {
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
		None     = AVDiscard::AVDISCARD_NONE as i32,
		#[default]
		Default  = AVDiscard::AVDISCARD_DEFAULT as i32,
		NonRef   = AVDiscard::AVDISCARD_NONREF as i32,
		Bidir    = AVDiscard::AVDISCARD_BIDIR as i32,
		NonIntra = AVDiscard::AVDISCARD_NONINTRA as i32,
		NonKey   = AVDiscard::AVDISCARD_NONKEY as i32,
		All      = AVDiscard::AVDISCARD_ALL as i32
	}
}

define_av_alias_casts! {
	#[repr(u32)]
	pub enum ChannelOrder = AVChannelOrder {
		#[default]
		Unspec    = AVChannelOrder::AV_CHANNEL_ORDER_UNSPEC as u32,
		Native    = AVChannelOrder::AV_CHANNEL_ORDER_NATIVE as u32,
		Custom    = AVChannelOrder::AV_CHANNEL_ORDER_CUSTOM as u32,
		Ambisonic = AVChannelOrder::AV_CHANNEL_ORDER_AMBISONIC as u32
	}
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ChannelCustom {
	pub id: Channel,
	pub name: [u8; 16]
}

#[derive(Debug, Clone)]
pub enum ChannelLayout {
	Unspec(u16),
	Native(u16, u64),
	Custom(Vec<ChannelCustom>),
	Ambisonic(u16, u64)
}

impl ChannelLayout {
	pub const LAYOUT_22POINT2: Self = Self::Native(24, AV_CH_LAYOUT_22POINT2);
	pub const LAYOUT_2POINT1: Self = Self::Native(3, AV_CH_LAYOUT_2POINT1);
	pub const LAYOUT_2_1: Self = Self::Native(3, AV_CH_LAYOUT_2_1);
	pub const LAYOUT_2_2: Self = Self::Native(4, AV_CH_LAYOUT_2_2);
	pub const LAYOUT_3POINT1: Self = Self::Native(4, AV_CH_LAYOUT_3POINT1);
	pub const LAYOUT_3POINT1POINT2: Self = Self::Native(6, AV_CH_LAYOUT_3POINT1POINT2);
	pub const LAYOUT_4POINT0: Self = Self::Native(4, AV_CH_LAYOUT_4POINT0);
	pub const LAYOUT_4POINT1: Self = Self::Native(5, AV_CH_LAYOUT_4POINT1);
	pub const LAYOUT_5POINT0: Self = Self::Native(5, AV_CH_LAYOUT_5POINT0);
	pub const LAYOUT_5POINT0_BACK: Self = Self::Native(5, AV_CH_LAYOUT_5POINT0_BACK);
	pub const LAYOUT_5POINT1: Self = Self::Native(6, AV_CH_LAYOUT_5POINT1);
	pub const LAYOUT_5POINT1POINT2_BACK: Self = Self::Native(8, AV_CH_LAYOUT_5POINT1POINT2_BACK);
	pub const LAYOUT_5POINT1POINT4_BACK: Self = Self::Native(10, AV_CH_LAYOUT_5POINT1POINT4_BACK);
	pub const LAYOUT_5POINT1_BACK: Self = Self::Native(6, AV_CH_LAYOUT_5POINT1_BACK);
	pub const LAYOUT_6POINT0: Self = Self::Native(6, AV_CH_LAYOUT_6POINT0);
	pub const LAYOUT_6POINT0_FRONT: Self = Self::Native(6, AV_CH_LAYOUT_6POINT0_FRONT);
	pub const LAYOUT_6POINT1: Self = Self::Native(7, AV_CH_LAYOUT_6POINT1);
	pub const LAYOUT_6POINT1_BACK: Self = Self::Native(7, AV_CH_LAYOUT_6POINT1_BACK);
	pub const LAYOUT_6POINT1_FRONT: Self = Self::Native(7, AV_CH_LAYOUT_6POINT1_FRONT);
	pub const LAYOUT_7POINT0: Self = Self::Native(7, AV_CH_LAYOUT_7POINT0);
	pub const LAYOUT_7POINT0_FRONT: Self = Self::Native(7, AV_CH_LAYOUT_7POINT0_FRONT);
	pub const LAYOUT_7POINT1: Self = Self::Native(8, AV_CH_LAYOUT_7POINT1);
	pub const LAYOUT_7POINT1POINT2: Self = Self::Native(10, AV_CH_LAYOUT_7POINT1POINT2);
	pub const LAYOUT_7POINT1POINT4_BACK: Self = Self::Native(12, AV_CH_LAYOUT_7POINT1POINT4_BACK);
	pub const LAYOUT_7POINT1_TOP_BACK: Self = Self::LAYOUT_5POINT1POINT2_BACK;
	pub const LAYOUT_7POINT1_WIDE: Self = Self::Native(8, AV_CH_LAYOUT_7POINT1_WIDE);
	pub const LAYOUT_7POINT1_WIDE_BACK: Self = Self::Native(8, AV_CH_LAYOUT_7POINT1_WIDE_BACK);
	pub const LAYOUT_7POINT2POINT3: Self = Self::Native(12, AV_CH_LAYOUT_7POINT2POINT3);
	pub const LAYOUT_9POINT1POINT4_BACK: Self = Self::Native(14, AV_CH_LAYOUT_9POINT1POINT4_BACK);
	pub const LAYOUT_AMBISONIC_FIRST_ORDER: Self = Self::Ambisonic(4, 0);
	pub const LAYOUT_CUBE: Self = Self::Native(8, AV_CH_LAYOUT_CUBE);
	pub const LAYOUT_HEXADECAGONAL: Self = Self::Native(16, AV_CH_LAYOUT_HEXADECAGONAL);
	pub const LAYOUT_HEXAGONAL: Self = Self::Native(6, AV_CH_LAYOUT_HEXAGONAL);
	pub const LAYOUT_MONO: Self = Self::Native(1, AV_CH_LAYOUT_MONO);
	pub const LAYOUT_OCTAGONAL: Self = Self::Native(8, AV_CH_LAYOUT_OCTAGONAL);
	pub const LAYOUT_QUAD: Self = Self::Native(4, AV_CH_LAYOUT_QUAD);
	pub const LAYOUT_STEREO: Self = Self::Native(2, AV_CH_LAYOUT_STEREO);
	pub const LAYOUT_STEREO_DOWNMIX: Self = Self::Native(2, AV_CH_LAYOUT_STEREO_DOWNMIX);
	pub const LAYOUT_SURROUND: Self = Self::Native(3, AV_CH_LAYOUT_SURROUND);

	pub fn get_default_for_count(channels: u16) -> Self {
		const LAYOUT_MAP: &[ChannelLayout] = &[
			ChannelLayout::LAYOUT_MONO,
			ChannelLayout::LAYOUT_STEREO,
			ChannelLayout::LAYOUT_2POINT1,
			ChannelLayout::LAYOUT_SURROUND,
			ChannelLayout::LAYOUT_2_1,
			ChannelLayout::LAYOUT_4POINT0,
			ChannelLayout::LAYOUT_QUAD,
			ChannelLayout::LAYOUT_2_2,
			ChannelLayout::LAYOUT_3POINT1,
			ChannelLayout::LAYOUT_5POINT0_BACK,
			ChannelLayout::LAYOUT_5POINT0,
			ChannelLayout::LAYOUT_4POINT1,
			ChannelLayout::LAYOUT_5POINT1_BACK,
			ChannelLayout::LAYOUT_5POINT1,
			ChannelLayout::LAYOUT_6POINT0,
			ChannelLayout::LAYOUT_6POINT0_FRONT,
			ChannelLayout::LAYOUT_3POINT1POINT2,
			ChannelLayout::LAYOUT_HEXAGONAL,
			ChannelLayout::LAYOUT_6POINT1,
			ChannelLayout::LAYOUT_6POINT1_BACK,
			ChannelLayout::LAYOUT_6POINT1_FRONT,
			ChannelLayout::LAYOUT_7POINT0,
			ChannelLayout::LAYOUT_7POINT0_FRONT,
			ChannelLayout::LAYOUT_7POINT1,
			ChannelLayout::LAYOUT_7POINT1_WIDE_BACK,
			ChannelLayout::LAYOUT_7POINT1_WIDE,
			ChannelLayout::LAYOUT_5POINT1POINT2_BACK,
			ChannelLayout::LAYOUT_OCTAGONAL,
			ChannelLayout::LAYOUT_CUBE,
			ChannelLayout::LAYOUT_5POINT1POINT4_BACK,
			ChannelLayout::LAYOUT_7POINT1POINT2,
			ChannelLayout::LAYOUT_7POINT1POINT4_BACK,
			ChannelLayout::LAYOUT_7POINT2POINT3,
			ChannelLayout::LAYOUT_9POINT1POINT4_BACK,
			ChannelLayout::LAYOUT_HEXADECAGONAL,
			ChannelLayout::LAYOUT_STEREO_DOWNMIX,
			ChannelLayout::LAYOUT_22POINT2
		];

		for layout in LAYOUT_MAP {
			if channels == layout.channel_count() {
				return layout.clone();
			}
		}

		Self::Unspec(channels)
	}

	/// # Panics
	/// if the custom channels count cannot fit into a u16
	pub fn channel_count(&self) -> u16 {
		#[allow(clippy::unwrap_used)]
		match self {
			Self::Unspec(channels) => *channels,
			Self::Native(channels, _) | Self::Ambisonic(channels, _) => *channels,
			Self::Custom(custom) => custom.len().try_into().unwrap()
		}
	}
}

impl Default for ChannelLayout {
	fn default() -> Self {
		Self::Unspec(0)
	}
}
