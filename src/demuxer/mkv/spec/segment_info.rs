use super::*;

ebml_define! {
	#[allow(dead_code)]
	pub struct SegmentInfo {
		pub uuid: Option<UUID> @ 0x73a4,
		pub file_name: Option<String> @ 0x7384,
		pub prev_uuid: Option<UUID> @ 0x3cb923,
		pub prev_file_name: Option<String> @ 0x3c83ab,
		pub next_uuid: Option<UUID> @ 0x3eb923,
		pub next_file_name: Option<String> @ 0x3e83bb,
		pub family: Option<Vec<UUID>> @ 0x4444,
		pub chapter_translate: Option<Vec<ChapterTranslate>> @ 0x6924,
		pub timestamp_scale: NonZeroUnsigned @ 0x2ad7b1 = 1_000_000,
		pub duration: Option<PositiveFloat> @ 0x4489,
		pub date: Option<Date> @ 0x4461,
		pub title: Option<String> @ 0x7ba9,
		pub muxing_app: Option<String> @ 0x4d80,
		pub writing_app: Option<String> @ 0x5741
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct ChapterTranslate {
		pub id: Bytes @ 0x69a5,
		pub codec: TranslateCodec @ 0x69bf,
		pub edition_uids: Vec<Unsigned> @ 0x69fc
	}
}
