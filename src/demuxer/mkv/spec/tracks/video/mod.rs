use super::*;

pub mod color;
pub mod projection;

use self::color::*;
use self::projection::*;

ebml_define! {
	#[repr(Unsigned)]
	pub enum Interlacing {
		Undetermined = 0,
		Interlaced   = 1,
		Progressive  = 2
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum FieldOrder {
		Progressive  = 0,
		Tff          = 1,
		Undetermined = 2,
		Bff          = 6,
		BffSwapped   = 9,
		TffSwapped   = 14
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum StereoMode {
		Mono                   = 0,
		SideBySideLeft         = 1,
		TopBottomRight         = 2,
		TopBottomLeft          = 3,
		CheckboardRight        = 4,
		CheckboardLeft         = 5,
		RowInterleavedRight    = 6,
		RowInterleavedLeft     = 7,
		ColumnInterleavedRight = 8,
		ColumnInterleavedLeft  = 9,
		AnaglyphCyanRed        = 10,
		SideBySideRight        = 11,
		AnaglyphGreenMagenta   = 12,
		BothEyesLacedLeft      = 13,
		BothEyesLacedRight     = 14
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum AlphaMode {
		None    = 0,
		Present = 1
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum OldStereoMode {
		Mono  = 0,
		Right = 1,
		Left  = 2,
		Both  = 3
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum DisplayUnit {
		Pixels      = 0,
		Centimeters = 1,
		Inches      = 2,
		AspectRatio = 3,
		Unknown     = 4
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum AspectRatioType {
		FreeResizing = 0,
		KeepRatio    = 1,
		Fixed        = 2
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Video {
		pub interlaced: Interlacing @ 0x9a = Interlacing::Undetermined,
		pub field_order: FieldOrder @ 0x9d = FieldOrder::Undetermined,
		pub stereo_mode: StereoMode @ 0x53b8 = StereoMode::Mono,
		pub alpha_mode: AlphaMode @ 0x53c0 = AlphaMode::None,
		pub old_stereo_mode: Option<OldStereoMode> @ 0x53b9,
		pub pixel_width: NonZeroUnsigned @ 0xb0,
		pub pixel_height: NonZeroUnsigned @ 0xba,
		pub pixel_crop_bottom: Unsigned @ 0x54aa = 0,
		pub pixel_crop_top: Unsigned @ 0x54bb = 0,
		pub pixel_crop_left: Unsigned @ 0x54cc = 0,
		pub pixel_crop_right: Unsigned @ 0x54dd = 0,
		pub display_width: Option<NonZeroUnsigned> @ 0x54b0,
		pub display_height: Option<NonZeroUnsigned> @ 0x54ba,
		pub display_unit: DisplayUnit @ 0x54b2 = DisplayUnit::Pixels,
		pub aspect_ratio_type: Option<AspectRatioType> @ 0x54b3,
		pub uncompressed_four_cc: Option<Bytes> @ 0x2eb524,
		pub gamma_value: Option<PositiveFloat> @ 0x2fb523,
		pub framerate: Option<PositiveFloat> @ 0x2383e3,
		pub color: Option<Color> @ 0x55b0,
		pub projection: Option<Projection> @ 0x7670
	}
}
