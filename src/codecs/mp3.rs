use super::{av::*, *};

codec_pair!(CodecId::Mp3, None, AV_CODEC_ID_MP3, Mp3);

pub struct Mp3Parser;

impl Mp3Parser {
	pub fn new(_: &mut CodecParams) -> Result<Box<dyn CodecParserTrait>> {
		Ok(Box::new(Self))
	}
}

impl CodecParserTrait for Mp3Parser {
	fn id(&self) -> CodecId {
		CodecId::Mp3
	}

	fn parse(&self, _: &mut Packet) -> Result<()> {
		Ok(())
	}
}
