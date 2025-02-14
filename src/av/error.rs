use super::*;

#[errors]
pub enum AVError {
	#[display("Bitstream filter not found")]
	#[kind = ErrorKind::NotFound]
	BitstreamFilterNotFound,

	#[display("AV internal bug")]
	InternalBug,

	#[display("Buffer too small")]
	#[kind = ErrorKind::InvalidInput]
	BufferTooSmall,

	#[display("Demuxer not found")]
	#[kind = ErrorKind::InvalidData]
	DemuxerNotFound,

	#[display("End of file")]
	EndOfFile,

	#[display("Exit requested")]
	ExitRequested,

	#[display("Error in external library")]
	ExternalError,

	#[display("Filter not found")]
	#[kind = ErrorKind::NotFound]
	FilterNotFound,

	#[display("Invalid data found while processing input")]
	#[kind = ErrorKind::InvalidData]
	InvalidData,

	#[display("Muxer not found")]
	#[kind = ErrorKind::NotFound]
	MuxerNotFound,

	#[display("Option not found")]
	#[kind = ErrorKind::NotFound]
	OptionNotFound,

	#[display("Not implemented")]
	#[kind = ErrorKind::Unimplemented]
	NotImplemented,

	#[display("Protocol not found")]
	#[kind = ErrorKind::NotFound]
	ProtocolNotFound,

	#[display("Stream not found")]
	#[kind = ErrorKind::NotFound]
	StreamNotFound,

	#[display("Unknown error or error in external library")]
	Unknown,

	#[display("Feature is experimental")]
	Experimental,

	#[display("Input changed")]
	InputChanged,

	#[display("Output changed")]
	OutputChanged,

	#[display("HTTP bad request")]
	HttpBadRequest,

	#[display("HTTP unauthorized")]
	HttpUnauthorized,

	#[display("HTTP forbidden")]
	HttpForbidden,

	#[display("HTTP not found")]
	HttpNotFound,

	#[display("HTTP 4xx")]
	HttpOther4xx,

	#[display("HTTP server error")]
	HttpServerError
}
