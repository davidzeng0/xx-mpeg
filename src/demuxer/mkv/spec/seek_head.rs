use super::*;

ebml_define! {
	#[allow(dead_code)]
	pub struct SeekHead {
		pub seek: Vec<Seek> @ 0x4dbb
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Seek {
		pub id: VIntId @ 0x53ab,
		pub position: Unsigned @ 0x53ac
	}
}
