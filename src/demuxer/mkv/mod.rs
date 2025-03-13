use std::io::SeekFrom;
use std::mem::size_of;
use std::ops::{Deref, DerefMut};

use num_traits::FromPrimitive;
use xx_core::coroutines::ops::{AsyncFnMut, AsyncFnMutExt};
use xx_core::error::*;
use xx_core::{trace, warn};

use super::*;

mod ebml;
mod spec;

use self::ebml::parse::*;
use self::ebml::spec::*;
use self::ebml::*;
use self::spec::cluster::*;
use self::spec::cues::*;
use self::spec::seek_head::*;
use self::spec::segment_info::*;
use self::spec::tracks::{TrackType, Tracks};
use self::spec::*;

#[errors]
enum MatroskaError {
	#[display("Unsupported EBML version {}", f0)]
	#[kind = ErrorKind::Unimplemented]
	UnknownEbmlVersion(u64),

	#[display("EBML id length too large ({} bytes)", f0)]
	#[kind = ErrorKind::Unsupported]
	IdTooLong(u64),

	#[display("EBML size length too large ({} bytes)", f0)]
	#[kind = ErrorKind::Unsupported]
	SizeTooLong(u64),

	#[display("Unknown doc type {}", f0)]
	#[kind = ErrorKind::Unimplemented]
	UnknownDocType(String),

	#[display("Unknown doc type version {}", f0)]
	#[kind = ErrorKind::Unimplemented]
	UnknownDocTypeVersion(u64)
}

struct Block {
	track_id: u64,
	timecode: u64,
	flags: u8,
	size: u64
}

struct Matroska {
	reader: Reader,

	stack: [MasterElemHdr; 16],
	level: usize,

	seen_header: bool,
	tracks: Option<Tracks>,
	seek_head: Option<SeekHead>,
	cues: Option<Cues>,

	duration: f64,
	timecode_scale: Rational,

	segment_offset: u64,
	cluster_timecode: u64,
	block: Option<Block>
}

#[asynchronous]
impl Matroska {
	fn new(reader: Reader) -> Self {
		Self {
			reader,

			stack: [MasterElemHdr::default(); 16],
			level: 0,

			seen_header: false,
			tracks: None,
			seek_head: None,
			cues: None,

			duration: 0.0,
			timecode_scale: Rational::default(),

			segment_offset: 0,
			cluster_timecode: 0,
			block: None
		}
	}

	fn stack_push(&mut self, master: MasterElemHdr) {
		self.stack[self.level] = master;

		#[allow(clippy::arithmetic_side_effects)]
		(self.level += 1);
	}

	#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
	fn stack_pop(&mut self) {
		self.level = self.level.checked_sub(1).unwrap();
	}

	fn new_segment(&mut self) {
		self.tracks = None;
		self.seek_head = None;
		self.cues = None;
		self.duration = 0.0;
		self.cluster_timecode = 0;
		self.block = None;
	}

	async fn post_read(&mut self, element: &ElemHdr) -> Result<()> {
		#[allow(clippy::never_loop)]
		loop {
			let Some(end) = element.end else { break };

			let remaining = end
				.get()
				.checked_sub(self.position())
				.ok_or(FormatError::ReadOverflow)?;

			if remaining == 0 {
				break;
			}

			#[allow(clippy::arithmetic_side_effects)]
			let indents = self.level * 2;

			trace!(target: &*self, "== {: <indents$}Remaining {} bytes in element", "", remaining);

			self.skip_element(element).await?;
		}

		Ok(())
	}

	fn verify_header(header: &Header) -> Result<()> {
		if header.version.0 != 1 {
			return Err(MatroskaError::UnknownEbmlVersion(header.version.0).into());
		}

		if header.max_id_length.0 > size_of::<u64>() as u64 {
			return Err(MatroskaError::IdTooLong(header.max_id_length.0).into());
		}

		if header.max_size_length.0 > size_of::<u64>() as u64 {
			return Err(MatroskaError::SizeTooLong(header.max_size_length.0).into());
		}

		match header.doc_type.0.as_ref() {
			"matroska" | "webm" => (),
			ty => return Err(MatroskaError::UnknownDocType(ty.into()).into())
		}

		if header.doc_type_version.0 > 4 {
			return Err(MatroskaError::UnknownDocTypeVersion(header.doc_type_version.0).into());
		}

		Ok(())
	}

	async fn read_root(&mut self) -> Result<()> {
		if self.level == 0 {
			self.stack_push(MasterElemHdr::root::<PartialMatroskaRoot>());
		}

		let mut stop = false;

		while !stop {
			#[allow(clippy::arithmetic_side_effects)]
			let master = &self.stack[self.level - 1];

			let element = match master.next_element(&mut self.reader).await? {
				Some(element) => element,
				None => {
					self.post_read(&master.element.clone()).await?;
					self.stack_pop();

					if self.level == 0 {
						break;
					}

					continue;
				}
			};

			if !element.known_end() {
				let msg = "Elements with unknown size are not currently supported";

				return Err(FormatError::InvalidData(msg.into()).into());
			}

			if !master.is_child(element.id) {
				#[allow(clippy::arithmetic_side_effects)]
				let indents = self.level * 2;

				trace!(target: &*self, "== {: <indents$}Skipping unknown element with id {} of size {}", "", element.id, element.size);

				self.skip_element(&element).await?;

				continue;
			}

			match element.id {
				EbmlRoot::HEADER_ID => {
					self.trace_element("Header", &element);

					let header = Header::parse(self, &element).await?;

					self.seen_header = true;
					Self::verify_header(&header)?;
				}

				MatroskaRoot::SEGMENTS_ID => {
					if !self.seen_header {
						warn!(target: &*self, "== Ebml header not found");

						self.seen_header = true;
					}

					self.trace_element("Segment", &element);
					self.stack_push(MasterElemHdr { element, children: PartialSegment::CHILDREN });
					self.new_segment();
					self.segment_offset = element.offset;

					continue;
				}

				Segment::INFO_ID => {
					self.trace_element("SegmentInfo", &element);

					let info = SegmentInfo::parse(self, &element).await?;

					if let Some(duration) = info.duration {
						self.duration = duration.0;
					}

					let num = info
						.timestamp_scale
						.0
						.try_into()
						.map_err(|_| FormatError::invalid_data())?;

					let timescale = Rational::new(num, 1_000_000_000);

					self.timecode_scale = timescale.reduce();
				}

				Segment::SEEK_HEAD_ID => {
					self.trace_element("SeekHead", &element);
					self.seek_head = Some(SeekHead::parse(self, &element).await?);
				}

				Segment::TRACKS_ID => {
					self.trace_element("Tracks", &element);
					self.tracks = Some(Tracks::parse(self, &element).await?);

					stop = true;
				}

				Segment::CUES_ID => {
					self.trace_element("Cues", &element);
					self.cues = Some(Cues::parse(self, &element).await?);
				}

				Segment::CLUSTERS_ID => {
					self.trace_element("Cluster", &element);
					self.stack_push(MasterElemHdr { element, children: PartialCluster::CHILDREN });

					continue;
				}

				Cluster::TIMESTAMP_ID => {
					self.trace_element("Timestamp", &element);
					self.cluster_timecode = Unsigned::parse(self, &element).await?.0;
				}

				Cluster::BLOCK_GROUPS_ID => {
					self.trace_element("BlockGroup", &element);
					self.stack_push(MasterElemHdr {
						element,
						children: PartialBlockGroup::CHILDREN
					});

					continue;
				}

				id @ (Cluster::SIMPLE_BLOCKS_ID | BlockGroup::BLOCK_ID) => {
					self.trace_element(
						if id == Cluster::SIMPLE_BLOCKS_ID {
							"SimpleBlock"
						} else {
							"Block"
						},
						&element
					);

					let header = BlockHeader::parse(self, &element).await?;

					#[allow(clippy::unwrap_used)]
					let size = element.remaining(self.reader.position()).unwrap();
					let timecode = self
						.cluster_timecode
						.checked_add(header.timecode as u64)
						.ok_or(ErrorKind::Overflow)?;

					self.block = Some(Block {
						track_id: header.track_id.0,
						timecode,
						flags: header.flags,
						size
					});

					break;
				}

				_ => ()
			}

			self.post_read(&element).await?;
		}

		Ok(())
	}
}

impl Deref for Matroska {
	type Target = Reader;

	fn deref(&self) -> &Reader {
		&self.reader
	}
}

impl DerefMut for Matroska {
	fn deref_mut(&mut self) -> &mut Reader {
		&mut self.reader
	}
}

#[asynchronous]
impl EbmlReader for Matroska {
	async fn read_children<F>(&mut self, master: &MasterElemHdr, mut handle_child: F) -> Result<()>
	where
		F: AsyncFnMut(&mut Self, &ElemHdr) -> Result<bool>
	{
		#[allow(clippy::arithmetic_side_effects)]
		(self.level += 1);

		default_read_children(
			self,
			master,
			|this: &mut Self, element: &ElemHdr| async move {
				let matched = handle_child.call_mut((this, element)).await?;

				this.post_read(element).await?;

				Ok(matched)
			}
		)
		.await?;

		#[allow(clippy::arithmetic_side_effects)]
		(self.level -= 1);
		self.post_read(&master.element).await?;

		Ok(())
	}

	fn trace_element(&self, name: &str, element: &ElemHdr) {
		#[allow(clippy::arithmetic_side_effects)]
		let indents = (self.level - 1) * 2;

		trace!(
			target: self,
			"== {: <indents$}{: <20} : {} @ {}", "",
			name,
			element.size,
			element.offset
		);
	}
}

const fn get_track_type(ty: TrackType) -> MediaType {
	match ty {
		TrackType::Video => MediaType::Video,
		TrackType::Audio => MediaType::Audio,
		TrackType::Subtitle => MediaType::Subtitle,
		_ => MediaType::Unknown
	}
}

fn get_track_codec(id: &str) -> CodecId {
	match id {
		"A_AAC" => CodecId::Aac,
		"A_OPUS" => CodecId::Opus,
		"A_FLAC" => CodecId::Flac,
		"A_VORBIS" => CodecId::Vorbis,
		"A_MPEG/L3" => CodecId::Mp3,
		_ => CodecId::Unknown
	}
}

fn get_track_codec_params(track: &tracks::Track) -> Result<CodecParams> {
	fn conv_float(value: f64) -> Result<u32> {
		#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
		let result = value as u32;

		if result as f64 == value {
			Ok(result)
		} else {
			Err(ErrorKind::Overflow.into())
		}
	}

	fn trunc<T>(value: u64) -> Result<T>
	where
		T: TryFrom<u64>
	{
		T::try_from(value).map_err(|_| ErrorKind::Overflow.into())
	}

	let mut params = CodecParams::default();

	params.id = get_track_codec(&track.codec_id);

	if let Some(private) = &track.codec_private {
		params.config.clone_from(&private.0);
	}

	if let Some(audio) = &track.audio {
		params.sample_rate = conv_float(audio.sampling_frequency.0)?;
		params.ch_layout = ChannelLayout::Unspec(trunc(audio.channels.0)?);

		if let Some(output_sr) = &audio.output_sampling_frequency {
			params.sample_rate = conv_float(output_sr.0)?;
		}

		if let Some(bit_depth) = &audio.bit_depth {
			params.bit_depth = trunc(bit_depth.0)?;
		}
	}

	params.time_base = Rational::nanos();
	params.delay = trunc(track.codec_delay.0)?;
	params.seek_preroll = trunc(track.seek_preroll.0)?;

	Ok(params)
}

#[asynchronous]
impl DemuxerImpl for Matroska {
	#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
	async fn open(&mut self, context: &mut FormatData) -> Result<()> {
		self.read_root().await?;

		let tracks = self.tracks.as_ref().ok_or(FormatError::NoTracks)?;

		context.duration = self.duration as u64;
		context.time_base = self.timecode_scale;

		for track in &tracks.tracks {
			let params = get_track_codec_params(track)?;

			let parse = if params.id != CodecId::Aac {
				CodecParse::Header
			} else {
				CodecParse::default()
			};

			context.tracks.push(Track {
				ty: get_track_type(track.ty),
				codec_params: params,
				id: track.number.0,
				duration: self.duration as u64,
				time_base: self.timecode_scale,
				parse,
				..Default::default()
			});
		}

		Ok(())
	}

	async fn seek(
		&mut self, _: &mut FormatData, track: u32, timecode: u64, _flags: BitFlags<SeekFlag>
	) -> Result<()> {
		#[allow(clippy::unwrap_used)]
		let track = &self.tracks.as_ref().unwrap().tracks[track as usize];
		let cues = self.cues.as_ref().ok_or(FormatError::CannotSeek)?;

		#[allow(clippy::arithmetic_side_effects)]
		let index = match cues
			.points
			.binary_search_by(|point| point.time.0.cmp(&timecode))
		{
			Ok(index) => index,
			Err(index) => index - 1
		};

		let mut cluster_position = None;

		for point in cues.points[0..=index].iter().rev() {
			if let Some(pos) = point
				.track_positions
				.iter()
				.find(|pos| pos.track == track.number)
			{
				cluster_position = Some(pos.cluster_position.0);

				break;
			}
		}

		let offset = self
			.segment_offset
			.checked_add(cluster_position.unwrap_or(0))
			.ok_or(ErrorKind::Overflow)?;

		self.reader.seek(SeekFrom::Start(offset)).await?;

		#[allow(clippy::arithmetic_side_effects)]
		while self.stack[self.level - 1].element.id != MatroskaRoot::SEGMENTS_ID {
			self.stack_pop();
		}

		self.block = None;

		Ok(())
	}

	#[allow(clippy::unwrap_used)]
	async fn read_packet(&mut self, context: &mut FormatData, packet: &mut Packet) -> Result<bool> {
		if self.block.is_none() {
			self.read_root().await?;
		}

		let block = match self.block.take() {
			Some(block) => block,
			None => return Ok(false)
		};

		context.get_packet_fields_for(packet, block.track_id)?;
		packet.timestamp = block.timecode.try_into().unwrap();

		if block.flags & 0x80 != 0 {
			packet.flags |= PacketFlag::Keyframe;
		}

		packet.data = self
			.reader
			.read_bytes(block.size.try_into().map_err(|_| ErrorKind::Overflow)?)
			.await?;

		Ok(true)
	}
}

struct Probe<'a> {
	reader: &'a mut Reader
}

impl Deref for Probe<'_> {
	type Target = Reader;

	fn deref(&self) -> &Reader {
		self.reader
	}
}

impl DerefMut for Probe<'_> {
	fn deref_mut(&mut self) -> &mut Reader {
		self.reader
	}
}

impl EbmlReader for Probe<'_> {}

#[allow(missing_copy_implementations)]
pub struct MatroskaClass;

#[asynchronous]
async fn do_probe(reader: &mut Reader) -> Result<f32> {
	let mut probe = Probe { reader };
	let mut score = 0.0;

	let master = MasterElemHdr::root::<PartialMatroskaRoot>();

	for _ in 0..4 {
		let element = match master.next_element(probe.reader).await? {
			Some(element) => element,
			None => break
		};

		match element.id {
			EbmlGlobal::VOID_ID | EbmlGlobal::CRC_32_ID | MatroskaRoot::SEGMENTS_ID => {
				score = 0.25;
			}

			EbmlRoot::HEADER_ID => {
				if Header::parse(&mut probe, &element).await.is_ok() {
					score = 1.0;
				} else {
					score = 0.0;
				}

				break;
			}

			_ => ()
		}

		probe.skip_element(&element).await?;
	}

	Ok(score)
}

#[asynchronous]
impl DemuxerClassImpl for MatroskaClass {
	fn name(&self) -> &'static str {
		"Matroska / WebM"
	}

	async fn create(&self, reader: Reader) -> Result<Demuxer> {
		Ok(Box::new(Matroska::new(reader)))
	}

	async fn probe(&self, reader: &mut Reader) -> Result<f32> {
		let err = match do_probe(reader).await {
			Ok(ok) => return Ok(ok),
			Err(err) => err
		};

		if err.downcast_ref::<EbmlError>().is_some() || err.downcast_ref::<FormatError>().is_some()
		{
			Ok(0.0)
		} else {
			Err(err)
		}
	}
}
