use std::result;

use bitreader::{BitReader, BitReaderError};

use super::av::*;
use super::*;

codec_pair!(CodecId::Aac, None, AV_CODEC_ID_AAC, Aac);

const AOT_ESCAPE: u8 = 0x1f;

const SAMPLE_RATE_TABLE: &[u32] = &[
	96000, 88200, 64000, 48000, 44100, 32000, 24000, 22050, 16000, 12000, 11025, 8000, 7350, 0, 0,
	57600, 51200, 40000, 38400, 34150, 28800, 25600, 20000, 19200, 17075, 14400, 12800, 9600, 0, 0,
	0, 0
];

const CHANNEL_TABLE: &[u16] = &[0, 1, 2, 3, 4, 5, 6, 8, 0, 0, 0, 7, 8, 0, 8, 0];

#[allow(unreachable_pub)]
mod internal {
	use super::*;

	parser_pair!(CodecId::Aac, AV_CODEC_ID_AAC, Aac);
}

pub struct AacParser;

impl AacParser {
	fn parse_config(params: &mut CodecParams) -> result::Result<(), BitReaderError> {
		let mut bits = BitReader::new(&params.config);
		let mut audio_object_type = bits.read_u8(5)?;

		#[allow(clippy::arithmetic_side_effects)]
		if audio_object_type == AOT_ESCAPE {
			audio_object_type = 0x20 + bits.read_u8(6)?;
		}

		let mut sample_rate = bits.read_u8(4)? as u32;

		if sample_rate == 0xf {
			sample_rate = bits.read_u32(24)?;
		} else {
			sample_rate = SAMPLE_RATE_TABLE[sample_rate as usize];
		}

		let channels = bits.read_u8(4)?;
		let _ = audio_object_type;

		params.ch_layout = ChannelLayout::Unspec(CHANNEL_TABLE[channels as usize]);
		params.sample_rate = sample_rate;

		Ok(())
	}

	pub fn new(
		parse: CodecParse, params: &mut CodecParams
	) -> Result<Box<dyn CodecParserImpl + Send + Sync>> {
		Self::parse_config(params)
			.map_err(|_| FormatError::InvalidData("Invalid aac config".into()))?;

		internal::AacParser::new(parse, params)
	}
}
