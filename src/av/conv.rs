use super::*;

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
		code => OsError::from_raw(AVUNERROR(code)).into()
	})
}

pub(super) fn av_from_error(err: &Error) -> i32 {
	match err.os_error() {
		Some(err) => AVERROR(err as i32),
		None => AVERROR_EXTERNAL
	}
}

pub fn result_from_av_maybe_none(err: i32) -> Result<bool> {
	const AGAIN: i32 = AVERROR(OsError::Again as i32);

	match err {
		AVERROR_EOF | AGAIN => Ok(false),
		err => {
			result_from_av(err)?;

			Ok(true)
		}
	}
}

pub fn into_cstr(str: &str) -> CString {
	#[allow(clippy::expect_used)]
	CString::new(str).expect("Valid C string")
}
