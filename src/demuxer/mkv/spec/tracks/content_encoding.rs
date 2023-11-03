use super::*;

ebml_element! {
	struct ContentEncodings {
		const ID = 0x6d80;

		encoding: Vec<Encoding>
	}
}

ebml_element! {
	struct Encoding {
		const ID = 0x6240;

		order: Order,
		scope: Scope,
		ty: Type,
		compression: Option<Compression>,
		encryption: Option<Encryption>
	}
}

ebml_element! {
	struct Order {
		const ID = 0x5031;

		value: vint
	}
}

ebml_element! {
	struct Scope {
		const ID = 0x5032;

		scope: vint = 1
	}
}

ebml_element! {
	struct Type {
		const ID = 0x5033;

		value: vint
	}
}

ebml_element! {
	struct Compression {
		const ID = 0x5034;

		algorithm: CompAlgorithm,
		settings: Option<CompSettings>
	}
}

ebml_element! {
	struct CompAlgorithm {
		const ID = 0x4254;

		value: vint = 0
	}
}

ebml_element! {
	struct CompSettings {
		const ID = 0x4255;

		value: Vec<u8>
	}
}

ebml_element! {
	struct Encryption {
		const ID = 0x5035;

		value: vint = 0
	}
}

ebml_element! {
	struct EncAlgorithm {
		const ID = 0x47e1;

		value: vint = 0
	}
}

ebml_element! {
	struct EncKeyId {
		const ID = 0x47e1;

		value: Vec<u8>
	}
}

ebml_element! {
	struct EncAesSettings {
		const ID = 0x47e7;

		cipher_mode: AesCipherMode
	}
}

ebml_element! {
	struct AesCipherMode {
		const ID = 0x47e8;

		value: vint
	}
}
