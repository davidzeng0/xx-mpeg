use super::*;

ebml_element! {
	struct Translate {
		const ID = 0x6624;

		id: Id,
		codec: Codec,
		edition_uids: Vec<EditionUID>
	}
}

ebml_element! {
	struct Id {
		const ID = 0x66a5;

		value: Vec<u8>
	}
}

ebml_element! {
	struct Codec {
		const ID = 0x66bf;

		value: vint
	}
}

ebml_element! {
	struct EditionUID {
		const ID = 0x66fc;

		value: vint
	}
}
