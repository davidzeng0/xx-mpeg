use super::*;

ebml_define! {
	pub struct Cues {
		pub points: Vec<Point> @ 0xbb
	}
}

ebml_define! {
	pub struct Point {
		pub time: Unsigned @ 0xb3,
		pub track_positions: Vec<TrackPositions> @ 0xb7
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct TrackPositions {
		pub track: NonZeroUnsigned @ 0xf7,
		pub cluster_position: Unsigned @ 0xf1,
		pub relative_position: Option<Unsigned> @ 0xf0,
		pub duration: Option<Unsigned> @ 0xb2,
		pub block_number: Option<NonZeroUnsigned> @ 0x5378,
		pub codec_state: Unsigned @ 0xea = 0,
		pub reference: Option<Vec<Reference>> @ 0xdb
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Reference {
		pub time: Unsigned @ 0x96,
		pub cluster: Unsigned @ 0x97,
		pub number: Unsigned @ 0x535f = 1,
		pub codec_state: Unsigned @ 0xeb = 0
	}
}
