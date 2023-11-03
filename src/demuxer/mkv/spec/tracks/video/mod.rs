use super::*;

mod color;
pub use color::*;
mod projection;
pub use projection::*;

ebml_element! {
	struct Video {
		const ID = 0xe0;

		interlaced: Interlaced,
		field_order: FieldOrder,
		stereo_mode: StereoMode,
		alpha_mode: AlphaMode,
		old_stereo_mode: OldStereoMode,
		pixel_width: PixelWidth,
		pixel_height: PixelHeight,
		pixel_crop_bottom: PixelCropBottom,
		pixel_crop_top: PixelCropTop,
		pixel_crop_left: PixelCropLeft,
		pixel_crop_right: PixelCropRight,
		display_width: Option<DisplayWidth>,
		display_height: Option<DisplayHeight>,
		display_unit: DisplayUnit,
		aspect_ratio_type: Option<AspectRatioType>,
		uncompressed_four_cc: Option<UncompressedFourCC>,
		gamma_value: Option<GammaValue>,
		framerate: Option<Framerate>,
		color: Option<Color>
	}
}

ebml_element! {
	struct Interlaced {
		const ID = 0x9a;

		value: vint
	}
}

ebml_element! {
	struct FieldOrder {
		const ID = 0x9d;

		value: vint = 2
	}
}

ebml_element! {
	struct StereoMode {
		const ID = 0x53b8;

		value: vint
	}
}

ebml_element! {
	struct AlphaMode {
		const ID = 0x53c0;

		value: vint
	}
}

ebml_element! {
	struct OldStereoMode {
		const ID = 0x53b9;

		value: vint
	}
}

ebml_element! {
	struct PixelWidth {
		const ID = 0xb0;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Pixel width cannot be zero"))
		}
	}
}

ebml_element! {
	struct PixelHeight {
		const ID = 0xba;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Pixel height cannot be zero"))
		}
	}
}

ebml_element! {
	struct PixelCropBottom {
		const ID = 0x54aa;

		value: vint
	}
}

ebml_element! {
	struct PixelCropTop {
		const ID = 0x54bb;

		value: vint
	}
}

ebml_element! {
	struct PixelCropLeft {
		const ID = 0x54cc;

		value: vint
	}
}

ebml_element! {
	struct PixelCropRight {
		const ID = 0x54dd;

		value: vint
	}
}

ebml_element! {
	struct DisplayWidth {
		const ID = 0x54b0;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Display width cannot be zero"))
		}
	}
}

ebml_element! {
	struct DisplayHeight {
		const ID = 0x54ba;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Display height cannot be zero"))
		}
	}
}

ebml_element! {
	struct DisplayUnit {
		const ID = 0x54b2;

		value: vint
	}
}

ebml_element! {
	struct AspectRatioType {
		const ID = 0x54b3;

		value: vint
	}
}

ebml_element! {
	struct UncompressedFourCC {
		const ID = 0x2eb524;

		value: Vec<u8>
	}
}

ebml_element! {
	struct GammaValue {
		const ID = 0x2fb523;

		value: vfloat
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value > 0.0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Gamma value must be positive"))
		}
	}
}

ebml_element! {
	struct Framerate {
		const ID = 0x2383e3;

		value: vfloat
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value > 0.0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Framerate must be positive"))
		}
	}
}
