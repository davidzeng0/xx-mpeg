#![allow(clippy::new_ret_no_self, missing_copy_implementations)]

use xx_core::macros::paste;

use super::*;

mod av;

pub mod aac;
pub mod flac;
pub mod mp2;
pub mod mp3;
pub mod opus;
pub mod vorbis;

pub use aac::*;
pub use flac::*;
pub use mp2::*;
pub use mp3::*;
pub use opus::{OpusDecoder, OpusEncoder, OpusParser};
pub use vorbis::*;

macro_rules! codec_pair {
	($codec_id:expr, $codec_name:expr, $av_codec:expr, $name:ident) => {
		paste! {
			pub struct [<$name Encoder>];

			impl [<$name Encoder>] {
				pub fn new(params: &mut CodecParams) -> Result<Box<dyn CodecImpl + Send + Sync>> {
					use crate::av::*;

					let codec_name: Option<&'static str> = $codec_name;
					let codec = codec_name.map(Codecs::find_encoder_by_name).unwrap_or_else(|| Codecs::find_encoder($av_codec));

					if let Some(codec) = codec {
						Ok(Box::new(AVCodec::new($codec_id, codec.into(), params, Mode::Encode)?))
					} else {
						Err(FormatError::CodecNotFound.into())
					}
				}
			}

			pub struct [<$name Decoder>];

			impl [<$name Decoder>] {
				pub fn new(params: &mut CodecParams) -> Result<Box<dyn CodecImpl + Send + Sync>> {
					use crate::av::*;
					let codec_name: Option<&'static str> = $codec_name;
					let codec = codec_name.map(Codecs::find_decoder_by_name).unwrap_or_else(|| Codecs::find_decoder($av_codec));

					if let Some(codec) = codec {
						Ok(Box::new(AVCodec::new($codec_id, codec.into(), params, Mode::Decode)?))
					} else {
						Err(FormatError::CodecNotFound.into())
					}
				}
			}
		}
	};
}

use codec_pair;

macro_rules! parser_pair {
	($codec_id:expr, $av_codec:expr, $name:ident) => {
		paste! {
			pub struct [<$name Parser>];

			impl [<$name Parser>] {
				pub fn new(parse: CodecParse, params: &mut CodecParams) -> Result<Box<dyn CodecParserImpl + Send + Sync>> {
					use xx_core::pointer::*;
					use crate::av::*;

					fn get_codec_parser() -> Option<(NonNull<ffmpeg_sys_next::AVCodec>, ParserContext)> {
						let codec = Codecs::find_decoder($av_codec)?;
						let parser = ParserContext::try_new($av_codec)?;

						Some((codec, parser))
					}

					let (codec, parser) = get_codec_parser().ok_or(FormatError::CodecNotFound)?;
					let parser = AVCodecParser::new($codec_id, codec, parser, parse, params)?;

					Ok(Box::new(parser))
				}
			}
		}
	};
}

use parser_pair;
