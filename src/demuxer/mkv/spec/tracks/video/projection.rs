use super::*;

ebml_define! {
	#[repr(Unsigned)]
	pub enum Type {
		Rectangular     = 0,
		Equirectangular = 1,
		Cubemap         = 2,
		Mesh            = 3
	}
}

ebml_define! {
	#[allow(dead_code)]
	pub struct Projection {
		#[rename = "type"]
		pub ty: Type @ 0x7671 = Type::Rectangular,
		pub private: Option<Bytes> @ 0x7672,
		pub pose_yaw: Float @ 0x7673 = 0.0,
		pub pose_pitch: Float @ 0x7674 = 0.0,
		pub pose_roll: Float @ 0x7675 = 0.0
	}
}
