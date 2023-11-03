use super::{av::*, *};

codec_pair!(CodecId::Flac, None, AV_CODEC_ID_FLAC, Flac);

pub struct FlacParser;

impl FlacParser {
	pub fn new(_: &mut CodecParams) -> Result<Box<dyn CodecParserTrait>> {
		Ok(Box::new(Self))
	}
}

impl CodecParserTrait for FlacParser {
	fn id(&self) -> CodecId {
		CodecId::Flac
	}

	fn parse(&self, _: &mut Packet) -> Result<()> {
		Ok(())
	}
}
