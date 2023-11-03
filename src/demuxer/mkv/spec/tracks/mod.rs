use super::*;

mod translate;
use num_derive::FromPrimitive;
pub use translate::*;
mod video;
pub use video::*;
mod audio;
pub use audio::*;
mod content_encoding;
pub use content_encoding::*;

#[derive(FromPrimitive)]
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

ebml_element! {
	struct Tracks {
		const ID = 0x1654ae6b;

		tracks: Vec<Track>
	}
}

ebml_element! {
	struct Track {
		const ID = 0xae;

		number: Number,
		uid: UID,
		ty: Type,
		enabled: Enabled,
		default: Default,
		forced: Forced,
		hearing_impaired: Option<HearingImpaired>,
		visual_impaired: Option<VisualImpaired>,
		text_descriptions: Option<TextDescriptions>,
		original: Option<Original>,
		commentary: Option<Commentary>,
		lacing: Lacing,
		min_cache: MinCache,
		max_cache: Option<MaxCache>,
		default_duration: Option<DefaultDuration>,
		default_decoded_field_duration: Option<DefaultDecodedFieldDuration>,
		track_timestamp_scale: TimestampScale,
		track_offset: Option<Offset>,
		name: Option<Name>,
		language: Language,
		codec_id: CodecId,
		codec_private: Option<CodecPrivate>,
		codec_name: Option<CodecName>,
		attachment_link: Option<AttachmentLink>,
		codec_settings: Option<CodecSettings>,
		codec_info_url: Vec<CodecInfoUrl>,
		codec_download_url: Vec<CodecDownloadUrl>,
		codec_decode_all: CodecDecodeAll,
		overlay: Vec<Overlay>,
		codec_delay: CodecDelay,
		seek_preroll: SeekPreroll,
		translate: Vec<Translate>,
		video: Option<Video>,
		audio: Option<Audio>,
		content_encodings: Option<ContentEncodings>
	}
}

ebml_element! {
	struct Number {
		const ID = 0xd7;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Track number cannot be zero"))
		}
	}
}

ebml_element! {
	struct UID {
		const ID = 0x73c5;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Track UID cannot be zero"))
		}
	}
}

ebml_element! {
	struct Type {
		const ID = 0x83;

		value: vint
	}
}

ebml_element! {
	struct Enabled {
		const ID = 0xb9;

		value: vint = 1
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct Default {
		const ID = 0x88;

		value: vint = 1
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct Forced {
		const ID = 0x55aa;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct HearingImpaired {
		const ID = 0x55ab;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct VisualImpaired {
		const ID = 0x55ac;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct TextDescriptions {
		const ID = 0x55ad;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct Original {
		const ID = 0x55ae;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct Commentary {
		const ID = 0x55af;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct Lacing {
		const ID = 0x9c;

		value: vint = 1
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct MinCache {
		const ID = 0x6de7;

		value: vint
	}
}

ebml_element! {
	struct MaxCache {
		const ID = 0x6df8;

		value: vint
	}
}

ebml_element! {
	struct DefaultDuration {
		const ID = 0x23e383;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Default duration cannot be zero"))
		}
	}
}

ebml_element! {
	struct DefaultDecodedFieldDuration {
		const ID = 0x234e7a;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Default decoded field duration cannot be zero"))
		}
	}
}

ebml_element! {
	struct TimestampScale {
		const ID = 0x23314f;

		value: vfloat = 1.0
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value > 0.0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Track timestamp scale must be positive"))
		}
	}
}

ebml_element! {
	struct Offset {
		const ID = 0x537f;

		value: vint
	}
}

ebml_element! {
	struct Name {
		const ID = 0x536e;

		value: String
	}
}

ebml_element! {
	struct Language {
		const ID = 0x22b59d;

		value: String = "eng"
	}
}

ebml_element! {
	struct CodecId {
		const ID = 0x86;

		value: String
	}
}

ebml_element! {
	struct CodecPrivate {
		const ID = 0x63a2;

		value: Vec<u8>
	}
}

ebml_element! {
	struct CodecName {
		const ID = 0x258688;

		value: String
	}
}

ebml_element! {
	struct AttachmentLink {
		const ID = 0x7446;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Attachment link cannot be zero"))
		}
	}
}

ebml_element! {
	struct CodecSettings {
		const ID = 0x3a9697;

		value: String
	}
}

ebml_element! {
	struct CodecInfoUrl {
		const ID = 0x3b4040;

		value: String
	}
}

ebml_element! {
	struct CodecDownloadUrl {
		const ID = 0x26b240;

		value: String
	}
}

ebml_element! {
	struct CodecDecodeAll {
		const ID = 0xaa;

		value: vint = 1
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value <= 1 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid value for boolean"))
		}
	}
}

ebml_element! {
	struct Overlay {
		const ID = 0x6fab;

		value: vint
	}
}

ebml_element! {
	struct CodecDelay {
		const ID = 0x56aa;

		value: vint
	}
}

ebml_element! {
	struct SeekPreroll {
		const ID = 0x56bb;

		value: vint
	}
}
