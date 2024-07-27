use super::*;

ebml_define! {
	#[allow(dead_code)]
	pub struct ContentEncodings {
		pub encodings: Vec<Encoding> @ 0x6240
	}
}

ebml_define! {
	#[bitflags]
	#[repr(u64)]
	pub enum Scope {
		Block   = 1,
		Private = 2,
		Next    = 4
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum EncodingType {
		Compression = 0,
		Encryption  = 1
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Encoding {
		pub order: Unsigned @ 0x5031,
		pub scope: BitFlags<Scope> @ 0x5032 = Scope::Block,
		#[rename = "type"]
		pub ty: EncodingType @ 0x5033 = EncodingType::Compression,
		pub compression: Option<Compression> @ 0x5034,
		pub encryption: Option<Encryption> @ 0x5035
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum CompAlgo {
		Zlib            = 0,
		Bzlib           = 1,
		Lzo1x           = 2,
		HeaderStripping = 3
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Compression {
		pub algorithm: CompAlgo @ 0x4254 = CompAlgo::Zlib,
		pub settings: Option<Bytes> @ 0x4255
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum EncAlgo {
		Unencrypted = 0,
		Des         = 1,
		ThreeDes    = 2,
		Twofish     = 3,
		Blowflish   = 4,
		Aes         = 5
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Encryption {
		pub algo: EncAlgo @ 0x47e1 = EncAlgo::Unencrypted,
		pub key_id: Option<Bytes> @ 0x47e2,
		pub aes_settings: Option<AesSettings> @ 0x47e7
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum AesCipherMode {
		Ctr = 1,
		Cbc = 2
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct AesSettings {
		cipher_mode: AesCipherMode @ 0x47e8
	}
}
