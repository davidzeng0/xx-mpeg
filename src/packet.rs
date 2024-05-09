use super::*;

#[allow(clippy::partial_pub_fields)]
pub struct Packet {
	pub data: Vec<u8>,
	pub time_base: Rational,
	pub duration: u64,
	pub timestamp: i64,
	pub track_index: u32,
	pub flags: BitFlags<PacketFlag>
}

impl Packet {
	#[must_use]
	pub fn new() -> Self {
		Self::default()
	}
}

impl Default for Packet {
	fn default() -> Self {
		Self {
			data: Vec::new(),
			time_base: Rational::default(),
			duration: 0,
			timestamp: UNKNOWN_TIMESTAMP,
			track_index: 0,
			flags: Default::default()
		}
	}
}
