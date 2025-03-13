use std::result;

use super::av::*;
use super::*;

codec_pair!(CodecId::Opus, Some("libopus"), AV_CODEC_ID_OPUS, Opus);

pub const SAMPLE_RATE: u32 = 48_000;

#[errors]
pub enum OpusError {
	#[display("Invalid opus packet")]
	#[kind = ErrorKind::InvalidData]
	InvalidPacket
}

pub fn get_nb_frames(packet: &[u8]) -> result::Result<u32, OpusError> {
	let [config, rest @ ..] = packet else {
		return Err(OpusError::InvalidPacket);
	};

	#[allow(clippy::unreachable)]
	Ok(match config & 0x3 {
		0 => 1,
		1 | 2 => 2,
		3 => *rest.first().ok_or(OpusError::InvalidPacket)? as u32,
		_ => unreachable!()
	})
}

#[must_use]
pub const fn get_samples_per_frame(config: u8, sample_rate: u32) -> u32 {
	if config & 0x80 != 0 {
		let audio_size = (config >> 3) & 0x3;

		(sample_rate << audio_size) / 400
	} else if config & 0x60 == 0x60 {
		if config & 0x08 != 0 {
			sample_rate / 50
		} else {
			sample_rate / 100
		}
	} else {
		let audio_size = (config >> 3) & 0x3;

		if audio_size == 3 {
			#[allow(clippy::arithmetic_side_effects)]
			(sample_rate * 60 / 1000)
		} else {
			(sample_rate << audio_size) / 100
		}
	}
}

#[allow(clippy::arithmetic_side_effects)]
pub fn get_nb_samples(packet: &[u8], sample_rate: u32) -> result::Result<u32, OpusError> {
	let frames = get_nb_frames(packet)?;
	let samples = frames * get_samples_per_frame(packet[0], sample_rate);

	if samples * 25 > sample_rate * 3 {
		return Err(OpusError::InvalidPacket);
	}

	Ok(samples)
}

pub struct OpusParser;

impl OpusParser {
	pub fn new(
		_: CodecParse, params: &mut CodecParams
	) -> Result<Box<dyn CodecParserImpl + Send + Sync>> {
		params.sample_rate = SAMPLE_RATE;
		params.change_time_base(Rational::inverse(SAMPLE_RATE));

		Ok(Box::new(Self))
	}
}

impl CodecParserImpl for OpusParser {
	fn id(&self) -> CodecId {
		CodecId::Opus
	}

	fn parse(&mut self, packet: &mut Packet) -> Result<()> {
		if let Ok(samples) = get_nb_samples(&packet.data, SAMPLE_RATE) {
			let new_timescale = Rational::inverse(SAMPLE_RATE);

			if packet.timestamp != UNKNOWN_TIMESTAMP {
				packet.timestamp = new_timescale.rescale(packet.timestamp, packet.time_base);
			}

			packet.time_base = new_timescale;
			packet.duration = samples as u64;

			Ok(())
		} else {
			Err(FormatError::InvalidData("Invalid opus packet".into()).into())
		}
	}
}
