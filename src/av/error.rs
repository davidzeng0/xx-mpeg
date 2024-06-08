#![allow(clippy::module_name_repetitions)]

use super::*;

#[errors]
pub enum AVError {
	#[error("Bitstream filter not found")]
	#[kind = ErrorKind::NotFound]
	BitstreamFilterNotFound,

	#[error("AV internal bug")]
	InternalBug,

	#[error("Buffer too small")]
	#[kind = ErrorKind::InvalidInput]
	BufferTooSmall,

	#[error("Demuxer not found")]
	#[kind = ErrorKind::InvalidData]
	DemuxerNotFound,

	#[error("End of file")]
	EndOfFile,

	#[error("Exit requested")]
	ExitRequested,

	#[error("Error in external library")]
	ExternalError,

	#[error("Filter not found")]
	#[kind = ErrorKind::NotFound]
	FilterNotFound,

	#[error("Invalid data found while processing input")]
	#[kind = ErrorKind::InvalidData]
	InvalidData,

	#[error("Muxer not found")]
	#[kind = ErrorKind::NotFound]
	MuxerNotFound,

	#[error("Option not found")]
	#[kind = ErrorKind::NotFound]
	OptionNotFound,

	#[error("Not implemented")]
	#[kind = ErrorKind::Unimplemented]
	NotImplemented,

	#[error("Protocol not found")]
	#[kind = ErrorKind::NotFound]
	ProtocolNotFound,

	#[error("Stream not found")]
	#[kind = ErrorKind::NotFound]
	StreamNotFound,

	#[error("Unknown error or error in external library")]
	Unknown,

	#[error("Feature is experimental")]
	Experimental,

	#[error("Input changed")]
	InputChanged,

	#[error("Output changed")]
	OutputChanged,

	#[error("HTTP bad request")]
	HttpBadRequest,

	#[error("HTTP unauthorized")]
	HttpUnauthorized,

	#[error("HTTP forbidden")]
	HttpForbidden,

	#[error("HTTP not found")]
	HttpNotFound,

	#[error("HTTP 4xx")]
	HttpOther4xx,

	#[error("HTTP server error")]
	HttpServerError
}
