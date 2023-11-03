use xx_mpeg_macros::ebml_element;

use super::*;

ebml_element! {
	struct Header {
		const ID = 0x1a45dfa3;

		version: Version,
		reader_version: ReaderVersion,
		max_id_length: MaxIdLength,
		max_size_length: MaxSizeLength,
		doc_type: DocType,
		doc_type_version: DocTypeVersion,
		doc_type_reader_version: DocTypeReaderVersion,
		doc_type_extension: Vec<DocTypeExtension>
	}
}

ebml_element! {
	struct Version {
		const ID = 0x4286;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML version cannot be zero"))
		}
	}
}

ebml_element! {
	struct ReaderVersion {
		const ID = 0x42f7;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML reader version cannot be zero"))
		}
	}
}

ebml_element! {
	struct MaxIdLength {
		const ID = 0x42f2;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value >= 4 && self.value <= 8 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML max id length out of range"))
		}
	}
}

ebml_element! {
	struct MaxSizeLength {
		const ID = 0x42f3;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML max size length cannot be zero"))
		}
	}
}

ebml_element! {
	struct DocType {
		const ID = 0x4282;

		value: String
	}

	fn post_parse(&mut self) -> Result<()> {
		if !self.value.is_empty() {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML doc type cannot be empty"))
		}
	}
}

ebml_element! {
	struct DocTypeVersion {
		const ID = 0x4287;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML doc type version cannot be zero"))
		}
	}
}

ebml_element! {
	struct DocTypeReaderVersion {
		const ID = 0x4285;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML doc type reader version cannot be zero"))
		}
	}
}

ebml_element! {
	struct DocTypeExtension {
		const ID = 0x4281;

		name: DocTypeExtensionName,
		version: DocTypeExtensionVersion
	}
}

ebml_element! {
	struct DocTypeExtensionName {
		const ID = 0x4283;

		value: String
	}

	fn post_parse(&mut self) -> Result<()> {
		if !self.value.is_empty() {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML doc type extension cannot be empty"))
		}
	}
}

ebml_element! {
	struct DocTypeExtensionVersion {
		const ID = 0x4283;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "EBML doc type extension version cannot be zero"))
		}
	}
}

ebml_element! {
	struct Crc32 {
		const ID = 0xbf;

		value: u32
	}
}

ebml_element! {
	struct Void {
		const ID = 0xec;
	}
}
