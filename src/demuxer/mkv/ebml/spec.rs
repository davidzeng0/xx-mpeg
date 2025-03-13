#![allow(clippy::unreadable_literal)]

use super::*;

ebml_define! {
	#[allow(dead_code)]
	pub struct Header {
		pub version: Unsigned @ 0x4286 = 1,
		pub reader_version: NonZeroUnsigned @ 0x42f7 = 1,
		pub max_id_length: Unsigned @ 0x42f2 = 4,
		pub max_size_length: NonZeroUnsigned @ 0x42f3 = 8,
		pub doc_type: NonEmptyString @ 0x4282,
		pub doc_type_version: NonZeroUnsigned @ 0x4287 = 1,
		pub doc_type_reader_version: NonZeroUnsigned @ 0x4285 = 1,
		pub doc_type_extensions: Option<Vec<DocTypeExtension>> @ 0x4281
	}

	fn check(&mut self) -> Result<()> {
		if self.max_id_length.0 >= 4 {
			Ok(())
		} else {
			Err(FormatError::InvalidData("Invalid value for `MaxIdLength`".into()).into())
		}
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct DocTypeExtension {
		pub name: NonEmptyString @ 0x4283,
		pub version: NonZeroUnsigned @ 0x4284
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Crc32(pub u32);
}

ebml_define! {
	pub struct Void(pub ());
}

ebml_define! {
	#[allow(dead_code)]
	pub struct EbmlRoot {
		pub header: Header @ 0x1a45dfa3,
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct EbmlGlobal {
		pub crc32: Option<Crc32> @ 0xbf,
		pub void: Option<Vec<Void>> @ 0xec
	}
}
