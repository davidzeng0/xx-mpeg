use std::borrow::Cow;

use super::*;

#[errors]
pub enum FormatError {
	#[display("Unknown format")]
	UnknownFormat,

	#[display("Track not found")]
	#[kind = ErrorKind::InvalidData]
	TrackNotFound,

	#[display(transparent)]
	#[kind = ErrorKind::InvalidData]
	InvalidData(Cow<'static, str>),

	#[display("Codec not found")]
	#[kind = ErrorKind::NotFound]
	CodecNotFound,

	#[display("Read overflowed")]
	#[kind = ErrorKind::InvalidData]
	ReadOverflow,

	#[display("Invalid seek: requested position {}, got {}", f0, f1)]
	InvalidSeek(u64, u64),

	#[display("No tracks")]
	NoTracks,

	#[display("Cannot seek this stream")]
	#[kind = ErrorKind::NotSeekable]
	CannotSeek
}

impl FormatError {
	#[must_use]
	pub fn invalid_data() -> Self {
		Self::InvalidData("Invalid data found while processing input".into())
	}
}
