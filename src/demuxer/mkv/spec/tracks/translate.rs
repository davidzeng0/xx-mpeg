use super::*;

ebml_define! {
	#[allow(dead_code)]
	pub struct Translate {
		pub id: Bytes @ 0x66a5,
		pub codec: TranslateCodec @ 0x66bf,
		pub edition_uids: Vec<Unsigned> @ 0x66fc
	}
}
