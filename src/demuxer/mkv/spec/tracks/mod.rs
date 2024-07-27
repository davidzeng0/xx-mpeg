use super::*;

pub mod audio;
pub mod content_encoding;
pub mod translate;
pub mod video;

use self::audio::*;
use self::content_encoding::*;
use self::translate::*;
use self::video::*;

ebml_define! {
	pub struct Tracks {
		pub tracks: Vec<Track> @ 0xae
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum TrackType {
		Video    = 0x01,
		Audio    = 0x02,
		Complex  = 0x03,
		Logo     = 0x10,
		Subtitle = 0x11,
		Button   = 0x12,
		Control  = 0x20,
		Metadata = 0x21
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Track {
		pub number: NonZeroUnsigned @ 0xd7,
		pub uid: NonZeroUnsigned @ 0x73c5,
		#[rename = "type"]
		pub ty: TrackType @ 0x83,
		pub enabled: bool @ 0xb9 = true,
		pub default: bool @ 0x88 = true,
		pub forced: bool @ 0x55aa = false,
		pub hearing_impaired: Option<bool> @ 0x55ab,
		pub visual_impaired: Option<bool> @ 0x55ac,
		pub text_descriptions: Option<bool> @ 0x55ad,
		pub original: Option<bool> @ 0x55ae,
		pub commentary: Option<bool> @ 0x55af,
		pub lacing: bool @ 0x9c = true,
		pub min_cache: Unsigned @ 0x6de7 = 0,
		pub max_cache: Option<Unsigned> @ 0x6df8,
		pub default_duration: Option<NonZeroUnsigned> @ 0x23e383,
		pub default_decoded_field_duration: Option<NonZeroUnsigned> @ 0x234e7a,
		pub track_timestamp_scale: PositiveFloat @ 0x23314f = 1.0,
		pub track_offset: Option<Signed> @ 0x537f,
		pub max_block_addition_id: Unsigned @ 0x55ee = 0,
		pub block_addition_mappings: Option<Vec<BlockAdditionMapping>> @ 0x41e4,
		pub name: Option<String> @ 0x536e,
		pub language: String @ 0x22b59d = "eng",
		pub codec_id: String @ 0x86,
		pub codec_private: Option<Bytes> @ 0x63a2,
		pub codec_name: Option<String> @ 0x258688,
		pub attachment_link: Option<NonZeroUnsigned> @ 0x7446,
		pub codec_settings: Option<String> @ 0x3a9697,
		pub codec_info_url: Option<Vec<String>> @ 0x3b4040,
		pub codec_download_url: Option<Vec<String>> @ 0x26b240,
		pub codec_decode_all: bool @ 0xaa = true,
		pub overlay: Option<Vec<Unsigned>> @ 0x6fab,
		pub codec_delay: Unsigned @ 0x56aa = 0,
		pub seek_preroll: Unsigned @ 0x56bb = 0,
		pub translate: Option<Vec<Translate>> @ 0x6624,
		pub video: Option<Video> @ 0xe0,
		pub audio: Option<Audio> @ 0xe1,
		pub content_encodings: Option<ContentEncodings> @ 0x6d80
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct BlockAdditionMapping {
		pub add_id_value: Option<Unsigned> @ 0x41f0,
		pub add_id_name: Option<String> @ 0x41a4,
		pub add_id_type: Unsigned @ 0x41e7 = 0,
		pub add_id_extra_data: Option<Bytes> @ 0x41ed
	}

	fn check(&mut self) -> Result<()> {
		if !self.add_id_value.as_ref().is_some_and(|val| val.0 < 2) {
			Ok(())
		} else {
			Err(FormatError::InvalidData("`AddIdValue` must be >= 2".into()).into())
		}
	}
}
