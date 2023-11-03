use ::opus::packet::get_nb_samples;

use super::{av::*, *};

codec_pair!(CodecId::Opus, Some("libopus"), AV_CODEC_ID_OPUS, Opus);

const OPUS_SAMPLE_RATE: u32 = 48_000;

pub struct OpusParser;

impl OpusParser {
	pub fn new(params: &mut CodecParams) -> Result<Box<dyn CodecParserTrait>> {
		params.sample_rate = OPUS_SAMPLE_RATE;
		params.packet_time_base = Rational::inverse(OPUS_SAMPLE_RATE);

		Ok(Box::new(Self))
	}
}

impl CodecParserTrait for OpusParser {
	fn id(&self) -> CodecId {
		CodecId::Opus
	}

	fn parse(&self, packet: &mut Packet) -> Result<()> {
		if let Ok(samples) = get_nb_samples(packet.data(), OPUS_SAMPLE_RATE) {
			let new_timescale = Rational::inverse(OPUS_SAMPLE_RATE);

			if packet.timestamp != UNKNOWN_TIMESTAMP {
				packet.timestamp = new_timescale.rescale(packet.timestamp, packet.time_base);
			}

			packet.time_base = new_timescale;
			packet.duration = samples as u64;

			Ok(())
		} else {
			Err(Error::new(ErrorKind::InvalidData, "Invalid opus packet"))
		}
	}
}
