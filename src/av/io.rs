use xx_core::macros::const_assert;
use xx_core::runtime::catch_unwind_safe;

use super::*;

enum Errors {
	None,
	Err(Error),
	Panic(Box<dyn Any + Send + 'static>)
}

impl Errors {
	fn fail(&mut self, error: Error) -> &Error {
		*self = Self::Err(error);

		match self {
			Self::Err(err) => err,

			/* Safety: just stored it */
			_ => unsafe { unreachable_unchecked() }
		}
	}
}

struct IoReader<'a> {
	context: &'a Context,
	reader: &'a mut Reader,
	error: Errors
}

#[asynchronous(sync)]
impl IoReader<'_> {
	/// # Safety
	/// valid ptr
	unsafe fn with_adapter<F, O: From<i32>>(adapter: *mut c_void, func: F) -> O
	where
		F: AsyncFnOnce(&mut Reader, &mut Errors) -> O
	{
		/* Safety: guaranteed by caller */
		let adapter: &mut Self = unsafe { ptr!(adapter).cast().as_mut() };

		/* Safety: perform async read */
		let result = catch_unwind_safe(|| unsafe {
			scoped(
				adapter.context,
				func.call_once((adapter.reader, &mut adapter.error))
			)
		});

		match result {
			Ok(n) => n,
			Err(err) => {
				adapter.error = Errors::Panic(err);

				AVERROR_BUG.into()
			}
		}
	}

	/// # Safety
	/// valid ptrs
	unsafe extern "C" fn read(adapter: *mut c_void, buf: *mut u8, buf_size: i32) -> i32 {
		let read = |reader: &mut Reader, error: &mut Errors| async move {
			let Ok(size) = buf_size.try_into() else {
				return AVERROR(OsError::Inval as i32);
			};

			/* Safety: exclusive access to the buffer */
			let buf = unsafe { MutPtr::slice_from_raw_parts(buf.into(), size).as_mut() };
			let result = reader.read_partial(buf).await;

			#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
			let err = match result {
				Ok(0) => return AVERROR_EOF,
				Ok(n) => return length_check(buf, n) as i32,
				Err(err) => err
			};

			if err == ErrorKind::UnexpectedEof {
				return AVERROR_EOF;
			}

			av_from_error(error.fail(err))
		};

		/* Safety: guaranteed by caller */
		unsafe { Self::with_adapter(adapter, read) }
	}

	/// # Safety
	/// valid ptrs
	#[allow(clippy::missing_const_for_fn)]
	unsafe extern "C" fn write(_: *mut c_void, _: *const u8, _: i32) -> i32 {
		AVERROR(OsError::Inval as i32)
	}

	/// # Safety
	/// valid ptrs
	#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
	unsafe extern "C" fn seek(adapter: *mut c_void, offset: i64, mut whence: i32) -> i64 {
		let seek = |reader: &mut Reader, error: &mut Errors| async move {
			if whence & AVSEEK_SIZE != 0 {
				return match reader.len().await {
					Ok(n) => n.try_into().unwrap(),
					Err(err) => av_from_error(error.fail(err)) as i64
				};
			}

			/* reader seek is force by default */
			whence &= !AVSEEK_FORCE;

			let seek = match whence {
				/* SEEK_SET */
				0 => SeekFrom::Start(offset.try_into().unwrap()),

				/* SEEK_CUR */
				1 => SeekFrom::Current(offset),

				/* SEEK_END */
				2 => SeekFrom::End(offset),

				_ => return AVERROR(OsError::Inval as i32) as i64
			};

			match reader.seek(seek).await {
				Ok(()) => reader.position().try_into().unwrap(),
				Err(err) => av_from_error(error.fail(err)) as i64
			}
		};

		/* Safety: guaranteed by caller */
		unsafe { Self::with_adapter(adapter, seek) }
	}
}

struct Buf(MutNonNull<u8>);

impl Drop for Buf {
	fn drop(&mut self) {
		ffi!(av_free, self.0.as_mut_ptr().cast());
	}
}

pub struct IoContext(MutNonNull<AVIOContext>);

impl IoContext {
	pub fn new() -> Self {
		const_assert!(DEFAULT_BUFFER_SIZE < i32::MAX as usize);

		let buf = Buf(alloc_with(|| ffi!(av_malloc, DEFAULT_BUFFER_SIZE)).cast());

		#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
		let ptr = alloc_with(|| {
			ffi!(
				avio_alloc_context,
				buf.0.as_mut_ptr().cast(),
				DEFAULT_BUFFER_SIZE as i32,
				0,
				MutPtr::null().as_mut_ptr(),
				Some(IoReader::read),
				Some(IoReader::write),
				Some(IoReader::seek)
			)
		});

		forget(buf);

		Self(ptr)
	}
}

ptr_deref!(IoContext, AVIOContext);

impl Drop for IoContext {
	fn drop(&mut self) {
		/* Safety: cannot be null */
		let buf = unsafe { MutNonNull::new_unchecked(self.buffer.into()) };

		/* free the buffer */
		let _ = Buf(buf.cast());

		let mut ptr = self.as_mut_ptr();

		/* we own this pointer */
		ffi!(avio_context_free, &mut ptr);
	}
}

pub struct Adapter<'a> {
	context: &'a mut IoContext,
	reader: &'a mut Reader
}

#[asynchronous]
impl<'a> Adapter<'a> {
	pub fn new(context: &'a mut IoContext, reader: &'a mut Reader) -> Self {
		Self { context, reader }
	}

	pub async fn with<F, Output>(&mut self, func: F) -> Result<Output>
	where
		F: AsyncFnOnce(&mut IoContext) -> Result<Output>
	{
		self.context.seekable = if self.reader.seekable() {
			AVIO_SEEKABLE_NORMAL
		} else {
			0
		};

		let mut reader = IoReader {
			context: get_context().await,
			reader: self.reader,
			error: Errors::None
		};

		self.context.opaque = ptr!(&mut reader).as_mut_ptr().cast();

		let result = func.call_once(self.context).await;

		self.context.opaque = MutPtr::null().as_mut_ptr();

		match reader.error {
			Errors::None => result,
			Errors::Err(err) => Err(err),
			Errors::Panic(panic) => resume_unwind(panic)
		}
	}
}
