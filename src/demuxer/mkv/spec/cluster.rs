use super::*;

ebml_element! {
	struct Cluster {
		const ID = 0x1f43b675;

		timestamp: Timestamp,
		silent_tracks: Option<SilentTracks>,
		position: Option<Position>,
		prev_size: Option<PrevSize>,
		simple_blocks: Vec<SimpleBlock>,
		block_groups: Vec<BlockGroup>
	}
}

ebml_element! {
	struct Timestamp {
		const ID = 0xe7;

		value: vint
	}
}

ebml_element! {
	struct SilentTracks {
		const ID = 0x5854;

		numbers: Vec<Number>
	}
}

ebml_element! {
	struct Number {
		const ID = 0x58d7;

		value: vint
	}
}

ebml_element! {
	struct Position {
		const ID = 0xa7;

		value: vint
	}
}

ebml_element! {
	struct PrevSize {
		const ID = 0xab;

		value: vint
	}
}

ebml_element! {
	struct SimpleBlock {
		const ID = 0xa3;

		data: Vec<u8>
	}
}

ebml_element! {
	struct BlockGroup {
		const ID = 0xa0;

		block: Block,
		block_virtual: Option<BlockVirtual>,
		additions: Option<BlockAdditions>,
		duration: Option<Duration>,
		reference_priority: Option<ReferencePriority>,
		reference_block: Option<ReferenceBlock>,
		reference_virtual: Option<ReferenceVirtual>,
		codec_state: Option<CodecState>,
		discard_padding: Option<DiscardPadding>
	}
}

ebml_element! {
	struct Block {
		const ID = 0xa1;

		data: Vec<u8>
	}
}

ebml_element! {
	struct BlockVirtual {
		const ID = 0xa2;

		data: Vec<u8>
	}
}

ebml_element! {
	struct BlockAdditions {
		const ID = 0x75a1;

		more: Vec<More>
	}
}

ebml_element! {
	struct More {
		const ID = 0xa6;

		additional: Additional,
		id: Id
	}
}

ebml_element! {
	struct Additional {
		const ID = 0xa5;

		data: Vec<u8>
	}
}

ebml_element! {
	struct Id {
		const ID = 0xee;

		value: vint = 1
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Block add ID cannot be zero"))
		}
	}
}

ebml_element! {
	struct Duration {
		const ID = 0x9b;

		value: vint
	}
}

ebml_element! {
	struct ReferencePriority {
		const ID = 0xfa;

		value: vint
	}
}

ebml_element! {
	struct ReferenceBlock {
		const ID = 0xfb;

		value: vint
	}
}

ebml_element! {
	struct ReferenceVirtual {
		const ID = 0xfd;

		value: vint
	}
}

ebml_element! {
	struct CodecState {
		const ID = 0xa4;

		value: Vec<u8>
	}
}

ebml_element! {
	struct DiscardPadding {
		const ID = 0x75a2;

		value: vint
	}
}
