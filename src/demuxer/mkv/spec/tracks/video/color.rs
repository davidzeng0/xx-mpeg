use super::*;

ebml_define! {
	#[repr(Unsigned)]
	pub enum MatrixCoefficients {
		Identity         = 0,
		IturBt709        = 1,
		Unspecified      = 2,
		Reserved         = 3,
		UsFcc73_682      = 4,
		IturBt470Bg      = 5,
		Smpte170M        = 6,
		Smpte240M        = 7,
		YCoCg            = 8,
		Bt2020NCL        = 9,
		Bt2020CL         = 10,
		SmpteSt2085      = 11,
		ChromaDerivedNCL = 12,
		ChromaDerivedCL  = 13,
		IturBt2100_0     = 14
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum ChromaSiting {
		Unspecified = 0,

		/// Left collated if horizontal, otherwise top
		Collated    = 1,
		Half        = 2
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum Range {
		Unspecified = 0,
		Broadcast   = 1,
		Full        = 2,
		Defined     = 3
	}
}

ebml_define! {
	#[repr(Unsigned)]
	pub enum TransferCharacteristics {
		Reserved            = 0,
		IturBt709           = 1,
		Unspecified         = 2,
		Reserved2           = 3,
		Gamma22CurveBt470M  = 4,
		Gamma28CurveBt470Bg = 5,
		Smpte170M           = 6,
		Smpte240M           = 7,
		Linear              = 8,
		Log                 = 9,
		LogSqrt             = 10,
		Iec61966_2_4        = 11,
		IturBt1361Extended  = 12,
		Iec61966_2_1        = 13,
		IturBt2020_10Bit    = 14,
		IturBt2020_12Bit    = 15,
		IturBt2100PerceptialQuantization = 16,
		SmpteSt428_1        = 17,
		AribStdB67HLG       = 18
	}
}

ebml_define! {
	#[allow(non_camel_case_types)]
	#[repr(Unsigned)]
	pub enum Primaries {
		Reserved                = 0,
		IturBt709               = 1,
		Unspecified             = 2,
		Reserved2               = 3,
		IturBt470M              = 4,
		IturBt470BG_Bt601_625   = 5,
		IturBt601_525_Smpte170M = 6,
		Smpte240M               = 7,
		Film                    = 8,
		IturBt2020              = 9,
		SmpteSt428_1            = 10,
		SmpteRp432_2            = 11,
		SmpteEg432_2            = 12,
		EbuTech3213E_JEDECP22   = 22
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Color {
		pub matrix_coefficients: MatrixCoefficients @ 0x55b1 = MatrixCoefficients::Unspecified,
		pub bits_per_channel: Unsigned @ 0x55b2 = 0,
		pub chroma_subsampling_horz: Option<Unsigned> @ 0x55b3,
		pub chroma_subsampling_vert: Option<Unsigned> @ 0x55b4,
		pub cb_subsampling_horz: Option<Unsigned> @ 0x55b5,
		pub cb_subsampling_vert: Option<Unsigned> @ 0x55b6,
		pub chroma_siting_horz: ChromaSiting @ 0x55b7 = ChromaSiting::Unspecified,
		pub chroma_siting_vert: ChromaSiting @ 0x55b8 = ChromaSiting::Unspecified,
		pub range: Range @ 0x55b9 = Range::Unspecified,
		pub transfer_characteristics: TransferCharacteristics @ 0x55ba = TransferCharacteristics::Unspecified,
		pub primaries: Primaries @ 0x55bb = Primaries::Unspecified,
		pub max_cll: Option<Unsigned> @ 0x55bc,
		pub max_fall: Option<Unsigned> @ 0x55bd
	}
}
