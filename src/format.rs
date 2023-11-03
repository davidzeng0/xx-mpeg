use xx_core::{debug, error::*};
use xx_pulse::*;

use super::*;

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MediaType {
	#[default]
	Unknown,
	Video,
	Audio,
	Other
}

#[derive(Default, Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Discard {
	None,
	#[default]
	Default,
	All
}

#[derive(Default)]
pub struct Track {
	pub ty: MediaType,

	pub codec_params: CodecParams,
	pub parse: CodecParse,
	pub parser: Option<CodecParser>,
	pub packet_padding: usize,

	pub id: u64,
	pub time_base: Rational,
	pub start_time: i64,
	pub duration: u64,

	pub discard: Discard
}

#[derive(Default)]
pub struct FormatContext {
	pub tracks: Vec<Track>,
	pub start_time: u64,
	pub duration: u64,
	pub time_base: Rational
}

impl FormatContext {
	fn get_track_by_id(&mut self, id: u64) -> Result<(u32, &mut Track)> {
		let track_index = self
			.tracks
			.iter()
			.position(|track| track.id == id)
			.ok_or_else(|| Error::new(ErrorKind::InvalidData, "Unknown track id"))?;
		Ok((track_index as u32, &mut self.tracks[track_index]))
	}

	pub fn alloc_packet_for(
		&mut self, track_id: u64, size: u64, pool: Option<&Pool>
	) -> Result<Packet> {
		let (index, track) = self.get_track_by_id(track_id)?;
		let mut packet = Packet::alloc(size as usize, track.packet_padding, pool)?;

		packet.track_index = index;
		packet.time_base = track.time_base;

		Ok(packet)
	}
}

pub struct Format {
	demuxer: Demuxer,
	context: FormatContext
}

static DEMUXERS: &[DemuxerClass] = &[&MatroskaClass];

#[async_fn]
impl Format {
	pub async fn open(resource: &Resource) -> Result<Self> {
		let mut reader = Reader::new(resource.create_stream().await?);
		let mut demuxer = None;
		let mut score = 0.0;

		for demuxer_class in DEMUXERS.iter() {
			reader.set_peeking(true).await;
			score = demuxer_class.probe(&mut reader).await?;
			reader.set_peeking(false).await;

			if score > 0.0 {
				demuxer = Some(demuxer_class);

				break;
			}
		}

		let demuxer = match demuxer {
			Some(demuxer) => demuxer,
			None => return Err(Error::new(ErrorKind::Unsupported, "Unknown format"))
		};

		let mut this = Self {
			demuxer: demuxer.create(reader).await?,
			context: FormatContext::default()
		};

		debug!(target: &this, "== Probed format {} with a score of {:.2}%", demuxer.name(), score * 100.0);

		this.demuxer.open(&mut this.context).await?;

		for track in &mut this.context.tracks {
			track.codec_params.ty = track.ty;
			track.codec_params.packet_time_base = track.time_base;
			track.parser = Some(CodecParser::new(&mut track.codec_params)?);
			track.start_time = track.time_base.rescale(
				-(track.codec_params.delay as i64),
				track.codec_params.time_base
			);

			track.duration = track.duration.wrapping_add_signed(track.start_time);

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

	pub fn tracks(&self) -> &Vec<Track> {
		&self.context.tracks
	}

	pub fn tracks_mut(&mut self) -> &mut Vec<Track> {
		&mut self.context.tracks
	}

	pub fn start_time(&self) -> u64 {
		self.context.start_time
	}

	pub fn duration(&self) -> u64 {
		self.context.duration
	}

	pub fn time_base(&self) -> Rational {
		self.context.time_base
	}

	#[inline(always)]
	async fn read_packet_maybe_pool(&mut self, pool: Option<&Pool>) -> Result<Option<Packet>> {
		loop {
			let mut packet = Packet::new();

			match self
				.demuxer
				.read_packet(&mut self.context, &mut packet, pool)
				.await?
			{
				true => (),
				false => return Ok(None)
			};

			let track = &mut self.context.tracks[packet.track_index as usize];

			if track.discard == Discard::All {
				continue;
			}

			if track.parse != CodecParse::None {
				let parser = track.parser.as_mut().unwrap();

				parser.parse(&mut packet)?;
			}

			break Ok(Some(packet));
		}
	}

	pub async fn read_packet(&mut self) -> Result<Option<Packet>> {
		self.read_packet_maybe_pool(None).await
	}

	pub async fn read_packet_with_pool(&mut self, pool: &Pool) -> Result<Option<Packet>> {
		self.read_packet_maybe_pool(Some(pool)).await
	}

	pub async fn seek(&mut self, track_index: u32, timecode: u64) -> Result<()> {
		self.demuxer.seek(track_index, timecode).await
	}
}
