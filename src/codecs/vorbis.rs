use super::{av::*, *};

codec_pair!(CodecId::Vorbis, None, AV_CODEC_ID_VORBIS, Vorbis);

pub struct VorbisParser;

impl VorbisParser {
	pub fn new(_: &mut CodecParams) -> Result<Box<dyn CodecParserTrait>> {
		Ok(Box::new(Self))
	}
}

impl CodecParserTrait for VorbisParser {
	fn id(&self) -> CodecId {
		CodecId::Vorbis
	}

	fn parse(&self, _: &mut Packet) -> Result<()> {
		Ok(())
	}
}
