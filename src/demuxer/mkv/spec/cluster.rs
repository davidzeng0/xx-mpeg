use super::*;

ebml_define! {
	#[allow(dead_code)]
	pub struct Cluster {
		pub timestamp: Unsigned @ 0xe7,
		pub silent_tracks: Option<SilentTracks> @ 0x5854,
		pub position: Option<Unsigned> @ 0xa7,
		pub prev_size: Option<Unsigned> @ 0xab,
		pub simple_blocks: Option<Vec<Block>> @ 0xa3,
		pub block_groups: Option<Vec<BlockGroup>> @ 0xa0
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct SilentTracks {
		pub numbers: Option<Vec<Unsigned>> @ 0x58d7
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct BlockHeader {
		pub track_id: VInt,
		pub timecode: u16,
		pub flags: u8,
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Block {
		pub header: BlockHeader,
		pub data: Bytes
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct BlockGroup {
		pub block: Block @ 0xa1,
		pub block_virtual: Option<Bytes> @ 0xa2,
		pub additions: Option<BlockAdditions> @ 0x75a1,
		pub duration: Option<Unsigned> @ 0x9b,
		pub reference_priority: Unsigned @ 0xfa = 0,
		pub reference_block: Option<Vec<Signed>> @ 0xfb,
		pub reference_virtual: Option<Signed> @ 0xfd,
		pub codec_state: Option<Bytes> @ 0xa4,
		pub discard_padding: Option<Signed> @ 0x75a2
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct BlockAdditions {
		pub more: Vec<More> @ 0xa6
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct More {
		pub data: Bytes @ 0xa5,
		pub id: NonZeroUnsigned @ 0xee = 1
	}
}
