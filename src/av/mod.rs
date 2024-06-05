#![allow(unreachable_pub)]

use std::{
	any::Any,
	ffi::{c_char, c_void, CStr, CString},
	io::{Cursor, SeekFrom, Write},
	mem::{forget, transmute, zeroed},
	ops::{Deref, DerefMut},
	panic::*,
	str::from_utf8
};

use enumflags2::*;
pub use ffmpeg_sys_next::AVCodecID;
use ffmpeg_sys_next::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use xx_core::{
	async_std::io::*,
	coroutines::Context,
	ctor::ctor,
	error::*,
	impls::AsyncFnOnce,
	log::{internal::*, Level},
	opt::hint::*,
	os::error::OsError,
	paste::paste,
	pointer::*
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
mod opt;
mod packet;
mod parser;

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
pub use opt::*;
pub use packet::AVPacket;
pub use parser::*;

trait IntoResult {
	type Type;

	fn into_result(value: Self) -> Self::Type;
}

impl IntoResult for i32 {
	type Type = Result<Self>;

	fn into_result(value: Self) -> Self::Type {
		result_from_av(value)
	}
}

impl IntoResult for () {
	type Type = Self;

	fn into_result((): ()) {}
}

impl<T: ?Sized> IntoResult for *mut T {
	type Type = *mut T;

	fn into_result(value: Self) -> Self {
		value
	}
}

impl<T: ?Sized> IntoResult for *const T {
	type Type = *const T;

	fn into_result(value: Self) -> Self {
		value
	}
}

macro_rules! ffi {
	($func:ident $(, $args:expr)*) => {
		/* Safety: FFI call */
		IntoResult::into_result(unsafe { $func($($args,)*) })
	}
}

use ffi;

macro_rules! ffi_optional {
	($func:ident $(, $args:expr)*) => {
		/* Safety: FFI call */
		result_from_av_maybe_none(unsafe { $func($($args,)*) })
	}
}

use ffi_optional;

fn alloc_with<T, F>(alloc: F) -> MutPtr<T>
where
	F: FnOnce() -> *mut T
{
	let ptr = MutPtr::from(alloc());

	assert!(!ptr.is_null(), "Memory allocation failed");

	ptr
}

fn find_with<T, F>(find: F) -> Option<Ptr<T>>
where
	F: FnOnce() -> *const T
{
	let ptr = Ptr::from(find());

	if !ptr.is_null() {
		Some(ptr)
	} else {
		None
	}
}

unsafe fn get_av_class(context: MutPtr<()>) -> MutPtr<AVClass> {
	if !context.is_null() {
		/* Safety: guaranteed by caller */
		unsafe { ptr!(*context.cast::<MutPtr<AVClass>>()) }
	} else {
		MutPtr::null()
	}
}

#[allow(clippy::multiple_unsafe_ops_per_block)]
unsafe fn item_name<'a>(obj: MutPtr<()>, class: MutPtr<AVClass>) -> &'a str {
	let item_name = if !class.is_null() {
		/* Safety: guaranteed by caller */
		unsafe { ptr!(class=>item_name).unwrap_unchecked() }
	} else {
		av_default_item_name
	};

	/* Safety: get name */
	let str = unsafe { CStr::from_ptr(item_name(obj.as_mut_ptr().cast())) };

	str.to_str().unwrap_or("<error>")
}

#[allow(clippy::multiple_unsafe_ops_per_block)]
unsafe extern "C" fn log_callback(
	ptr: *mut c_void, level: i32, fmt: *const c_char, args: *mut __va_list_tag
) {
	let level = match level {
		AV_LOG_PANIC | AV_LOG_ERROR => Level::Error,
		AV_LOG_WARNING => Level::Warn,
		AV_LOG_INFO => Level::Info,
		AV_LOG_VERBOSE => Level::Debug,
		AV_LOG_DEBUG => Level::Trace,
		AV_LOG_TRACE => Level::Trace,
		_ => Level::Trace
	};

	if !log_enabled!(level) {
		return;
	}

	let mut target = Cursor::new([0u8; 1024]);
	let context = MutPtr::from(ptr).cast::<()>();

	/* Safety: all `ptr` store a MutPtr<AVClass> at the beginning for logging */
	let class = unsafe { get_av_class(context) };

	if !class.is_null() {
		/* Safety: valid pointer */
		let offset = unsafe { ptr!(class=>parent_log_context_offset) };

		if offset != 0 {
			/* Safety: assumes a valid AVClass declaration */
			let parent_context = unsafe {
				ptr!(*context
					.cast::<u8>()
					.offset(offset as isize)
					.cast::<MutPtr<()>>())
			};

			/* Safety: all `ptr` store a MutPtr<AVClass> at the beginning for logging */
			let parent_class = unsafe { get_av_class(parent_context) };

			if !parent_class.is_null() {
				/* Safety: class is valid */
				let _ = format_struct(&mut target, parent_context, unsafe {
					item_name(parent_context, parent_class)
				});

				let _ = target.write(b" | ");
			}
		}

		/* Safety: class is valid */
		let _ = format_struct(&mut target, context, unsafe { item_name(context, class) });
	}

	#[allow(clippy::cast_possible_truncation)]
	let pos = target.position() as usize;
	let target = from_utf8(&target.get_ref()[0..pos]).unwrap_or("<error>");

	/* Safety: repr C */
	let mut content = unsafe { zeroed() };

	ffi!(av_bprint_init, &mut content, 0, 0x01_0000);
	ffi!(av_vbprintf, &mut content, fmt, args);

	/* Safety: is always null terminated */
	let str = unsafe { CStr::from_ptr(content.str_).to_string_lossy() };

	log!(target: target, level, "{}", str);

	let _ = ffi!(
		av_bprint_finalize,
		&mut content,
		MutPtr::null().as_mut_ptr()
	);
}

#[ctor]
fn init_log() {
	ffi!(av_log_set_callback, Some(log_callback));
}
