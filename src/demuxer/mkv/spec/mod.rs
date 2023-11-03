use xx_mpeg_macros::ebml_element;

use super::*;

pub mod seek_head;
pub use seek_head::*;
pub mod segment_info;
pub use segment_info::*;
pub mod tracks;
pub use tracks::*;
pub mod cues;
pub use cues::*;
pub mod cluster;
pub use cluster::*;

ebml_element! {
	struct Segment {
		const ID = 0x18538067;

		info: SegmentInfo,
		seek_head: Vec<SeekHead>,
		tracks: Option<Tracks>,
		cues: Option<Cues>,
		clusters: Vec<Cluster>
	}
}
