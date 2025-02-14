use std::mem::zeroed;

use super::*;
use crate::codec::CodecId;

impl From<Rational> for AVRational {
	fn from(value: Rational) -> Self {
		let Rational { num, den } = value;

		#[allow(clippy::unwrap_used)]
		Self {
			num: num.try_into().unwrap(),
			den: den.try_into().unwrap()
		}
	}
}

impl From<AVRational> for Rational {
	fn from(value: AVRational) -> Self {
		let AVRational { num, den } = value;

		#[allow(clippy::unwrap_used)]
		Self {
			num: num.try_into().unwrap(),
			den: den.try_into().unwrap()
		}
	}
}

impl From<u64> for Channel {
	fn from(channel: u64) -> Self {
		Self::from_u64(channel).unwrap_or(Self::Unknown)
	}
}

impl From<AVCodecID> for CodecId {
	fn from(id: AVCodecID) -> Self {
		match id {
			AVCodecID::AV_CODEC_ID_AAC => Self::Aac,
			AVCodecID::AV_CODEC_ID_OPUS => Self::Opus,
			AVCodecID::AV_CODEC_ID_FLAC => Self::Flac,
			AVCodecID::AV_CODEC_ID_VORBIS => Self::Vorbis,
			AVCodecID::AV_CODEC_ID_MP2 => Self::Mp2,
			AVCodecID::AV_CODEC_ID_MP3 => Self::Mp3,
			_ => Self::Unknown
		}
	}
}

pub(super) fn result_from_av(code: i32) -> Result<i32> {
	if code >= 0 {
		return Ok(code);
	}

	Err(match code {
		AVERROR_BSF_NOT_FOUND => AVError::BitstreamFilterNotFound.into(),
		AVERROR_BUG | AVERROR_BUG2 => AVError::InternalBug.into(),
		AVERROR_BUFFER_TOO_SMALL => AVError::BufferTooSmall.into(),
		AVERROR_DECODER_NOT_FOUND => FormatError::CodecNotFound.into(),
		AVERROR_DEMUXER_NOT_FOUND => AVError::DemuxerNotFound.into(),
		AVERROR_ENCODER_NOT_FOUND => FormatError::CodecNotFound.into(),
		AVERROR_EOF => AVError::EndOfFile.into(),
		AVERROR_EXIT => AVError::ExitRequested.into(),
		AVERROR_EXTERNAL => AVError::ExternalError.into(),
		AVERROR_FILTER_NOT_FOUND => AVError::FilterNotFound.into(),
		AVERROR_INVALIDDATA => AVError::InvalidData.into(),
		AVERROR_MUXER_NOT_FOUND => AVError::MuxerNotFound.into(),
		AVERROR_OPTION_NOT_FOUND => AVError::OptionNotFound.into(),
		AVERROR_PATCHWELCOME => AVError::NotImplemented.into(),
		AVERROR_PROTOCOL_NOT_FOUND => AVError::ProtocolNotFound.into(),
		AVERROR_STREAM_NOT_FOUND => AVError::StreamNotFound.into(),
		AVERROR_UNKNOWN => AVError::Unknown.into(),
		AVERROR_EXPERIMENTAL => AVError::Experimental.into(),
		AVERROR_INPUT_CHANGED => AVError::InputChanged.into(),
		AVERROR_OUTPUT_CHANGED => AVError::OutputChanged.into(),
		AVERROR_HTTP_BAD_REQUEST => AVError::HttpBadRequest.into(),
		AVERROR_HTTP_UNAUTHORIZED => AVError::HttpUnauthorized.into(),
		AVERROR_HTTP_FORBIDDEN => AVError::HttpForbidden.into(),
		AVERROR_HTTP_NOT_FOUND => AVError::HttpNotFound.into(),
		AVERROR_HTTP_OTHER_4XX => AVError::HttpOther4xx.into(),
		AVERROR_HTTP_SERVER_ERROR => AVError::HttpServerError.into(),
		code => OsError::from(AVUNERROR(code)).into()
	})
}

pub(super) fn av_from_error(err: &Error) -> i32 {
	match err.os_error() {
		Some(err) => AVERROR(err as i32),
		None => AVERROR_EXTERNAL
	}
}

pub(super) fn result_from_av_maybe_none(err: i32) -> Result<bool> {
	const AGAIN: i32 = AVERROR(OsError::Again as i32);

	match err {
		AVERROR_EOF | AGAIN => Ok(false),
		err => {
			result_from_av(err)?;

			Ok(true)
		}
	}
}

/// # Panics
/// if the input str is not a valid cstring
pub(super) fn into_cstr(str: &str) -> CString {
	#[allow(clippy::expect_used)]
	CString::new(str).expect("Valid C string")
}

impl From<&AVChannelLayout> for ChannelLayout {
	#[allow(clippy::unwrap_used, clippy::multiple_unsafe_ops_per_block)]
	fn from(value: &AVChannelLayout) -> Self {
		let channels = value.nb_channels.try_into().unwrap();

		/* Safety: read mask */
		let mask = unsafe { value.u.mask };

		let layout = match value.order.into() {
			ChannelOrder::Unspec => Self::Unspec(channels),
			ChannelOrder::Native => Self::Native(channels, mask),
			ChannelOrder::Custom => Self::Custom({
				let mut custom = Vec::new();

				/* Safety: the channel mapping is owned */
				let channels = unsafe {
					MutPtr::slice_from_raw_parts(value.u.map.into(), channels as usize).as_mut()
				};

				for channel in channels {
					custom.push(ChannelCustom {
						id: channel.id.into(),

						/* Safety: transmute i8 to u8 */
						name: unsafe { transmute(channel.name) }
					});
				}

				custom
			}),

			ChannelOrder::Ambisonic => Self::Ambisonic(channels, mask)
		};

		layout
	}
}

impl From<AVChannelLayout> for ChannelLayout {
	fn from(mut value: AVChannelLayout) -> Self {
		let layout = Self::from(&value);

		ffi!(av_channel_layout_uninit, &mut value);

		layout
	}
}

impl From<&ChannelLayout> for AVChannelLayout {
	#[allow(clippy::unwrap_used, clippy::multiple_unsafe_ops_per_block)]
	fn from(value: &ChannelLayout) -> Self {
		/* Safety: repr C */
		let mut layout: Self = unsafe { zeroed() };

		match value {
			ChannelLayout::Unspec(channels) => {
				layout.order = AVChannelOrder::AV_CHANNEL_ORDER_UNSPEC;
				layout.nb_channels = *channels as i32;
			}

			ChannelLayout::Native(channels, mask) => {
				layout.order = AVChannelOrder::AV_CHANNEL_ORDER_NATIVE;
				layout.nb_channels = *channels as i32;
				layout.u.mask = *mask;
			}

			ChannelLayout::Ambisonic(channels, mask) => {
				layout.order = AVChannelOrder::AV_CHANNEL_ORDER_AMBISONIC;
				layout.nb_channels = *channels as i32;
				layout.u.mask = *mask;
			}

			ChannelLayout::Custom(custom) => {
				ffi!(
					av_channel_layout_custom_init,
					&mut layout,
					custom.len().try_into().unwrap()
				)
				.unwrap();

				/* Safety: channel mapping is owned */
				let channels = unsafe {
					MutPtr::slice_from_raw_parts(layout.u.map.into(), custom.len()).as_mut()
				};

				for (channel, custom) in channels.iter_mut().zip(custom.iter()) {
					channel.id = custom.id.into();

					/* Safety: transmute u8 to i8 */
					channel.name = unsafe { transmute(custom.name) };
				}
			}
		}

		layout
	}
}

impl From<ChannelLayout> for AVChannelLayout {
	fn from(value: ChannelLayout) -> Self {
		Self::from(&value)
	}
}
