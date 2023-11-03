use super::*;

mod chapter_translate;
pub use chapter_translate::*;

ebml_element! {
	struct SegmentInfo {
		const ID = 0x1549a966;

		uuid: Option<SegmentUUID>,
		filename: Option<SegmentFilename>,
		prev_uuid: Option<SegmentUUID>,
		prev_filename: Option<PrevFilename>,
		next_uuid: Option<NextUUID>,
		next_filename: Option<NextFilename>,
		family: Vec<Family>,
		chapter_translate: Vec<ChapterTranslate>,
		timestamp_scale: TimestampScale,
		duration: Option<Duration>,
		date: Option<Date>,
	}
}

ebml_element! {
	struct SegmentUUID {
		const ID = 0x73a4;

		value: u128
	}
}

ebml_element! {
	struct SegmentFilename {
		const ID = 0x7384;

		value: String
	}
}

ebml_element! {
	struct PrevUUID {
		const ID = 0x3cb923;

		value: u128
	}
}

ebml_element! {
	struct PrevFilename {
		const ID = 0x3c83ab;

		value: String
	}
}

ebml_element! {
	struct NextUUID {
		const ID = 0x3eb923;

		value: u128
	}
}

ebml_element! {
	struct NextFilename {
		const ID = 0x3e83bb;

		value: String
	}
}

ebml_element! {
	struct Family {
		const ID = 0x4444;

		value: u128
	}
}

ebml_element! {
	struct TimestampScale {
		const ID = 0x2ad7b1;

		value: vint = 1_000_000
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Timestamp scale cannot be zero"))
		}
	}
}

ebml_element! {
	struct Duration {
		const ID = 0x4489;

		value: vfloat
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value > 0.0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Duration must be positive"))
		}
	}
}

ebml_element! {
	struct Date {
		const ID = 0x4461;

		value: vint
	}
}

ebml_element! {
	struct Title {
		const ID = 0x7ba9;

		value: String
	}
}

ebml_element! {
	struct MuxingApp {
		const ID = 0x4d80;

		value: String
	}
}

ebml_element! {
	struct WritingApp {
		const ID = 0x5741;

		value: String
	}
}
