#![allow(clippy::module_name_repetitions)]

use super::*;

#[errors]
pub enum AVError {
	#[error("Bitstream filter not found")]
	BitstreamFilterNotFound,

	#[error("AV internal bug")]
	InternalBug,

	#[error("Buffer too small")]
	BufferTooSmall,

	#[error("Demuxer not found")]
	DemuxerNotFound,

	#[error("End of file")]
	EndOfFile,

	#[error("Exit requested")]
	ExitRequested,

	#[error("Error in external library")]
	ExternalError,

	#[error("Filter not found")]
	FilterNotFound,

	#[error("Invalid data found while processing input")]
	InvalidData,

	#[error("Muxer not found")]
	MuxerNotFound,

	#[error("Option not found")]
	OptionNotFound,

	#[error("Not implemented")]
	NotImplemented,

	#[error("Protocol not found")]
	ProtocolNotFound,

	#[error("Stream not found")]
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
