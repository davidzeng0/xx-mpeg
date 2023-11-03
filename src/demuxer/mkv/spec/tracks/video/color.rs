use super::*;

ebml_element! {
	struct Color {
		const ID = 0x55b0;

		matrix_coefficients: MatrixCoefficients,
		bits_per_channel: BitsPerChannel,
		chroma_subsampling_horz: Option<ChromaSubsamplingHorz>,
		chroma_subsampling_vert: Option<ChromaSubsamplingVert>,
		cb_subsampling_horz: Option<CbSubsamplingHorz>,
		cb_subsampling_vert: Option<CbSubsamplingVert>,
		chroma_siting_horz: ChromaSitingHorz,
		chroma_siting_vert: ChromaSitingVert,
		range: Range,
		transfer_characteristics: TransferCharacteristics,
		primaries: Primaries,
		max_cll: Option<MaxCLL>,
		max_fall: Option<MaxFALL>
	}
}

ebml_element! {
	struct MatrixCoefficients {
		const ID = 0x55b1;

		value: vint = 2
	}
}

ebml_element! {
	struct BitsPerChannel {
		const ID = 0x55b2;

		value: vint
	}
}

ebml_element! {
	struct ChromaSubsamplingHorz {
		const ID = 0x55b3;

		value: vint
	}
}

ebml_element! {
	struct ChromaSubsamplingVert {
		const ID = 0x55b4;

		value: vint
	}
}

ebml_element! {
	struct CbSubsamplingHorz {
		const ID = 0x55b5;

		value: vint
	}
}

ebml_element! {
	struct CbSubsamplingVert {
		const ID = 0x55b6;

		value: vint = 2
	}
}

ebml_element! {
	struct ChromaSitingHorz {
		const ID = 0x55b7;

		value: vint
	}
}

ebml_element! {
	struct ChromaSitingVert {
		const ID = 0x55b8;

		value: vint
	}
}

ebml_element! {
	struct Range {
		const ID = 0x55b9;

		value: vint
	}
}

ebml_element! {
	struct TransferCharacteristics {
		const ID = 0x55ba;

		value: vint = 2
	}
}

ebml_element! {
	struct Primaries {
		const ID = 0x55bb;

		value: vint = 2
	}
}

ebml_element! {
	struct MaxCLL {
		const ID = 0x55bc;

		value: vint
	}
}

ebml_element! {
	struct MaxFALL {
		const ID = 0x55bd;

		value: vint
	}
}
