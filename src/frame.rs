use av::AVFrame;

use super::*;

#[allow(clippy::partial_pub_fields)]
pub struct Frame {
	pub(crate) data: AVFrame,

	pub time_base: Rational,
	pub decode_timestamp: i64,
	pub presentation_timestamp: i64,
	pub duration: u64,
	pub flags: BitFlags<FrameFlag>,

	pub samples: u32,
	pub sample_rate: u32,
	pub channels: u16,
	pub channel_layout: u64,
	pub sample_format: SampleFormat,

	pub picture_type: PictureType,
	pub sample_aspect_ratio: Rational,
	pub width: u32,
	pub height: u32,
	pub repeat_picture: i32,
	pub pixel_format: PixelFormat
}

impl Frame {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for Frame {
	fn default() -> Self {
		Self {
			data: AVFrame::new(),

			time_base: Rational::default(),
			decode_timestamp: UNKNOWN_TIMESTAMP,
			presentation_timestamp: UNKNOWN_TIMESTAMP,
			duration: 0,
			flags: BitFlags::default(),

			samples: 0,
			sample_rate: 0,
			channels: 0,
			channel_layout: 0,
			sample_format: SampleFormat::None,

			picture_type: PictureType::default(),
			sample_aspect_ratio: Rational::default(),
			width: 0,
			height: 0,
			repeat_picture: 0,
			pixel_format: PixelFormat::None
		}
	}
}
