#![allow(clippy::module_name_repetitions)]

use std::ops::{Deref, DerefMut};

use xx_core::debug;

use super::*;

#[derive(Default)]
pub struct Track {
	pub ty: MediaType,

	pub codec_params: CodecParams,
	pub parse: CodecParse,
	pub parser: Option<CodecParser>,

	pub id: u64,
	pub time_base: Rational,
	pub start_time: i64,
	pub duration: u64,

	pub discard: Discard
}

#[derive(Default)]
pub struct FormatData {
	pub tracks: Vec<Track>,
	pub start_time: u64,
	pub duration: u64,
	pub time_base: Rational
}

impl FormatData {
	fn get_track_by_id(&mut self, id: u64) -> Result<(u32, &mut Track)> {
		let track_index = self
			.tracks
			.iter()
			.position(|track| track.id == id)
			.ok_or(FormatError::TrackNotFound)?;

		#[allow(clippy::cast_possible_truncation)]
		Ok((track_index as u32, &mut self.tracks[track_index]))
	}

	pub fn get_packet_fields_for(&mut self, packet: &mut Packet, track_id: u64) -> Result<()> {
		let (index, track) = self.get_track_by_id(track_id)?;

		packet.track_index = index;
		packet.time_base = track.time_base;

		Ok(())
	}
}

pub struct Format {
	demuxer: Demuxer,
	data: FormatData
}

static DEMUXERS: &[DemuxerClass<'static>] = &[&mkv::MatroskaClass];

#[asynchronous]
impl Format {
	pub async fn open(resource: &Resource) -> Result<Self> {
		let mut reader = Reader::new(resource.create_stream().await?);
		let mut demuxer = None;
		let mut score = 0.0;

		for demuxer_class in DEMUXERS {
			reader.set_peeking(true).await;

			score = match demuxer_class.probe(&mut reader).await {
				Ok(score) => score,
				Err(err) if err == ReaderError::PeekBufferExhausted => 0.0,
				Err(err) => return Err(err)
			};

			reader.set_peeking(false).await;

			if score > 0.0 {
				demuxer = Some(demuxer_class);

				break;
			}
		}

		let demuxer = match demuxer {
			Some(demuxer) => demuxer,
			None => return Err(FormatError::UnknownFormat.into())
		};

		let mut this = Self {
			demuxer: demuxer.create(reader).await?,
			data: FormatData::default()
		};

		debug!(target: &this, "== Probed format {} with a score of {:.2}%", demuxer.name(), score * 100.0);

		this.demuxer.open(&mut this.data).await?;

		for track in &mut this.data.tracks {
			track.codec_params.ty = track.ty;
			track.codec_params.packet_time_base = track.time_base;

			track.start_time = track.time_base.rescale(
				#[allow(clippy::arithmetic_side_effects)]
				-(track.codec_params.delay as i64),
				track.codec_params.time_base
			);

			track.duration = track
				.duration
				.checked_add_signed(track.start_time)
				.ok_or(Core::Overflow)?;

			#[allow(clippy::single_match)]
			match track.codec_params.ty {
				MediaType::Audio => {
					track
						.codec_params
						.change_time_base(Rational::inverse(track.codec_params.sample_rate));
				}

				_ => ()
			}
		}

		Ok(this)
	}

	#[inline]
	pub async fn read_packet(&mut self) -> Result<Option<Packet>> {
		loop {
			let mut packet = Packet::new();

			match self
				.demuxer
				.read_packet(&mut self.data, &mut packet)
				.await?
			{
				true => (),
				false => return Ok(None)
			};

			let track = &mut self.data.tracks[packet.track_index as usize];

			if track.discard == Discard::All {
				continue;
			}

			if track.parse != CodecParse::None {
				let parser = match &mut track.parser {
					Some(parser) => parser,
					None => track
						.parser
						.insert(CodecParser::new(&mut track.codec_params)?)
				};

				parser.parse(&mut packet)?;
			}

			break Ok(Some(packet));
		}
	}

	pub async fn seek(&mut self, track_index: u32, timecode: u64) -> Result<()> {
		self.demuxer.seek(track_index, timecode).await
	}
}

impl Deref for Format {
	type Target = FormatData;

	fn deref(&self) -> &FormatData {
		&self.data
	}
}

impl DerefMut for Format {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.data
	}
}
