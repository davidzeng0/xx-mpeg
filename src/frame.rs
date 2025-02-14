use av::AVFrame;

use self::av::MediaType;
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
	pub ch_layout: ChannelLayout,
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

	/// # Panics
	/// if some of the fields in the frame are out of range
	#[allow(clippy::unwrap_used)]
	pub(crate) fn copy_fields_to(&self, frame: &mut AVFrame) {
		frame.time_base = self.time_base.into();
		frame.pkt_dts = self.decode_timestamp;
		frame.pts = self.presentation_timestamp;
		frame.duration = self.duration.try_into().unwrap();

		frame.sample_rate = self.sample_rate.try_into().unwrap();
		frame.ch_layout = (&self.ch_layout).into();

		frame.pict_type = self.picture_type.into();
		frame.sample_aspect_ratio = self.sample_aspect_ratio.into();
		frame.repeat_pict = self.repeat_picture;
	}

	/// # Panics
	/// if some of the fields in the frame are out of range
	#[allow(clippy::unwrap_used, clippy::cast_sign_loss)]
	pub(crate) fn get_fields_from_inner(&mut self, codec_type: Option<MediaType>) {
		self.time_base = self.data.time_base.into();
		self.decode_timestamp = self.data.pkt_dts;
		self.presentation_timestamp = self.data.pts;
		self.duration = self.data.duration.try_into().unwrap();
		self.flags = BitFlags::from_bits_truncate(self.data.flags as u32);

		self.samples = self.data.nb_samples.try_into().unwrap();
		self.sample_rate = self.data.sample_rate.try_into().unwrap();
		self.ch_layout = (&self.data.ch_layout).into();
		self.sample_format = SampleFormat::None;

		self.picture_type = self.data.pict_type.into();
		self.sample_aspect_ratio = self.data.sample_aspect_ratio.into();
		self.width = self.data.width.try_into().unwrap();
		self.height = self.data.height.try_into().unwrap();
		self.repeat_picture = self.data.repeat_pict;

		match codec_type {
			Some(MediaType::Video) => self.pixel_format = self.data.format.into(),
			Some(MediaType::Audio) => self.sample_format = self.data.format.into(),
			_ => ()
		}
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
			ch_layout: ChannelLayout::default(),
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
