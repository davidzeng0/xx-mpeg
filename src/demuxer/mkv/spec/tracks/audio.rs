use super::*;

ebml_define! {
	#[repr(Unsigned)]
	pub enum Emphasis {
		None            = 0,
		Cd              = 1,
		Reserved        = 2,
		CCITJ17         = 3,
		Fm50            = 4,
		Fm75            = 5,
		PhonoRIAA       = 10,
		PhonoIECN78     = 11,
		PhonoTelDec     = 12,
		PhonoEmi        = 13,
		PhonoColumbiaLP = 14,
		PhonoLondon     = 15,
		PhonoNARTB      = 16
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Audio {
		pub sampling_frequency: PositiveFloat @ 0xb5 = 8_000.0,
		pub output_sampling_frequency: Option<PositiveFloat> @ 0x78b5,
		pub channels: NonZeroUnsigned @ 0x9f = 1,
		pub channel_positions: Option<Bytes> @ 0x7d7b,
		pub bit_depth: Option<NonZeroUnsigned> @ 0x6264,
		pub emphasis: Emphasis @ 0x52f1 = Emphasis::None
	}
}
