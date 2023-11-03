use super::*;

ebml_element! {
	struct Projection {
		const ID = 0x7670;

		ty: Type
	}
}

ebml_element! {
	struct Type {
		const ID = 0x7671;

		value: vint
	}
}

ebml_element! {
	struct Private {
		const ID = 0x7672;

		value: Vec<u8>
	}
}

ebml_element! {
	struct PoseYaw {
		const ID = 0x7673;

		value: vfloat
	}
}

ebml_element! {
	struct PosePitch {
		const ID = 0x7674;

		value: vfloat
	}
}

ebml_element! {
	struct PoseRoll {
		const ID = 0x7675;

		value: vfloat
	}
}
