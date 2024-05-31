#![allow(
	clippy::module_name_repetitions,
	clippy::new_ret_no_self,
	missing_copy_implementations
)]

use xx_core::paste::paste;

use super::*;

mod av;

pub mod aac;
pub mod flac;
pub mod mp3;
pub mod opus;
pub mod vorbis;

pub use aac::*;
pub use flac::*;
pub use mp3::*;
pub use opus::{OpusDecoder, OpusEncoder, OpusParser};
pub use vorbis::*;

macro_rules! codec_pair {
	($codec_id:expr, $codec_name:expr, $av_codec:expr, $name:ident) => {
		paste! {
			pub struct [<$name Encoder>];

			impl [<$name Encoder>] {
				pub fn new(params: &mut CodecParams) -> Result<Box<dyn CodecImpl>> {
					use ::std::ffi::CString;
					use ::xx_core::pointer::*;
					use ::ffmpeg_sys_next::{avcodec_find_encoder_by_name, avcodec_find_encoder};

					let codec_name: Option<&'static str> = $codec_name;

					let mut codec = if let Some(name) = codec_name {
						let name = CString::new(name).unwrap();

						/* Safety: FFI call */
						unsafe { avcodec_find_encoder_by_name(name.as_ptr()) }.into()
					} else {
						Ptr::null()
					};

					if codec.is_null() {
						/* Safety: FFI call */
						codec = unsafe { avcodec_find_encoder($av_codec) }.into();
					}

					if !codec.is_null() {
						Ok(Box::new(AVCodec::new($codec_id, codec.into(), params, Mode::Encode)?))
					} else {
						Err(FormatError::CodecNotFound.into())
					}
				}
			}

			pub struct [<$name Decoder>];

			impl [<$name Decoder>] {
				pub fn new(params: &mut CodecParams) -> Result<Box<dyn CodecImpl>> {
					use ::std::ffi::CString;
					use ::xx_core::pointer::*;
					use ::ffmpeg_sys_next::{avcodec_find_decoder_by_name, avcodec_find_decoder};

					let codec_name: Option<&'static str> = $codec_name;

					let mut codec = if let Some(name) = codec_name {
						let name = CString::new(name).unwrap();

						/* Safety: FFI call */
						unsafe { avcodec_find_decoder_by_name(name.as_ptr()) }.into()
					} else {
						Ptr::null()
					};

					if codec.is_null() {
						/* Safety: FFI call */
						codec = unsafe { avcodec_find_decoder($av_codec) }.into();
					}

					if !codec.is_null() {
						Ok(Box::new(AVCodec::new($codec_id, codec, params, Mode::Decode)?))
					} else {
						Err(FormatError::CodecNotFound.into())
					}
				}
			}
		}
	};
}

pub(super) use codec_pair;
