use std::borrow::Cow;

use super::*;

#[errors]
pub enum FormatError {
	#[error("Unknown format")]
	UnknownFormat,

	#[error("Track not found")]
	#[kind = ErrorKind::InvalidData]
	TrackNotFound,

	#[error(transparent)]
	#[kind = ErrorKind::InvalidData]
	InvalidData(Cow<'static, str>),

	#[error("Codec not found")]
	#[kind = ErrorKind::NotFound]
	CodecNotFound,

	#[error("Read overflowed")]
	#[kind = ErrorKind::InvalidData]
	ReadOverflow,

	#[error("Invalid seek: requested position {}, got {}", f0, f1)]
	InvalidSeek(u64, u64),

	#[error("No tracks")]
	NoTracks,

	#[error("Cannot seek this stream")]
	#[kind = ErrorKind::NotSeekable]
	CannotSeek
}

impl FormatError {
	#[must_use]
	pub fn invalid_data() -> Self {
		Self::InvalidData("Invalid data found while processing input".into())
	}
}
