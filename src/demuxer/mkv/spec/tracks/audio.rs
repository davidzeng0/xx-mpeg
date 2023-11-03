use super::*;

ebml_element! {
	struct Audio {
		const ID = 0xe1;

		sampling_frequency: SamplingFrequency,
		output_sampling_frequency: Option<OutputSamplingFrequency>,
		channels: Channels,
		channel_positions: Option<ChannelPositions>,
		bit_depth: Option<BitDepth>,
		emphasis: Emphasis
	}
}

ebml_element! {
	struct SamplingFrequency {
		const ID = 0xb5;

		value: vfloat = 8000.0
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value > 0.0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Sampling frequency must be positive"))
		}
	}
}

ebml_element! {
	struct OutputSamplingFrequency {
		const ID = 0x78b5;

		value: vfloat
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value > 0.0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Output sampling frequency must be positive"))
		}
	}
}

ebml_element! {
	struct Channels {
		const ID = 0x9f;

		value: vint = 1
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Channel count cannot be zero"))
		}
	}
}

ebml_element! {
	struct ChannelPositions {
		const ID = 0x7d7b;

		value: Vec<u8>
	}
}

ebml_element! {
	struct BitDepth {
		const ID = 0x6264;

		value: vint
	}

	fn post_parse(&mut self) -> Result<()> {
		if self.value != 0 {
			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Bit depth cannot be zero"))
		}
	}
}

ebml_element! {
	struct Emphasis {
		const ID = 0x52f1;

		value: vint
	}
}
