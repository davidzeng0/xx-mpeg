#![allow(unreachable_pub)]

use std::{
	any::Any,
	ffi::{c_char, c_void, CStr, CString},
	io::SeekFrom,
	mem::{forget, transmute},
	ops::{Deref, DerefMut},
	panic::*
};

use enumflags2::*;
pub use ffmpeg_sys_next::AVCodecID;
use ffmpeg_sys_next::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use xx_core::{
	async_std::io::*, coroutines::Context, error::*, impls::AsyncFnOnce, opt::hint::*,
	os::error::OsError, paste::paste, pointer::*
};
use xx_pulse::*;

use crate::{format::*, reader::*, FormatError, Rational};

pub const UNKNOWN_TIMESTAMP: i64 = AV_NOPTS_VALUE;
pub const INPUT_BUFFER_PADDING: usize = AV_INPUT_BUFFER_PADDING_SIZE as usize;
pub const TIME_BASE: u32 = AV_TIME_BASE as u32;

mod codec;
mod conv;
mod defs;
mod error;
mod filter;
mod filters;
mod format;
mod frame;
mod io;
mod macros;
mod packet;

pub use codec::*;
use conv::*;
pub use defs::*;
pub use error::*;
pub use filter::*;
pub use filters::*;
pub use format::*;
pub use frame::AVFrame;
use io::*;
use macros::*;
pub use packet::AVPacket;

fn alloc_with<T, F>(alloc: F) -> MutPtr<T>
where
	F: FnOnce() -> *mut T
{
	let ptr = MutPtr::from(alloc());

	assert!(!ptr.is_null(), "Memory allocation failed");

	ptr
}
