#![allow(clippy::unreadable_literal, unreachable_pub)]

use enumflags2::bitflags;

use super::*;
use crate::macros::ebml_define;

pub mod cluster;
pub mod cues;
pub mod seek_head;
pub mod segment_info;
pub mod tracks;

use self::cluster::*;
use self::cues::*;
use self::seek_head::*;
use self::segment_info::*;
use self::tracks::*;

ebml_define! {
	#[allow(dead_code)]
	pub struct Segment {
		pub info: SegmentInfo @ 0x1549a966,
		pub seek_head: Option<Vec<SeekHead>> @ 0x114d9b74,
		pub tracks: Option<Tracks> @ 0x1654ae6b,
		pub cues: Option<Cues> @ 0x1c53bb6b,
		pub clusters: Option<Vec<Cluster>> @ 0x1f43b675
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct MatroskaRoot {
		#[flatten]
		pub root: EbmlRoot,
		pub segments: Option<Vec<Segment>> @ 0x18538067
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum TranslateCodec {
		MatroskaScript = 0,
		DvdMenu        = 1
	}
}
