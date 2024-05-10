#![allow(unreachable_pub)]

use xx_core::pointer::*;

use super::*;
use crate::av::{AVCodecID, AVPacket, FormatContext, ProbeResult, TIME_BASE};

struct AVDemuxer {
	format: FormatContext,
	reader: Reader,
	packet: AVPacket
}

#[asynchronous]
impl AVDemuxer {
	fn new(reader: Reader) -> Self {
		Self {
			format: FormatContext::new(),
			reader,
			packet: AVPacket::new()
		}
	}
}

#[asynchronous]
impl DemuxerImpl for AVDemuxer {
	#[allow(clippy::unwrap_used, clippy::field_reassign_with_default)]
	async fn open(&mut self, context: &mut FormatData) -> Result<()> {
		self.format.open(&mut self.reader).await?;

		context.duration = self.format.duration.try_into().unwrap();
		context.start_time = self.format.start_time;
		context.time_base = Rational::inverse(TIME_BASE);

		for index in 0..self.format.nb_streams {
			let stream_ptr = MutPtr::from(self.format.streams);

			#[allow(clippy::multiple_unsafe_ops_per_block)]
			/* Safety: FFI */
			unsafe {
				let stream = MutPtr::from(ptr!(*stream_ptr.add(index as usize))).as_ref();
				let params = MutPtr::from(stream.codecpar).as_ref();

				let mut codec_params = CodecParams::default();

				codec_params.id = match params.codec_id {
					AVCodecID::AV_CODEC_ID_AAC => CodecId::Aac,
					AVCodecID::AV_CODEC_ID_OPUS => CodecId::Opus,
					AVCodecID::AV_CODEC_ID_FLAC => CodecId::Flac,
					AVCodecID::AV_CODEC_ID_VORBIS => CodecId::Vorbis,
					AVCodecID::AV_CODEC_ID_MP3 => CodecId::Mp3,
					_ => CodecId::Unknown
				};

				codec_params.config = MutPtr::slice_from_raw_parts(
					params.extradata.into(),
					params.extradata_size.try_into().unwrap()
				)
				.as_mut()
				.to_vec();

				codec_params.bit_rate = params.bit_rate.try_into().unwrap();
				codec_params.bit_depth = params.bits_per_raw_sample.try_into().unwrap();
				codec_params.seek_preroll = params.seek_preroll.try_into().unwrap();

				codec_params.channel_layout = params.channel_layout;
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

				context.tracks.push(Track {
					ty,
					time_base: stream.time_base.into(),
					codec_params,
					id: stream.id.try_into().unwrap(),
					start_time: stream.start_time,
					duration: stream.duration.try_into().unwrap_or(0),
					..Default::default()
				});
			}
		}

		Ok(())
	}

	async fn seek(&mut self, track: u32, timecode: u64, flags: BitFlags<SeekFlag>) -> Result<()> {
		self.format
			.seek(track, timecode, flags, &mut self.reader)
			.await?;

		Ok(())
	}

	#[allow(clippy::unwrap_used, clippy::cast_sign_loss)]
	async fn read_packet(&mut self, context: &mut FormatData, packet: &mut Packet) -> Result<bool> {
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
		packet.duration = self.packet.duration.try_into().unwrap();
		packet.flags = BitFlags::from_bits_truncate(self.packet.flags as u32);

		self.packet.unref();

		Ok(true)
	}
}

#[allow(missing_copy_implementations)]
pub struct AVFormatClass;

#[asynchronous]
impl AVFormatClass {
	pub async fn create(reader: Reader) -> Result<Demuxer> {
		Ok(Box::new(AVDemuxer::new(reader)))
	}

	pub async fn probe(reader: &mut Reader) -> Result<Option<ProbeResult>> {
		FormatContext::probe(reader).await
	}
}
