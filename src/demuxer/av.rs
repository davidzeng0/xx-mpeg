#![allow(unreachable_pub)]

use ffmpeg_sys_next::AVInputFormat;
use xx_core::impls::UintExt;
use xx_core::pointer::*;

use super::*;
use crate::av::{AVPacket, FormatContext, ProbeResult, TIME_BASE};

struct AVDemuxer {
	format: FormatContext,
	reader: Reader,
	packet: AVPacket
}

#[asynchronous]
impl AVDemuxer {
	fn new(reader: Reader, input_format: Option<NonNull<AVInputFormat>>) -> Self {
		let mut format = FormatContext::new();

		format.iformat = input_format
			.map(NonNull::as_pointer)
			.unwrap_or_default()
			.as_ptr();

		Self { format, reader, packet: AVPacket::new() }
	}
}

#[asynchronous]
impl DemuxerImpl for AVDemuxer {
	#[allow(clippy::unwrap_used)]
	async fn open(&mut self, context: &mut FormatData) -> Result<()> {
		self.format.open(&mut self.reader).await?;

		context.start_time = self.format.start_time;
		context.duration = self.format.duration.try_into().unwrap();
		context.duration = context
			.duration
			.checked_sub_signed(context.start_time)
			.unwrap();
		context.time_base = Rational::inverse(TIME_BASE);

		for index in 0..self.format.nb_streams {
			let stream_ptr = ptr!(self.format.streams);

			#[allow(clippy::multiple_unsafe_ops_per_block)]
			/* Safety: FFI */
			unsafe {
				let stream = ptr!(ptr!(*stream_ptr.add(index as usize))).as_ref();
				let params = ptr!(stream.codecpar).as_ref();

				let mut codec_params = CodecParams::default();

				codec_params.id = params.codec_id.into();

				if !params.extradata.is_null() {
					codec_params.config = MutPtr::slice_from_raw_parts(
						params.extradata.into(),
						params.extradata_size.try_into().unwrap()
					)
					.as_mut()
					.to_vec();
				}

				codec_params.bit_rate = params.bit_rate.try_into().unwrap();
				codec_params.bit_depth = params.bits_per_raw_sample.try_into().unwrap();
				codec_params.seek_preroll = params.seek_preroll.try_into().unwrap();

				codec_params.ch_layout = (&params.ch_layout).into();
				codec_params.sample_rate = params.sample_rate.try_into().unwrap();
				codec_params.frame_size = params.frame_size.try_into().unwrap();

				codec_params.width = params.width.try_into().unwrap();
				codec_params.height = params.height.try_into().unwrap();
				codec_params.sample_aspect_ratio = params.sample_aspect_ratio.into();
				codec_params.framerate = params.framerate.into();

				let ty = params.codec_type.into();

				match ty {
					MediaType::Video => codec_params.delay = params.video_delay.try_into().unwrap(),
					MediaType::Audio => {
						codec_params.delay = params.initial_padding.try_into().unwrap();
						codec_params.time_base = Rational::inverse(codec_params.sample_rate);
					}

					_ => ()
				}

				let start_time = match stream.start_time {
					UNKNOWN_TIMESTAMP => 0,
					ts => ts
				};

				context.tracks.push(Track {
					ty,
					time_base: stream.time_base.into(),
					codec_params,
					id: stream.id.try_into().unwrap(),
					start_time,
					duration: stream.duration.try_into().unwrap_or(0),
					..Default::default()
				});
			}
		}

		Ok(())
	}

	async fn seek(
		&mut self, data: &mut FormatData, track_index: u32, timecode: u64,
		flags: BitFlags<SeekFlag>
	) -> Result<()> {
		let track = &data.tracks[track_index as usize];

		#[allow(clippy::unwrap_used)]
		self.format
			.seek(
				track_index,
				timecode
					.checked_add_signed(track.start_time.checked_neg().unwrap())
					.unwrap(),
				flags,
				&mut self.reader
			)
			.await?;

		Ok(())
	}

	#[allow(clippy::unwrap_used, clippy::cast_sign_loss)]
	async fn read_packet(&mut self, context: &mut FormatData, packet: &mut Packet) -> Result<bool> {
		for index in 0..self.format.nb_streams {
			let stream_ptr = ptr!(self.format.streams);

			#[allow(clippy::multiple_unsafe_ops_per_block)]
			/* Safety: FFI */
			unsafe {
				let stream = ptr!(ptr!(*stream_ptr.add(index as usize)));

				ptr!(stream=>discard) = context.tracks[index as usize].discard.into();
			}
		}

		if !self
			.format
			.read_frame(&mut self.packet, &mut self.reader)
			.await?
		{
			return Ok(false);
		}

		let index: usize = self.packet.stream_index.try_into().unwrap();
		let track = &context.tracks[index];

		#[allow(clippy::cast_possible_truncation)]
		(packet.track_index = index as u32);
		packet.time_base = track.time_base;

		/* Safety: non-null */
		packet.data = unsafe { self.packet.data().as_ref().to_vec() };
		packet.timestamp = self.packet.dts.checked_sub(track.start_time).unwrap();
		packet.duration = self.packet.duration.try_into().unwrap_or(0);
		packet.flags = BitFlags::from_bits_truncate(self.packet.flags as u32);

		self.packet.unref();

		Ok(true)
	}
}

#[allow(missing_copy_implementations)]
pub struct AVFormatClass;

#[asynchronous]
impl AVFormatClass {
	pub fn create(reader: Reader, format: Option<NonNull<AVInputFormat>>) -> Demuxer {
		Box::new(AVDemuxer::new(reader, format))
	}

	pub async fn probe(reader: &mut Reader) -> Result<Option<ProbeResult>> {
		FormatContext::probe(reader).await
	}
}
