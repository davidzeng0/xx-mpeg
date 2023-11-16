use std::marker::PhantomData;

use xx_core::{error::*, trace};
use xx_pulse::*;

use super::*;

mod ebml;
use ebml::{spec::*, *};

mod spec;
use num_traits::FromPrimitive;
use spec::*;

use super::{CodecId, MediaType};

#[derive(Default)]
struct Block {
	track_id: u64,
	timecode: u64,
	flags: u8,
	size: u64
}

struct Matroska {
	reader: Reader,

	stack: [MasterElement; 16],
	level: usize,

	tracks: Option<Tracks>,
	duration: f64,

	timecode_scale: Rational,

	seek_head: Option<SeekHead>,
	cues: Option<Cues>,

	segment_offset: u64,
	cluster_timecode: u64,
	block: Option<Block>
}

#[async_fn]
impl Matroska {
	fn new(reader: Reader) -> Self {
		Self {
			reader,

			stack: [MasterElement::ROOT; 16],
			level: 0,

			tracks: None,
			duration: 0.0,

			timecode_scale: Rational::default(),

			seek_head: None,
			cues: None,

			segment_offset: 0,
			cluster_timecode: 0,
			block: None
		}
	}

	fn stack_push(&mut self, master: MasterElement) {
		self.stack[self.level] = master;
		self.level += 1;
	}

	fn stack_pop(&mut self) {
		self.level = self.level.checked_sub(1).unwrap();
	}

	async fn post_read(&mut self, element: Element) -> Result<()> {
		if element.end != UNKNOWN_END {
			let pos = self.reader().position();

			if pos < element.end {
				let indents = self.level * 2;

				trace!(target: self, "== {: <indents$}Remaining {} bytes in element", "", element.end - pos);
				skip_element(self.reader(), &element).await?;
			} else if pos > element.end {
				return Err(Error::new(
					ErrorKind::InvalidData,
					"Read element out of bounds"
				));
			}
		}

		Ok(())
	}

	fn handle_header(&self, header: &Header) -> Result<()> {
		if header.version.value != 1 {
			return Err(Error::new(
				ErrorKind::Unsupported,
				format!("Unsupported EBML version {}", header.version.value)
			));
		}

		if header.max_id_length.value > 8 {
			return Err(Error::new(
				ErrorKind::Unsupported,
				format!("Unsupported EBML max id length")
			));
		}

		if header.max_size_length.value > 8 {
			return Err(Error::new(
				ErrorKind::Unsupported,
				format!("Unsupported EBML max size length")
			));
		}

		match &header.doc_type.value as &str {
			"matroska" | "webm" => (),
			ty => {
				return Err(Error::new(
					ErrorKind::Unsupported,
					format!("Unknown doc type {}", ty)
				))
			}
		}

		if header.doc_type_version.value > 4 {
			return Err(Error::new(
				ErrorKind::Unsupported,
				format!(
					"Unimplemented doc type version {}",
					header.doc_type_version.value
				)
			));
		}

		Ok(())
	}

	async fn read_root(&mut self) -> Result<()> {
		if self.level == 0 {
			self.stack_push(MasterElement::ROOT);
		}

		let mut stop = false;

		while !stop {
			let master = &self.stack[self.level - 1];

			let element = match next_element(&mut self.reader, &master.element).await? {
				Some(element) => element,
				None => {
					self.post_read(master.element.clone()).await?;
					self.stack_pop();

					if self.level == 0 {
						break;
					}

					continue;
				}
			};

			if element.end == UNKNOWN_END {
				return Err(Error::new(
					ErrorKind::Unsupported,
					"Elements with unknown size are not currently supported"
				));
			}

			if !master.is_child(element.id) {
				let indents = self.level * 2;

				trace!(target: self, "== {: <indents$}Skipping unknown element with id {} of size {}", "", element.id, element.size);
				skip_element(self.reader(), &element).await?;

				continue;
			}

			match element.id {
				Header::ID => {
					self.pre_parse::<Header>(&element, PhantomData)?;

					let header = Header::parse(self, &element).await?;

					self.handle_header(&header)?;
				}

				Segment::ID => {
					self.pre_parse::<Segment>(&element, PhantomData)?;
					self.segment_offset = element.offset;
					self.stack_push(MasterElement { element, children: Segment::CHILDREN });

					continue;
				}

				SegmentInfo::ID => {
					self.pre_parse::<SegmentInfo>(&element, PhantomData)?;

					let info = SegmentInfo::parse(self, &element).await?;

					if let Some(duration) = info.duration {
						self.duration = duration.value;
					}

					let mut timescale = Rational::new(
						info.timestamp_scale
							.value
							.try_into()
							.map_err(Error::map_as_invalid_data)?,
						1_000_000_000
					);

					timescale.reduce();

					self.timecode_scale = timescale;
				}

				SeekHead::ID => {
					self.pre_parse::<SeekHead>(&element, PhantomData)?;
					self.seek_head = Some(SeekHead::parse(self, &element).await?);
				}

				Tracks::ID => {
					self.pre_parse::<Tracks>(&element, PhantomData)?;
					self.tracks = Some(Tracks::parse(self, &element).await?);

					stop = true;
				}

				Cues::ID => {
					self.pre_parse::<Cues>(&element, PhantomData)?;
					self.cues = Some(Cues::parse(self, &element).await?);
				}

				Cluster::ID => {
					self.pre_parse::<Cluster>(&element, PhantomData)?;
					self.stack_push(MasterElement { element, children: Cluster::CHILDREN });

					continue;
				}

				Timestamp::ID => {
					self.pre_parse::<Timestamp>(&element, PhantomData)?;
					self.cluster_timecode = Timestamp::parse(self, &element).await?.value;
				}

				SimpleBlock::ID => {
					self.pre_parse::<SimpleBlock>(&element, PhantomData)?;

					let mut block = Block::default();

					block.track_id = read_vint(self.reader(), VintKind::Unsigned).await?;
					block.timecode =
						self.reader().read_u16_be().await? as u64 + self.cluster_timecode;
					block.flags = self.reader().read_u8().await?;

					let remaining = element.remaining(self.reader.position());

					if remaining < 0 {
						return Err(Error::new(ErrorKind::InvalidData, "Block read overflow"));
					}

					block.size = remaining as u64;

					self.block = Some(block);

					break;
				}

				_ => ()
			}

			self.post_read(element).await?;
		}

		Ok(())
	}
}

#[async_trait_impl]
impl Parser for Matroska {
	fn reader(&mut self) -> &mut Reader {
		&mut self.reader
	}

	async fn read_children<F: FnMut(&mut Self, &Element) -> Result<()>>(
		&mut self, master: &MasterElement, mut handle_child: F
	) -> Result<()> {
		self.level += 1;

		read_children(self, master, |parser, element| {
			handle_child(parser, element)?;

			parser.post_read(element.clone()).await
		})
		.await?;

		self.level -= 1;
		self.post_read(master.element.clone()).await?;

		Ok(())
	}

	#[inline(always)]
	fn pre_parse<E: Parse>(&self, element: &Element, _: PhantomData<E>) -> Result<()> {
		let indents = (self.level - 1) * 2;

		trace!(
			target: self,
			"== {: <indents$}{: <20} : {} @ {}", "",
			E::NAME,
			element.size,
			element.offset
		);

		Ok(())
	}
}

fn get_track_type(ty: u64) -> Result<super::MediaType> {
	let ty = match tracks::TrackType::from_u64(ty) {
		Some(ty) => ty,
		None => return Err(Error::new(ErrorKind::InvalidData, "Unknown track type"))
	};

	Ok(match ty {
		tracks::TrackType::Video => MediaType::Video,
		tracks::TrackType::Audio => MediaType::Audio,
		_ => MediaType::Other
	})
}

fn get_track_codec(id: &str) -> CodecId {
	match id {
		"A_OPUS" => CodecId::Opus,
		"A_VORBIS" => CodecId::Vorbis,
		"A_AAC" => CodecId::Aac,
		"A_FLAC" => CodecId::Flac,
		_ => CodecId::Unknown
	}
}

fn get_track_codec_params(track: &tracks::Track) -> CodecParams {
	let mut params = CodecParams::default();

	params.id = get_track_codec(&track.codec_id.value as &str);

	if let Some(private) = &track.codec_private {
		params.config = private.value.clone();
	}

	if let Some(audio) = &track.audio {
		params.sample_rate = audio.sampling_frequency.value as u32;
		params.channels = audio.channels.value as u16;

		if let Some(output_sr) = &audio.output_sampling_frequency {
			params.sample_rate = output_sr.value as u32;
		}

		if let Some(bit_depth) = &audio.bit_depth {
			params.bit_depth = bit_depth.value as u16;
		}
	}

	params.time_base = Rational::nanos();
	params.delay = track.codec_delay.value as u32;
	params.seek_preroll = track.seek_preroll.value as u32;
	params
}

#[async_trait_impl]
impl DemuxerTrait for Matroska {
	async fn open(&mut self, context: &mut FormatContext) -> Result<()> {
		self.read_root().await?;

		let tracks = match self.tracks.take() {
			Some(tracks) => tracks,
			None => {
				return Err(Error::new(
					ErrorKind::InvalidData,
					"Could not find any tracks"
				))
			}
		};

		context.duration = self.duration as u64;
		context.time_base = self.timecode_scale;

		for track in &tracks.tracks {
			let params = get_track_codec_params(track);
			let parse = if params.id != CodecId::Aac {
				CodecParse::Header
			} else {
				CodecParse::default()
			};

			context.tracks.push(super::Track {
				ty: get_track_type(track.ty.value)?,
				codec_params: params,
				id: track.number.value,
				duration: self.duration as u64,
				time_base: self.timecode_scale,
				parse,
				..std::default::Default::default()
			});
		}

		Ok(())
	}

	async fn seek(&mut self, track: u32, timecode: u64) -> Result<()> {
		Ok(())
	}

	async fn read_packet(
		&mut self, context: &mut FormatContext, packet: &mut Packet, pool: Option<&Pool>
	) -> Result<bool> {
		if self.block.is_none() {
			self.read_root().await?;
		}

		let block = match self.block.take() {
			Some(block) => block,
			None => return Ok(false)
		};

		*packet = context.alloc_packet_for(block.track_id, block.size, pool)?;

		packet.timestamp = block.timecode as i64;

		if block.flags & 0x80 != 0 {
			packet.flags |= PacketFlag::Keyframe;
		}

		self.reader.read(packet.data_mut()).await?;

		Ok(true)
	}
}

struct Probe<'a> {
	reader: &'a mut Reader
}

#[async_trait_impl]
impl<'a> Parser for Probe<'a> {
	fn reader(&mut self) -> &mut Reader {
		self.reader
	}
}

pub struct MatroskaClass;

#[async_trait_impl]
impl DemuxerClassTrait for MatroskaClass {
	fn name(&self) -> &'static str {
		"Matroska"
	}

	async fn create(&self, reader: Reader) -> Result<Demuxer> {
		Ok(Box::new(Matroska::new(reader)))
	}

	async fn probe(&self, reader: &mut Reader) -> Result<f32> {
		let master = MasterElement::ROOT;
		let mut probe = Probe { reader };
		let mut score = 0.0;

		for _ in 0..4 {
			let element = match next_element(probe.reader, &master.element).await? {
				Some(element) => element,
				None => break
			};

			match element.id {
				Void::ID | Crc32::ID | Segment::ID => score = 0.25,
				Header::ID => {
					if Header::parse(&mut probe, &element).await.is_ok() {
						score = 1.0;
					} else {
						score = 0.0;
					}

					break;
				}
				_ => break
			}
		}

		Ok(score)
	}
}
