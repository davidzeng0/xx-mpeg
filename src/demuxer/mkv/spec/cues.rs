use super::*;

ebml_element! {
	struct Cues {
		const ID = 0x1c53bb6b;

		points: Vec<Point>
	}
}

ebml_element! {
	struct Point {
		const ID = 0xbb;

		time: Time,
		track_positions: Vec<TrackPositions>
	}
}

ebml_element! {
	struct Time {
		const ID = 0xb3;

		value: vint
	}
}

ebml_element! {
	struct TrackPositions {
		const ID = 0xb7;

		track: TrackId,
		cluster_position: ClusterPosition,
		relative_position: Option<RelativePosition>,
		duration: Option<Duration>,
		block_number: Option<BlockNumber>,
		codec_state: CodecState,
		reference: Option<Reference>
	}
}

ebml_element! {
	struct TrackId {
		const ID = 0xf7;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Track UID cannot be zero"))
		}
	}
}

ebml_element! {
	struct ClusterPosition {
		const ID = 0xf1;

		value: vint
	}
}

ebml_element! {
	struct RelativePosition {
		const ID = 0xf0;

		value: vint
	}
}

ebml_element! {
	struct Duration {
		const ID = 0xb2;

		value: vint
	}
}

ebml_element! {
	struct BlockNumber {
		const ID = 0x5378;

		value: vint
	}
}

ebml_element! {
	struct CodecState {
		const ID = 0xea;

		value: vint
	}
}

ebml_element! {
	struct Reference {
		const ID = 0xdb;

		time: RefTime,
		cluster: RefCluster,
		number: Option<RefNumber>,
		codec_state: Option<RefCodecState>
	}
}

ebml_element! {
	struct RefTime {
		const ID = 0x96;

		value: vint
	}
}

ebml_element! {
	struct RefCluster {
		const ID = 0x97;

		value: vint
	}
}

ebml_element! {
	struct RefNumber {
		const ID = 0x535f;

		value: vint = 1
	}
}

ebml_element! {
	struct RefCodecState {
		const ID = 0xeb;

		value: vint
	}
}
