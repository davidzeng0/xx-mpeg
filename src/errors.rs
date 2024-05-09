use super::*;

#[errors]
pub enum FormatError {
	#[error("Unknown format")]
	UnknownFormat,

	#[error("Track not found")]
	TrackNotFound,

	#[error("{0}")]
	InvalidData(SimpleMessage),

	#[error("Codec not found")]
	CodecNotFound,

	#[error("Read overflowed")]
	ReadOverflow,

	#[error("Invalid seek: requested position {0}, got {1}")]
	InvalidSeek(u64, u64),

	#[error("No tracks")]
	NoTracks,

	#[error("Cannot seek this stream")]
	CannotSeek
}

impl FormatError {
	#[must_use]
	pub fn invalid_data() -> Self {
		Self::InvalidData("Invalid data found while processing input".into())
	}
}
