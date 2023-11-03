use super::*;

ebml_element! {
	struct ChapterTranslate {
		const ID = 0x6924;

		id: Id,
		codec: Codec,
		edition_uids: Vec<EditionUID>
	}
}

ebml_element! {
	struct Id {
		const ID = 0x69a5;

		value: Vec<u8>
	}
}

ebml_element! {
	struct Codec {
		const ID = 0x69bf;

		value: vint
	}
}

ebml_element! {
	struct EditionUID {
		const ID = 0x69fc;

		value: vint
	}
}
