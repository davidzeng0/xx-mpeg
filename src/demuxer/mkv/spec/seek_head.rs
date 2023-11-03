use super::*;

ebml_element! {
	struct SeekHead {
		const ID = 0x114d9b74;

		seek: Vec<Seek>
	}
}

ebml_element! {
	struct Seek {
		const ID = 0x4dbb;

		id: Id,
		position: Position
	}
}

ebml_element! {
	struct Id {
		const ID = 0x53ab;

		value: vint
	}
}

ebml_element! {
	struct Position {
		const ID = 0x53ac;

		value: vint
	}
}
